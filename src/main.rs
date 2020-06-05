#[macro_use] extern crate lazy_static;

extern crate dotenv;
extern crate reqwest;
extern crate serde_json;
extern crate serde_derive;

use sqlx::{
    mysql::{
        MySqlPool,
    }
};

use serde_derive::{
    Deserialize,
    Serialize
};

use std::env;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    content: String,
}

lazy_static! {
    static ref DISCORD_TOKEN: String = {
        dotenv::dotenv().unwrap();

        format!("Bot {}", env::var("DISCORD_TOKEN").unwrap())
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;

    let interval = env::var("INTERVAL").unwrap().parse::<u64>().unwrap();

    const URL: &str = "https://discord.com/api/v6";

    let pool = MySqlPool::new(&env::var("DATABASE_URL").expect("No database URL provided")).await.unwrap();
    let req_client = reqwest::Client::new();

    loop {
        let query = sqlx::query!(
            "
SELECT channel, message, to_send FROM deletes WHERE `time` < NOW()
            "
        )
            .fetch_all(&pool)
            .await?;

        for row in query {
            if let Some(text) = row.to_send {
                let m = Message {
                    content: text,
                };

                send_message(format!("{}/channels/{}/messages", URL, row.channel), serde_json::to_string(&m).unwrap(), &req_client).await;
            }

            send_delete(format!("{}/channels/{}/messages/{}", URL, row.channel, row.message), &req_client).await;
        }

        sqlx::query!(
            "
DELETE FROM deletes WHERE `time` < NOW()
            "
        )
            .execute(&pool)
            .await?;

        tokio::time::delay_for(Duration::from_secs(interval)).await;
    }
}

async fn send_delete(url: String, client: &reqwest::Client) {
    client.delete(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", DISCORD_TOKEN.as_str())
        .send()
        .await.unwrap();
}

async fn send_message(url: String, m: String, client: &reqwest::Client) {
    client.post(&url)
        .body(m)
        .header("Content-Type", "application/json")
        .header("Authorization", DISCORD_TOKEN.as_str())
        .send()
        .await.unwrap();
}
