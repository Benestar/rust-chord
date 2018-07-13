//! This module contains the interfaces for api and peer-to-peer requests
//!
//! The structs [`ApiHandler`] and [`P2PHandler`] implement the
//! [`ServerHandler`] trait and can be used as handlers for an instance of the
//! [`Server`] struct.
//!
//! [`ApiHandler`]: struct.ApiHandler.html
//! [`P2PHandler`]: struct.P2PHandler.html
//! [`ServerHandler`]: ../network/trait.ServerHandler.html
//! [`Server`]: ../network/struct.Server.html

pub use self::api::ApiHandler;
pub use self::p2p::P2PHandler;

mod api;
mod p2p;
