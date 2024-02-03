use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rand::seq::SliceRandom;
use rand::Rng;
use reqwest::blocking::Client;
use reqwest::header;
use serde_json::json;
use sha2::{Digest, Sha256};

use proctitle::set_title;

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

fn generate_random_string(length: usize) -> String {
    let random_string: String = (0..length)
        .map(|_| format!("{:02x}", rand::thread_rng().gen_range(0..=255)))
        .collect();
    random_string
}

fn create_hash(input_string: &str) -> String {
    let mut sha256 = Sha256::new();

    sha256.update(input_string.as_bytes());

    let hashed_value = sha256.finalize();
    format!("{:x}", hashed_value)
}

fn get_timestamp() -> String {
    let time_idk = chrono::Local::now().format("%H:%M:%S").to_string();
    let timestamp = format!("[\x1b[90m{}\x1b[0m]", time_idk);
    timestamp
}

fn gen(proxy: Option<String>, counter: Arc<Mutex<Counter>>) {
    loop {
        let url = "https://api.discord.gx.games/v1/direct-fulfillment";

        let random_string = generate_random_string(64);
        let hashed_result = create_hash(&random_string);

        let data = json!({
            "partnerUserId": &hashed_result,
        });

        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT, header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36 OPR/105.0.0.0"));
        headers.insert(header::ACCEPT, header::HeaderValue::from_static("*/*"));
        headers.insert(
            header::ACCEPT_LANGUAGE,
            header::HeaderValue::from_static("fr-FR,fr;q=0.9,en-US;q=0.8,en;q=0.7"),
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::ORIGIN,
            header::HeaderValue::from_static("https://www.opera.com"),
        );
        headers.insert(
            header::REFERER,
            header::HeaderValue::from_static("https://www.opera.com/"),
        );
        
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
                    .default_headers(headers)
                    .build()
                    .unwrap()
            }
            None => Client::builder().default_headers(headers).build().unwrap(),
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

                            set_title(format!(
                                "OperaGX-DiscordNitro-Gen by Kisakay | Generated: {} | Proxy: {:?}",
                                counter.count,
                                proxy.clone()
                            ));

                            if let Ok(mut file) = OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open("promos.txt")
                                .map_err(|e| e.to_string())
                            {
                                writeln!(file, "{}", cleanify_link).ok();
                            }

                            drop(counter); // Release the lock before println and file write
                            println!(
                                "{} {} Generated Promo Link : {}",
                                get_timestamp(),
                                GREEN,
                                cleanify_link
                            );
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
