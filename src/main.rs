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

// fn main() {
//     // let free_games = epic_client::get_free_games();
//     // println!("{:?}", free_games);
//     // discord::post_free_games_message(free_games);
//     println!("Hello world!");
// }

use warp::{Filter, Rejection};
use std::sync::{Arc, Mutex};
use std::env;

#[tokio::main]
async fn main() {
    let (token, channel_id) = read_env();
    env_logger::builder()
        .filter_module("sentyalie", LevelFilter::Info)
        .init();

    info!("token: {}, id: {}", token, channel_id);
    info!("Starting..");

    let (tx, rx) = tokio::sync::oneshot::channel();
    let get_shutdown_hook = Arc::new(Mutex::new(Some(tx)));
    let run_shutdown_hook = get_shutdown_hook.clone();
    let shutdown_hook = run_shutdown_hook.clone();

    let ping = warp::path!("ping")
        .map(||{
            info!("ping");
            "pong"
        });

    let run = warp::path!("run")
        .and_then(move || {
            let token = token.clone();
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

    let routes = ping.or(run).or(get).or(shutdown);
    let (addr, server) = warp::serve(routes)
        .bind_with_graceful_shutdown(([0, 0, 0, 0], 8080), async {
            rx.await.ok();
        });

    server.await;
}

fn read_env() -> (String, String) {
    match (env::var("DISCORD_TOKEN"), env::var("DISCORD_CHANNEL")) {
        (Ok(token), Ok(channel)) => (token, channel),
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

