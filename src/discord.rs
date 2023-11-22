use chrono::{DateTime, Utc};
use log::info;
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
struct PostMessage {
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    content: String,
    timestamp: DateTime<Utc>,
    author: Author,
}

#[derive(Serialize, Deserialize, Debug)]
struct Author {
    username: String,
}

pub(crate) async fn get_last_posted_message(
    base_url: &str,
    token: &str,
    channel_id: &str,
) -> Option<String> {
    let client = reqwest::Client::default();
    let url = format!(
        "{base_url}/api/channels/{channel_id}/messages",
        base_url = base_url,
        channel_id = channel_id
    );
    info!("Url: {url}");
    let res = client
        .get(url)
        .header("Authorization", format!("Bot {}", token))
        .header("Content-Type", "application/json")
        .send()
        .await
        .unwrap();

    info!("headers: {:?}", res.headers());
    let json = res.text().await.unwrap();
    info!("res: {}", json);
    let parsed: Vec<Message> = serde_json::from_str(json.as_str()).unwrap();
    info!("parsed: {:?}", parsed);

    parsed
        .into_iter()
        .filter(|msg| msg.author.username == "free-game-announcer")
        .max_by_key(|msg| msg.timestamp)
        .map(|msg| msg.content)
}

pub(crate) async fn post_free_games_message(
    base_url: &str,
    content: String,
    token: &str,
    channel_id: &str,
) {
    let body = PostMessage { content };

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

pub(crate) fn create_message(games: &[Game]) -> String {
    let games = games
        .iter()
        .map(|game| game.title.clone()) // todo don't clone
        .collect::<Vec<String>>()
        .join(", ");
    let message = format!("Free games this week: {}", games);
    message
}

pub(crate) async fn post_free_games_direct_message(
    base_url: &str,
    content: String,
    token: &str,
    user_id: &str,
) {
    let body = PostMessage { content };

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
