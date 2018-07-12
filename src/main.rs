#[macro_use]
extern crate clap;
extern crate dht;

use dht::config::Config;
use std::process;

fn main() {
    let matches = clap_app!(dht =>
        (version: "0.1")
        (author: "Benedikt Seidl, Stefan Su")
        (about: "Distributed Hash Table based on Chord")
        (@arg config: -c --config +takes_value * "Sets a custom config file")
        (@arg verbose: -v --verbose ... "Sets the level of verbosity")
    ).get_matches();

    let config_file = matches.value_of("config").unwrap();

    let config = Config::load_from_file(config_file).unwrap_or_else(|err| {
        println!("Argument error: {}", err);
        process::exit(2);
    });

    if let Err(e) = dht::run(config) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}
