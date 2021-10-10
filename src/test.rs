#[cfg(test)]
mod tests {
    use httpmock::prelude::*;

    use super::*;
    use crate::{run, Config};
    use chrono::{Utc, TimeZone};
    use std::time::Duration;
    use tokio::task::JoinHandle;

    fn start_server() -> (JoinHandle<()>, MockServer, Config) {
        let server = MockServer::start();

        let config = Config {
            port: 0,
            channel_id: String::from("channel-id"),
            epic_base_url: server.base_url(),
            discord_base_url: server.base_url(),
            ..Default::default()
        };
        let (target, port) = run(config.clone(), Utc.timestamp(1631467068, 0));
        let server_handle = tokio::spawn(target);
        (server_handle, server, Config {
            port,
            ..config
        })
    }

    #[tokio::test]
    async fn start_and_shutdown() {
        let (server_handle, _, config) = start_server();

        reqwest::get(format!("http://localhost:{port}/shutdown", port=config.port)).await.unwrap();

        server_handle.await;
    }

    #[tokio::test]
    async fn post_games_to_discord() {
        let (server_handle, server, config) = start_server();

        let epic_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/freeGamesPromotions");
            then.status(200)
                .body(include_str!("epic_response.json"));
        });

        let discord_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/api/channels/{channel_id}/messages", channel_id=config.channel_id))
                .body(r#"{"content":"Free games this week: Sheltered, Nioh: The Complete Edition"}"#);
            then.status(200);
        });

        let response = reqwest::get(format!("http://localhost:{port}/run", port=config.port)).await.unwrap().text().await.unwrap();

        discord_mock.assert();

        reqwest::get(format!("http://localhost:{port}/shutdown", port=config.port)).await.unwrap();
        server_handle.await;
    }

    #[tokio::test]
    async fn post_games_to_discord_private_message() {
        let (server_handle, server, config) = start_server();

        let epic_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/freeGamesPromotions");
            then.status(200)
                .body(include_str!("epic_response.json"));
        });

        let discord_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/users/@me/channels");
            then.status(200).body(r#"{"id":"cool-id"}"#);
        });

        let discord_mock2 = server.mock(|when, then| {
            when.method(POST)
                .path("/api/channels/cool-id/messages")
                .body(r#"{"content":"Free games this week: Sheltered, Nioh: The Complete Edition"}"#);
            then.status(200);
        });

        let response = reqwest::get(format!("http://localhost:{port}/test", port=config.port)).await.unwrap().text().await.unwrap();

        discord_mock2.assert();

        reqwest::get(format!("http://localhost:{port}/shutdown", port=config.port)).await.unwrap();
        server_handle.await;
    }
}
