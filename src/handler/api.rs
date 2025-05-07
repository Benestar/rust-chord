use crate::error::MessageError;
use crate::message::api::*;
use crate::message::Message;
use crate::network::{Connection, ServerHandler};
use crate::procedures::Procedures;
use crate::routing::identifier::{Identifier, Identify};
use crate::routing::Routing;
use crate::storage::Key;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::u8;

/// Handler for api requests
///
/// The supported incoming api messages are `DHT GET` and `DHT PUT`.
pub struct ApiHandler {
    routing: Arc<Mutex<Routing<SocketAddr>>>,
    procedures: Procedures,
}

impl ApiHandler {
    /// Creates a new `ApiHandler` instance.
    pub fn new(routing: Arc<Mutex<Routing<SocketAddr>>>, timeout: u64) -> Self {
        let procedures = Procedures::new(timeout);

        Self {
            routing,
            procedures,
        }
    }

    fn closest_peer(&self, identifier: Identifier) -> SocketAddr {
        let routing = self.routing.lock().unwrap();

        **routing.closest_peer(identifier)
    }

    fn find_peer(&self, identifier: Identifier) -> crate::Result<SocketAddr> {
        let closest_peer = self.closest_peer(identifier);

        self.procedures.find_peer(identifier, closest_peer)
    }

    fn handle_dht_get(&self, mut api_con: Connection, dht_get: DhtGet) -> crate::Result<()> {
        // iterate through all replication indices
        for i in 0..u8::MAX {
            let key = Key {
                raw_key: dht_get.key,
                replication_index: i,
            };

            let peer_addr = self.find_peer(key.identifier())?;

            if let Some(value) = self.procedures.get_value(peer_addr, key)? {
                let dht_success = DhtSuccess {
                    key: dht_get.key,
                    value,
                };
                api_con.send(&Message::DhtSuccess(dht_success))?;

                return Ok(());
            }
        }

        // send failure if no value was found throughout the iteration
        let dht_failure = DhtFailure { key: dht_get.key };
        api_con.send(&Message::DhtFailure(dht_failure))?;

        Ok(())
    }

    fn handle_dht_put(&self, _con: Connection, dht_put: DhtPut) -> crate::Result<()> {
        // iterate through all replication indices
        for i in 0..=dht_put.replication {
            let key = Key {
                raw_key: dht_put.key,
                replication_index: i,
            };

            let peer_addr = self.find_peer(key.identifier())?;

            self.procedures
                .put_value(peer_addr, key, dht_put.ttl, dht_put.value.clone())?;
        }

        Ok(())
    }

    fn handle_connection(&self, mut con: Connection) -> crate::Result<()> {
        let msg = con.receive()?;

        info!("Api handler received message of type {}", msg);

        match msg {
            Message::DhtGet(dht_get) => self.handle_dht_get(con, dht_get),
            Message::DhtPut(dht_put) => self.handle_dht_put(con, dht_put),
            _ => Err(Box::new(MessageError::new(msg))),
        }
    }

    fn handle_error(&self, error: &dyn Error) {
        error!("Error in ApiHandler: {}", error)
    }
}

impl ServerHandler for ApiHandler {
    fn handle_connection(&self, connection: Connection) {
        if let Err(err) = self.handle_connection(connection) {
            self.handle_error(&*err);
        }
    }

    fn handle_error(&self, error: io::Error) {
        self.handle_error(&error)
    }
}
