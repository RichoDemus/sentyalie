use std::env;
use std::future::Future;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use log::{info, LevelFilter};
use serde::Serialize;
use warp::{Filter, Rejection};

mod discord;
mod epic_client;
#[cfg(test)]
mod test;

// Use Jemalloc only for musl-64 bits platforms
#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[derive(Eq, PartialEq, Debug, Serialize)]
pub(crate) enum Platform {
    Epic,
    _Steam,
}

#[derive(Eq, PartialEq, Debug, Serialize)]
pub(crate) struct Game {
    title: String,
    platform: Platform,
}

#[derive(Debug, Clone)]
pub(crate) struct Config {
    port: u16,
    token: String,
    channel_id: String,
    user_id: String,
    epic_base_url: String,
    discord_base_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 8080,
            token: "".to_string(),
            channel_id: "".to_string(),
            user_id: "".to_string(),
            epic_base_url: "https://store-site-backend-static.ak.epicgames.com".to_string(),
            discord_base_url: "https://discordapp.com".to_string(),
        }
    }
}

#[tokio::main]
async fn main() {
    let (token, channel_id, user_id) = read_env();
    env_logger::builder()
        .filter_module("sentyalie", LevelFilter::Info)
        .init();

    let (server, _) = run(
        Config {
            token,
            channel_id,
            user_id,
            ..Default::default()
        },
        Utc::now(),
    );
    server.await;
}

fn run(config: Config, now: DateTime<Utc>) -> (impl Future<Output = ()>, u16) {
    info!("Config {:?}", config);
    info!("Starting..");

    // todo figure all this out
    let run_token = config.token.clone();
    let test_token = config.token.clone();

    let user_id = config.user_id.clone();
    let channel_id = config.channel_id.clone();
    let epic_base_url = Arc::new(config.epic_base_url.clone());
    let discord_base_url = Arc::new(config.discord_base_url.clone());
    let now = Arc::new(now);

    let (tx, rx) = tokio::sync::oneshot::channel();
    let shutdown_hook = Arc::new(Mutex::new(Some(tx)));

    let ping = warp::path!("ping").map(|| {
        info!("ping");
        "pong"
    });

    let epic_base_url_clone = epic_base_url.clone();
    let discord_base_url_clone = discord_base_url.clone();
    let now_clone = now.clone();
    let run = warp::path!("run").and_then(move || {
        let token = run_token.clone();
        let channel_id = channel_id.clone();
        let epic_base_url = epic_base_url_clone.clone();
        let discord_base_url_clone = discord_base_url_clone.clone();
        let now_clone = now_clone.clone();
        async move {
            info!("run");
            let free_games = epic_client::get_free_games(&epic_base_url, *now_clone).await;
            info!("free games: {:?}", free_games);
            discord::post_free_games_message(
                &discord_base_url_clone,
                free_games,
                &token,
                &channel_id,
            )
            .await;
            Ok::<_, Rejection>(warp::reply())
        }
    });

    let epic_base_url_clone = epic_base_url.clone();
    let discord_base_url_clone = discord_base_url.clone();
    let now_clone = now.clone();
    let test = warp::path!("test").and_then(move || {
        let token = test_token.clone();
        let user_id = user_id.clone();
        let epic_base_url = epic_base_url_clone.clone();
        let discord_base_url_clone = discord_base_url_clone.clone();
        let now_clone = now_clone.clone();
        async move {
            info!("run");
            let free_games = epic_client::get_free_games(&epic_base_url, *now_clone).await;
            info!("free games: {:?}", free_games);
            discord::post_free_games_direct_message(
                &discord_base_url_clone,
                free_games,
                &token,
                &user_id,
            )
            .await;
            Ok::<_, Rejection>(warp::reply())
        }
    });

    let epic_base_url_clone = epic_base_url.clone();
    let now_clone = now.clone();
    let get = warp::path!("get").and_then(move || {
        let epic_base_url = epic_base_url_clone.clone();
        let now_clone = now_clone.clone();
        async move {
            info!("get");
            let free_games = epic_client::get_free_games(&epic_base_url, *now_clone).await;
            info!("free games: {:?}", free_games);
            let free_games = serde_json::to_string(&free_games).expect("should work");
            Ok::<_, Rejection>(free_games)
        }
    });

    let shutdown = warp::path!("shutdown").map(move || {
        info!("shutdown");
        if let Some(tx) = shutdown_hook.clone().lock().unwrap().take() {
            tx.send(()).unwrap();
        }
        warp::reply()
    });

    let routes = ping.or(run).or(get).or(test).or(shutdown);
    let (addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], config.port), async {
            rx.await.ok();
        });

    (server, addr.port())
}

fn read_env() -> (String, String, String) {
    match (
        env::var("DISCORD_TOKEN"),
        env::var("DISCORD_CHANNEL"),
        env::var("DISCORD_TEST_USER"),
    ) {
        (Ok(token), Ok(channel), Ok(user)) => (token, channel, user),
        _ => panic!("Missing env"),
    }
}
