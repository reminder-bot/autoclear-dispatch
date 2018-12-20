extern crate mysql;

extern crate dotenv;
extern crate reqwest;
extern crate threadpool;

use std::env;
use std::thread;
use std::time::Duration;


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

        let q = mysql_conn.prep_exec("SELECT channel, message FROM deletes WHERE `time` < NOW()", ()).unwrap();

        for res in q {
            let (channel, message) = mysql::from_row::<(u64, u64)>(res.unwrap());

            let req = send(format!("{}/channels/{}/messages/{}", URL, channel, message), &token, &req_client);

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
