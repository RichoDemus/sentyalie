#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use httpmock::prelude::*;
    use tokio::task::JoinHandle;

    use crate::{run, Config};

    fn start_server() -> (JoinHandle<()>, MockServer, Config) {
        let server = MockServer::start();

        let config = Config {
            port: 0,
            channel_id: String::from("channel-id"),
            user_id: String::from("user-id"),
            token: String::from("token"),
            epic_base_url: server.base_url(),
            discord_base_url: server.base_url(),
            ..Default::default()
        };
        let (target, port) = run(config.clone(), Utc.timestamp(1631467068, 0));
        let server_handle = tokio::spawn(target);

        let _epic_mock = server.mock(|when, then| {
            when.method(GET).path("/freeGamesPromotions");
            then.status(200).body(include_str!("epic_response.json"));
        });

        (server_handle, server, Config { port, ..config })
    }

    #[tokio::test]
    async fn start_and_shutdown() {
        let (server_handle, _, config) = start_server();

        reqwest::get(format!(
            "http://localhost:{port}/shutdown",
            port = config.port
        ))
        .await
        .unwrap();

        server_handle.await.unwrap();
    }

    #[tokio::test]
    async fn post_games_to_discord() {
        let (server_handle, server, config) = start_server();

        let discord_mock = server.mock(|when, then| {
            when.method(POST)
                .path(format!(
                    "/api/channels/{channel_id}/messages",
                    channel_id = config.channel_id
                ))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bot {}", config.token).as_str())
                .body(
                    r#"{"content":"Free games this week: Sheltered, Nioh: The Complete Edition"}"#,
                );
            then.status(200);
        });

        let response = reqwest::get(format!("http://localhost:{port}/run", port = config.port))
            .await
            .unwrap();
        assert_eq!(response.status(), 200);

        discord_mock.assert();

        reqwest::get(format!(
            "http://localhost:{port}/shutdown",
            port = config.port
        ))
        .await
        .unwrap();
        server_handle.await.unwrap();
    }

    #[tokio::test]
    async fn post_games_to_discord_private_message() {
        let (server_handle, server, config) = start_server();

        let _discord_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/api/users/@me/channels")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bot {}", config.token).as_str())
                .body(format!("{{\"recipient_id\":\"{}\"}}", config.user_id));
            then.status(200).body(r#"{"id":"cool-id"}"#);
        });

        let discord_mock2 = server.mock(|when, then| {
            when.method(POST)
                .path("/api/channels/cool-id/messages")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bot {}", config.token).as_str())
                .body(
                    r#"{"content":"Free games this week: Sheltered, Nioh: The Complete Edition"}"#,
                );
            then.status(200);
        });

        let response = reqwest::get(format!("http://localhost:{port}/test", port = config.port))
            .await
            .unwrap();
        assert_eq!(response.status(), 200);

        discord_mock2.assert();

        reqwest::get(format!(
            "http://localhost:{port}/shutdown",
            port = config.port
        ))
        .await
        .unwrap();
        server_handle.await.unwrap();
    }
}
