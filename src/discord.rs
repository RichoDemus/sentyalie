use serde::{Deserialize, Serialize};
use crate::Game;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct Guild {
    id: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Channel {
    id: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DmChannel {
    id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    content: String,
}

pub(crate) async fn post_free_games_message(base_url: &str, games: Vec<Game>, token: &str, channel_id: &str) {
    let games = games.into_iter()
        .map(|game|game.title)
        .collect::<Vec<String>>()
        .join(", ");
    let body = Message {
        content: format!("Free games this week: {}", games),
    };

    let client = reqwest::Client::default();
    let _res = client.post(format!("{base_url}/api/channels/{channel_id}/messages", base_url=base_url, channel_id=channel_id))
        .header("Authorization", format!("Bot {}", token))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&body).unwrap())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
}

pub(crate) async fn post_free_games_direct_message(base_url: &str, games: Vec<Game>, token: &str, user_id: &str) {
    let games = games.into_iter()
        .map(|game|game.title)
        .collect::<Vec<String>>()
        .join(", ");
    let body = Message {
        content: format!("Free games this week: {}", games),
    };

    let client = reqwest::Client::default();

    let dm_channel:DmChannel = client.post(format!("{base_url}/api/users/@me/channels", base_url=base_url))
        .header("Authorization", format!("Bot {}", token))
        .header("Content-Type", "application/json")
        .body(format!("{{\"recipient_id\":\"{}\"}}", user_id))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let _res = client.post(format!("{base_url}/api/channels/{channel_id}/messages", base_url=base_url, channel_id=dm_channel.id))
        .header("Authorization", format!("Bot {}", token))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&body).unwrap())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    fn test_get_guilds() {
        let token = "";

        let client = reqwest::blocking::Client::default();
        let guilds:Vec<Guild> = client.get("https://discordapp.com/api/users/@me/guilds")
            .header("Authorization", format!("Bot {}", token))
            .send()
            .unwrap()
            .json()
            .unwrap();

        println!("{:?}", guilds);

        for guild in guilds {
            let channels: Vec<Channel> = client.get(format!("https://discordapp.com/api/guilds/{guild_id}/channels", guild_id=guild.id))
                .header("Authorization", format!("Bot {}", token))
                .send()
                .unwrap()
                .json()
                .unwrap();

            for channel in channels {
                println!("\t{:?}", channel)
            }
        }

    }

    // #[test]
    fn send_message() {
        let token = "";
        let channel_id = "";
        let client = reqwest::blocking::Client::default();
        let guilds = client.post(format!("https://discordapp.com/api/channels/{channel_id}/messages", channel_id=channel_id))
            .header("Authorization", format!("Bot {}", token))
            .header("Content-Type", "application/json")
            .body(r#"{"content":"hello world"}"#)
            .send()
            .unwrap()
            .text()
            .unwrap();
    }

    //#[test]
    fn send_private_message() {
        let token = "";
        let client = reqwest::blocking::Client::default();
        let dm_channel:DmChannel = client.post("https://discordapp.com/api/users/@me/channels")
            .header("Authorization", format!("Bot {}", token))
            .header("Content-Type", "application/json")
            .body(r#"{"recipient_id":""}"#)
            .send()
            .unwrap()
            .json()
            .unwrap();

        println!("channel: {:?}", dm_channel);

        let _response = client.post(format!("https://discordapp.com/api/channels/{channel_id}/messages", channel_id=dm_channel.id))
            .header("Authorization", format!("Bot {}", token))
            .header("Content-Type", "application/json")
            .body(r#"{"content":"hello world"}"#)
            .send()
            .unwrap()
            .text()
            .unwrap();
    }
}
