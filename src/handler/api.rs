use error::MessageError;
use message::api::*;
use message::Message;
use network::{Connection, ServerHandler};
use routing::identifier::{Identify, Identifier};
use routing::Routing;
use procedures::Procedures;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::{Mutex, MutexGuard};
use std::u8;
use storage::Key;

/// Handler for api requests
///
/// The supported incoming api messages are `DHT GET` and `DHT PUT`.
pub struct ApiHandler {
    routing: Mutex<Routing<SocketAddr>>,
    procedures: Procedures
}

impl ApiHandler {
    /// Creates a new `ApiHandler` instance.
    pub fn new(routing: Routing<SocketAddr>, timeout: u64) -> Self {
        Self {
            routing: Mutex::new(routing),
            procedures: Procedures::new(timeout)
        }
    }

    /// Acquire the lock
    fn lock_routing(&self) -> Result<MutexGuard<Routing<SocketAddr>>, &str> {
        self.routing.lock()
            .or(Err("Could not lock mutex for routing"))
    }

    fn get_closest_peer(&self, identifier: Identifier) -> Result<SocketAddr, &str> {
        let routing = self.lock_routing()?;
        Ok(**routing.closest_peer(identifier))
    }

    fn handle_dht_get(&self, mut api_con: Connection, dht_get: DhtGet)
        -> ::Result<()>
    {
        // find peer for id obtained from key
        let mut replicated_key = Key { raw_key: dht_get.key, replication_index: 0 };

        // iterate through all replication indices
        for i in 0..u8::MAX {
            replicated_key.replication_index = i;

            let identifier = replicated_key.identifier();
            let closest_peer = self.get_closest_peer(identifier)?;
            let target_sock_addr =
                self.procedures.find_peer(identifier, closest_peer)?;

            if let Some(value) = self.procedures.get_value(target_sock_addr, i, dht_get.key)? {
                let dht_success = DhtSuccess { key: dht_get.key, value };
                api_con.send(&Message::DhtSuccess(dht_success))?;

                return Ok(())
            }
        }

        // send failure if no value was found throughout the iteration
        let dht_failure = DhtFailure { key: dht_get.key };
        api_con.send(&Message::DhtFailure(dht_failure))?;

        Ok(())
    }

    fn handle_dht_put(&self, _con: Connection, dht_put: DhtPut)
        -> ::Result<()>
    {
        let target_replication = dht_put.replication;

        for i in 1..target_replication {
            let key = Key { raw_key: dht_put.key, replication_index: i };
            let identifier = key.identifier();
            let closest_peer = self.get_closest_peer(identifier)?;
            let target_sock_addr = self.procedures.find_peer(identifier, closest_peer)?;

            self.procedures.put_value(target_sock_addr, i, dht_put.ttl, dht_put.key, dht_put.value.clone())?;
        }

        Ok(())
    }

    fn handle_connection(&self, mut con: Connection) -> ::Result<()> {
        let msg = con.receive()?;

        match msg {
            Message::DhtGet(dht_get) =>
                self.handle_dht_get(con, dht_get),
            Message::DhtPut(dht_put) =>
                self.handle_dht_put(con, dht_put),
            _ =>
                Err(Box::new(MessageError::new(msg)))
        }
    }

    fn handle_error(&self, error: &Error) {
        eprintln!("Error in ApiHandler: {}", error)
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
