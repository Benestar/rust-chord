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

extern crate base64;
extern crate bigint;
extern crate byteorder;
extern crate ring;
extern crate threadpool;

use std::error::Error;

pub mod error;
pub mod handler;
pub mod message;
pub mod network;
pub mod routing;
pub mod storage;

type Result<T> = std::result::Result<T, Box<Error>>;
