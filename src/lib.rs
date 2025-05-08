//! This crate contains a [distributed hash table (DHT)][w:dht] implementation
//! based on the [Chord protocol][w:chord] using [consistent hashing][w:cons].
//!
//! # Introduction
//!
//! A DHT allows to store key-value pairs in a distributed network of peers.
//! No peer has to store the whole hash table but every node in the network
//! provides some storage and computing capacity to realize a functional
//! system.
//!
//! We distinguish between the api interface which is used to communicate with
//! other modules on the same peer and the peer-to-peer or inter-module
//! interface which allows the DHT modules on different peers to interact.
//! The DHT provides two operations via its api interface, namely PUT and GET,
//! which are used to store a value under a given key and later obtain the
//! value for this key. The peer-to-peer protocol is more complicated and
//! consists of two layers. The storage layer supports PUT and GET operations
//! just like the api interface while the routing layer implements standard
//! Chord functionality like finding a peer responsible for a given identifier.
//!
//! # Architecture Design
//!
//! ## Application Architecture
//!
//! To realize the distributed hash table we will implement the Chord protocol.
//! The central aspect of Chord is to provide a distributed lookup method. This
//! means to map a given key to a node in the network. The important aspects
//! are load balancing such that all nodes store approximately the same amount
//! of data, failure handling and recovery as described in a later section and
//! efficiency in the routing process.
//!
//! On top of this core functionality, we can implement a file storage system
//! which uses Chord to find one or several peers to store a file for a given
//! key. By separating these to layers of functionality, we can keep our
//! routing implementation as simple as possible and perform the file storage
//! operations separately. This also allows to implement redundancy and error
//! handling on a higher level.
//!
//! ## Process Architecture
//!
//! This DHT implementation is be based on TCP for both the api interface and
//! the peer-to-peer communication. Therefore, we listen on two addresses given
//! in the config for the two interfaces and wait for incoming connections in
//! two event loops. Each incoming request should be handled as fast as
//! possible. Therefore we use parallelization to balance the load on several
//! cores.
//!
//! Since we need to work on shared memory between requests and also because we
//! expect each request to only take very short to process, our preferred
//! solution for parallelization is multi-threading. For this purpose, we use
//! the thread pool pattern which creates a given number of worker threads and
//! handles jobs from a job queue. Whenever a request reaches our server, it
//! creates a new task to handle this request and adds it to the queue. This
//! allows us to work concurrently while not having the overhead of spawning
//! too many threads.
//!
//! [w:dht]: https://en.wikipedia.org/wiki/Distributed_hash_table
//! [w:chord]: https://en.wikipedia.org/wiki/Chord_(peer-to-peer)
//! [w:cons]: https://en.wikipedia.org/wiki/Consistent_hashing

use crate::config::Config;
use crate::handler::{ApiHandler, P2PHandler};
use crate::network::Server;
use crate::routing::Routing;
use crate::stabilization::{Bootstrap, Stabilization};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub mod config;
pub mod error;
pub mod handler;
pub mod message;
pub mod network;
pub mod procedures;
pub mod routing;
pub mod stabilization;
pub mod storage;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn run(config: Config, bootstrap: Option<SocketAddr>) -> Result<()> {
    println!("Distributed Hash Table based on CHORD");
    println!("-------------------------------------\n");

    log::debug!(
        "The current configuration is as follows.\n\n{:#?}\n",
        &config
    );

    let routing = if let Some(bootstrap_address) = bootstrap {
        println!("Connecting to bootstrap peer {}...", bootstrap_address);

        let bootstrap = Bootstrap::new(config.listen_address, bootstrap_address, config.fingers);
        bootstrap.bootstrap(config.timeout)?
    } else {
        println!("No bootstrapping peer provided, creating new network...");

        let finger_table = vec![config.listen_address; config.fingers];
        Routing::new(
            config.listen_address,
            config.listen_address,
            config.listen_address,
            finger_table,
        )
    };

    let routing = Arc::new(Mutex::new(routing));

    let p2p_handler = P2PHandler::new(Arc::clone(&routing));
    let p2p_server = Server::new(p2p_handler);
    let p2p_handle = p2p_server.listen(config.listen_address, config.worker_threads)?;

    let api_handler = ApiHandler::new(Arc::clone(&routing), config.timeout);
    let api_server = Server::new(api_handler);
    let api_handle = api_server.listen(config.api_address, 1)?;

    let mut stabilization = Stabilization::new(Arc::clone(&routing), config.timeout);
    let stabilization_handle = thread::spawn(move || loop {
        if let Err(err) = stabilization.stabilize() {
            log::error!("Error during stabilization:\n\n{:?}", err);
        }

        thread::sleep(Duration::from_secs(config.stabilization_interval));
    });

    if let Err(err) = p2p_handle.join() {
        log::error!("Error joining p2p handler:\n\n{:?}", err);
    }

    if let Err(err) = api_handle.join() {
        log::error!("Error joining api handler:\n\n{:?}", err);
    }

    if let Err(err) = stabilization_handle.join() {
        log::error!("Error joining stabilization:\n\n{:?}", err);
    }

    Ok(())
}
