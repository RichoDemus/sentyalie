use log::{debug, error, log_enabled, info, Level, LevelFilter};
use serde::{Deserialize, Serialize};

mod epic_client;
mod discord;

// Use Jemalloc only for musl-64 bits platforms
#[cfg(all(target_env = "musl", target_pointer_width = "64"))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[derive(Eq, PartialEq, Debug, Serialize)]
pub(crate) enum Platform {
    Epic, _Steam
}

#[derive(Eq, PartialEq, Debug, Serialize)]
pub(crate) struct Game {
    title: String,
    platform: Platform,
}

use warp::{Filter, Rejection};
use std::sync::{Arc, Mutex};
use std::env;

#[tokio::main]
async fn main() {
    let (token, channel_id, user_id) = read_env();
    env_logger::builder()
        .filter_module("sentyalie", LevelFilter::Info)
        .init();

    info!("token: {}, id: {}", token, channel_id);
    info!("Starting..");

    // todo figure all this out
    let run_token = token.clone();
    let test_token = token;

    let (tx, rx) = tokio::sync::oneshot::channel();
    let get_shutdown_hook = Arc::new(Mutex::new(Some(tx)));
    let run_shutdown_hook = get_shutdown_hook.clone();
    let test_shutdown_hook = get_shutdown_hook.clone();
    let shutdown_hook = run_shutdown_hook.clone();

    let ping = warp::path!("ping")
        .map(||{
            info!("ping");
            "pong"
        });

    let run = warp::path!("run")
        .and_then(move || {
            let token = run_token.clone();
            let channel_id = channel_id.clone();
            let tx = run_shutdown_hook.clone();
            async move {
                info!("run");
                let free_games = epic_client::get_free_games().await;
                info!("free games: {:?}", free_games);
                discord::post_free_games_message(free_games, &token, &channel_id).await;
                if let Some(tx) = tx.lock().unwrap().take() {
                    tx.send(());
                }
                Ok::<_,Rejection>(warp::reply())
            }
        });

    let test = warp::path!("test")
        .and_then(move || {
            let token = test_token.clone();
            let user_id = user_id.clone();
            let tx = test_shutdown_hook.clone();
            async move {
                info!("run");
                let free_games = epic_client::get_free_games().await;
                info!("free games: {:?}", free_games);
                discord::post_free_games_direct_message(free_games, &token, &user_id).await;
                if let Some(tx) = tx.lock().unwrap().take() {
                    tx.send(());
                }
                Ok::<_,Rejection>(warp::reply())
            }
        });

    let get = warp::path!("get")
        .and_then(move|| {
            let tx = get_shutdown_hook.clone();
            async move {
                info!("get");
                let free_games = epic_client::get_free_games().await;
                info!("free games: {:?}", free_games);
                let free_games = serde_json::to_string(&free_games).expect("should work");
                if let Some(tx) = tx.clone().lock().unwrap().take() {
                    tx.send(());
                }
                Ok::<_,Rejection>(free_games)
            }
        });

    let shutdown = warp::path!("shutdown")
        .map(move || {
            info!("shutdown");
            if let Some(tx) = shutdown_hook.clone().lock().unwrap().take() {
                tx.send(());
            }
            warp::reply()
        });

    let routes = ping.or(run).or(get).or(test).or(shutdown);
    let (addr, server) = warp::serve(routes)
        .bind_with_graceful_shutdown(([0, 0, 0, 0], 8080), async {
            rx.await.ok();
        });

    server.await;
}

fn read_env() -> (String, String, String) {
    match (env::var("DISCORD_TOKEN"), env::var("DISCORD_CHANNEL"), env::var("DISCORD_TEST_USER")) {
        (Ok(token), Ok(channel), Ok(user)) => (token, channel, user),
        _ => panic!("Missing env"),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn asd() {

    }
}

