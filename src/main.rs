use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rand::seq::SliceRandom;
use reqwest::blocking::Client;
use serde_json::json;
use std::fs::OpenOptions;
use uuid::Uuid;

struct Counter {
    count: usize,
}

impl Counter {
    fn new() -> Counter {
        Counter { count: 0 }
    }
}

static RED: &str = "\x1b[31m(-)\x1b[0m";
static BLUE: &str = "\x1b[34m(+)\x1b[0m";
static GREEN: &str = "\x1b[32m(+)\x1b[0m";
static YELLOW: &str = "\x1b[33m(!)\x1b[0m";

fn get_timestamp() -> String {
    let time_idk = chrono::Local::now().format("%H:%M:%S").to_string();
    let timestamp = format!("[\x1b[90m{}\x1b[0m]", time_idk);
    timestamp
}

fn gen(proxy: Option<String>, counter: Arc<Mutex<Counter>>) {
    loop {
        let url = "https://api.discord.gx.games/v1/direct-fulfillment";

        let data = json!({
            "partnerUserId": Uuid::new_v4().to_string(),
        });

        let client = match &proxy {
            Some(p) => {
                let credentials: Vec<&str> = p.split('@').collect();
                let user_pass: Vec<&str> = credentials[0].split(':').collect();
                let host_port: Vec<&str> = credentials[1].split(':').collect();

                let formatted_proxy = format!(
                    "http://{}:{}@{}:{}",
                    user_pass[0], user_pass[1], host_port[0], host_port[1]
                );

                Client::builder()
                    .proxy(reqwest::Proxy::http(&formatted_proxy).unwrap())
                    .proxy(reqwest::Proxy::https(&formatted_proxy).unwrap())
                    .build()
                    .unwrap()
            }
            None => Client::new(),
        };

        match client.post(url).json(&data).send() {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(json) = response.json::<serde_json::Value>() {
                        if let Some(token) = json.get("token") {
                            let link = format!(
                                "https://discord.com/billing/partner-promotions/1180231712274387115/{}",
                                token
                            );

                            let cleanify_link = link.replace('"', "");

                            let mut counter = counter.lock().unwrap();
                            counter.count += 1;
                            drop(counter); // Release the lock before println and file write
                            println!(
                                "{} {} Generated Promo Link : {}",
                                get_timestamp(),
                                GREEN,
                                cleanify_link
                            );

                            if let Ok(mut file) = OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open("promos.txt")
                                .map_err(|e| e.to_string())
                            {
                                writeln!(file, "{}", cleanify_link).ok();
                            }
                        }
                    }
                } else if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    println!("{} {} You are being rate-limited!", get_timestamp(), YELLOW);
                } else {
                    println!(
                        "{} {} Request failed : {}",
                        get_timestamp(),
                        RED,
                        response.status()
                    );
                }
            }
            Err(e) => {
                println!("{} {} Request Failed : {}", get_timestamp(), RED, e);
            }
        }

        // Pause entre les requêtes
        thread::sleep(Duration::from_secs(5));
    }
}

fn main() {
    let num_threads: u32 = loop {
        println!("{} {} Enter Number Of Threads : ", get_timestamp(), BLUE);
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        match input.trim().parse() {
            Ok(n) => break n,
            Err(_) => println!("{} {} Please enter a valid number", get_timestamp(), RED),
        }
    };

    let mut threads = vec![];

    let proxies: Vec<String> = match File::open("./proxies.txt") {
        Ok(file) => BufReader::new(file)
            .lines()
            .filter_map(|line| line.ok())
            .collect(),
        Err(e) => {
            eprintln!(
                "{} {} Unable to open proxies.txt: {}",
                get_timestamp(),
                RED,
                e
            );
            vec![]
        }
    };

    let counter = Arc::new(Mutex::new(Counter::new()));

    for _ in 0..num_threads {
        let proxy = proxies.choose(&mut rand::thread_rng()).cloned();
        let counter_clone = Arc::clone(&counter);

        let handle = thread::spawn(move || {
            gen(proxy, counter_clone);
        });

        threads.push(handle);
    }

    for handle in threads {
        handle.join().unwrap();
    }
}
