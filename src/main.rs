extern crate dht;
#[macro_use]
extern crate structopt;

use dht::config::Config;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "dht",
            version = "0.1",
            author = "Benedikt Seidl, Stefan Su",
            about = "Distributed Hash Table based on Chord")]
struct Opt {
    /// Path to a custom config file
    #[structopt(short = "c", parse(from_os_str))]
    config: PathBuf,

    /// Address of a bootstrapping peer
    #[structopt(short = "b")]
    bootstrap: Option<SocketAddr>,

    /// Level of verbosity
    #[structopt(short = "v", parse(from_occurrences))]
    verbose: u8,
}

fn main() {
    let opt = Opt::from_args();

    let config = Config::load_from_file(opt.config).unwrap_or_else(|err| {
        println!("Argument error: {}", err);
        process::exit(2);
    });

    // TODO init logger with verbosity flag

    if let Err(e) = dht::run(&config, opt.bootstrap) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}
