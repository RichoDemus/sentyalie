use serde::{Deserialize, Serialize};

use crate::Game;

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

pub(crate) async fn post_free_games_message(
    base_url: &str,
    games: Vec<Game>,
    token: &str,
    channel_id: &str,
) {
    let games = games
        .into_iter()
        .map(|game| game.title)
        .collect::<Vec<String>>()
        .join(", ");
    let body = Message {
        content: format!("Free games this week: {}", games),
    };

    let client = reqwest::Client::default();
    let _res = client
        .post(format!(
            "{base_url}/api/channels/{channel_id}/messages",
            base_url = base_url,
            channel_id = channel_id
        ))
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

pub(crate) async fn post_free_games_direct_message(
    base_url: &str,
    games: Vec<Game>,
    token: &str,
    user_id: &str,
) {
    let games = games
        .into_iter()
        .map(|game| game.title)
        .collect::<Vec<String>>()
        .join(", ");
    let body = Message {
        content: format!("Free games this week: {}", games),
    };

    let client = reqwest::Client::default();

    let dm_channel: DmChannel = client
        .post(format!(
            "{base_url}/api/users/@me/channels",
            base_url = base_url
        ))
        .header("Authorization", format!("Bot {}", token))
        .header("Content-Type", "application/json")
        .body(format!("{{\"recipient_id\":\"{}\"}}", user_id))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let _res = client
        .post(format!(
            "{base_url}/api/channels/{channel_id}/messages",
            base_url = base_url,
            channel_id = dm_channel.id
        ))
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
