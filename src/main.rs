extern crate mysql;

extern crate dotenv;
extern crate reqwest;
extern crate threadpool;
extern crate serde_json;
extern crate serde_derive;

use std::env;
use std::thread;
use std::time::Duration;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    content: String,
}

fn main() {
    dotenv::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").unwrap();
    let sql_url = env::var("SQL_URL").unwrap();
    let interval = env::var("INTERVAL").unwrap().parse::<u64>().unwrap();
    let threads = env::var("THREADS").unwrap().parse::<usize>().unwrap();

    const URL: &str = "https://discordapp.com/api/v6";

    let mysql_conn = mysql::Pool::new(sql_url).unwrap();
    let req_client = reqwest::Client::new();
    let pool = threadpool::ThreadPool::new(threads);

    loop {
        pool.join();

        let q = mysql_conn.prep_exec("SELECT channel, message, to_send FROM deletes WHERE `time` < NOW()", ()).unwrap();

        for res in q {
            let (channel, m_id, text) = mysql::from_row::<(u64, u64, Option<String>)>(res.unwrap());

            if let Some(t) = text {
                let m = Message {
                    content: t,
                };

                let req = send_message(format!("{}/channels/{}/messages", URL, channel), serde_json::to_string(&m).unwrap(), &token, &req_client);

                pool.execute(move || {
                    let _ = req.send();
                });
            }

            let req = send(format!("{}/channels/{}/messages/{}", URL, channel, m_id), &token, &req_client);

            pool.execute(move || {
                let _ = req.send();
            });
        }
        mysql_conn.prep_exec("DELETE FROM deletes WHERE `time` < NOW()", ()).unwrap();

        thread::sleep(Duration::from_secs(interval));
    }
}

fn send(url: String, token: &str, client: &reqwest::Client) -> reqwest::RequestBuilder {
    client.delete(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bot {}", token))
}

fn send_message(url: String, m: String, token: &str, client: &reqwest::Client) -> reqwest::RequestBuilder {
    client.post(&url)
        .body(m)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bot {}", token))
}
