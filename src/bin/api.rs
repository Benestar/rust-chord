extern crate dht;
extern crate structopt;

use dht::config::Config;
use dht::message::api::{DhtGet, DhtPut};
use dht::message::Message;
use dht::network::Connection;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "api",
    version = "0.1",
    author = "Benedikt Seidl, Stefan Su",
    about = "Client to talk to the DHT api"
)]
struct Opt {
    /// Path to a custom config file
    #[structopt(short = "c", parse(from_os_str))]
    config: PathBuf,
}

fn main() {
    let opt = Opt::from_args();

    let config = Config::load_from_file(opt.config).unwrap_or_else(|err| {
        eprintln!("Argument error: {}", err);
        process::exit(2);
    });

    println!("Client to talk to the DHT api");
    println!("-----------------------------\n");

    loop {
        let command = read_line("Enter a command").unwrap();

        if "put" == command {
            handle_put(config);
        }

        if "get" == command {
            handle_get(config);
        }
    }
}

fn read_line(question: &str) -> Option<String> {
    print!("{}: ", question);
    io::stdout().flush().unwrap();

    let mut line = String::new();

    match io::stdin().read_line(&mut line) {
        Ok(_) => Some(line.trim().to_string()),
        Err(err) => {
            eprintln!("Error: {}", err);
            None
        }
    }
}

fn handle_put(config: Config) {
    let key = read_line("Enter a key").unwrap();
    let value = read_line("Enter a value").unwrap();

    let len = std::cmp::min(32, key.len());

    let mut raw_key = [0; 32];
    raw_key[..len].copy_from_slice(&key.as_bytes()[..len]);

    let dht_put = DhtPut {
        ttl: 10,
        replication: 2,
        key: raw_key,
        value: value.as_bytes().to_vec(),
    };

    let mut con = Connection::open(config.api_address, config.timeout).unwrap();
    con.send(&Message::DhtPut(dht_put)).unwrap();

    println!("Sent a DHT PUT message to {}", config.api_address);
}

fn handle_get(config: Config) {
    let key = read_line("Enter a key").unwrap();

    let len = std::cmp::min(32, key.len());

    let mut raw_key = [0; 32];
    raw_key[..len].copy_from_slice(&key.as_bytes()[..len]);

    let dht_get = DhtGet { key: raw_key };

    let mut con = Connection::open(config.api_address, config.timeout).unwrap();
    con.send(&Message::DhtGet(dht_get)).unwrap();

    match con.receive().unwrap() {
        Message::DhtSuccess(dht_success) => {
            let key = std::str::from_utf8(&dht_success.key).unwrap();
            let value = std::str::from_utf8(&dht_success.value).unwrap();
            println!("Received value for key {}:\n\n{}", key, value);
        }
        Message::DhtFailure(dht_failure) => {
            let key = std::str::from_utf8(&dht_failure.key).unwrap();
            println!("Failed to retrieve value for key {}", key);
        }
        msg => eprintln!("Unexpected message of type {}", msg),
    }
}
