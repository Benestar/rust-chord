extern crate dht;
#[macro_use]
extern crate log;
extern crate stderrlog;
extern crate structopt;

use dht::config::Config;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "dht",
    version = "0.1",
    author = "Benedikt Seidl, Stefan Su",
    about = "Distributed Hash Table based on Chord"
)]
struct Opt {
    /// Path to a custom config file
    #[structopt(short = "c", parse(from_os_str))]
    config: PathBuf,

    /// Address of a bootstrapping peer
    #[structopt(short = "b")]
    bootstrap: Option<SocketAddr>,

    /// Silence all output
    #[structopt(short = "q", long = "quiet")]
    quiet: bool,

    /// Level of verbosity (v, vv, vvv)
    #[structopt(short = "v", parse(from_occurrences))]
    verbose: usize,

    /// Timestamp (sec, ms, ns, none)
    #[structopt(short = "t")]
    timestamp: Option<stderrlog::Timestamp>,
}

fn main() {
    let opt = Opt::from_args();

    // init logger with verbosity flag
    stderrlog::new()
        .module(module_path!())
        .quiet(opt.quiet)
        .verbosity(opt.verbose)
        .timestamp(opt.timestamp.unwrap_or(stderrlog::Timestamp::Off))
        .init()
        .expect("Failed to initialize logger");

    let config = Config::load_from_file(opt.config).unwrap_or_else(|err| {
        error!("Error while loading config file: {}", err);
        process::exit(2);
    });

    if let Err(e) = dht::run(config, opt.bootstrap) {
        error!("Fatal application error: {}", e);
        process::exit(1);
    }
}
