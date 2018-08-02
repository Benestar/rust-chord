use error::MessageError;
use message::Message;
use message::p2p::*;
use network::{Connection, ServerHandler};
use routing::identifier::{Identifier, Identify};
use routing::Routing;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::{Mutex, MutexGuard};
use storage::Key;
use std::sync::Arc;

type Storage = HashMap<Key, Vec<u8>>;

/// Handler for peer-to-peer requests
///
/// The supported incoming peer-to-peer messages are `STORAGE GET`,
/// `STORAGE PUT`, `PEER FIND`, `PREDECESSOR GET` and `PREDECESSOR SET`.
pub struct P2PHandler {
    routing: Arc<Mutex<Routing<SocketAddr>>>,
    storage: Mutex<Storage>
}

impl P2PHandler {
    /// Creates a new `P2PHandler` instance.
    pub fn new(routing: Arc<Mutex<Routing<SocketAddr>>>) -> Self {
        let storage = Mutex::new(Storage::new());

        Self { routing, storage }
    }

    fn lock_routing(&self) -> Result<MutexGuard<Routing<SocketAddr>>, &str> {
        self.routing.lock()
            .or(Err("Could not lock mutex for routing"))
    }

    fn lock_storage(&self) -> Result<MutexGuard<Storage>, &str> {
        self.storage.lock()
            .or(Err("Could not lock mutex for storage"))
    }

    fn responsible_for(&self, identifier: Identifier) -> Result<bool, &str> {
        let routing = self.lock_routing()?;
        Ok(routing.responsible_for(identifier))
    }

    fn get_from_storage(&self, key: Key) -> ::Result<Option<Vec<u8>>> {
        let storage = self.lock_storage()?;
        Ok(storage.get(&key).map(Vec::clone))
    }

    fn put_to_storage(&self, key: Key, value: Vec<u8>) -> ::Result<bool> {
        let mut storage = self.lock_storage()?;

        if storage.contains_key(&key) {
            return Ok(false)
        }

        storage.insert(key, value);

        Ok(true)
    }

    fn handle_storage_get(&self, mut con: Connection, storage_get: StorageGet) -> ::Result<()> {
        let raw_key = storage_get.raw_key;
        let replication_index = storage_get.replication_index;

        let key = Key { raw_key, replication_index };

        info!("Received STORAGE GET request for key {}", key);

        // 1. check if given key falls into range
        if self.responsible_for(key.identifier())? {
            // 2. find value for given key
            let value_opt = self.get_from_storage(key)?;

            let msg = if let Some(value) = value_opt {
                info!("Found value for key {} and replying with STORAGE GET SUCCESS", key);

                Message::StorageGetSuccess(StorageGetSuccess { raw_key, value })
            } else {
                info!("Did not find value for key {} and replying with STORAGE FAILURE", key);

                Message::StorageFailure(StorageFailure { raw_key })
            };

            // 3. reply with STORAGE GET SUCCESS or STORAGE FAILURE
            con.send(&msg)?
        }

        Ok(())
    }

    fn handle_storage_put(&self, mut con: Connection, storage_put: StoragePut) -> ::Result<()> {
        let raw_key = storage_put.raw_key;
        let replication_index = storage_put.replication_index;

        let key = Key { raw_key, replication_index };

        info!("Received STORAGE PUT request for key {}", key);

        // 1. check if given key falls into range
        if self.responsible_for(key.identifier())? {
            // 2. save value for given key
            let msg = if self.put_to_storage(key, storage_put.value)? {
                info!("Stored value for key {} and replying with STORAGE PUT SUCCESS", key);

                Message::StoragePutSuccess(StoragePutSuccess { raw_key })
            } else {
                info!("Value for key {} already exists, thus replying with STORAGE FAILURE", key);

                Message::StorageFailure(StorageFailure { raw_key })
            };

            // 3. reply with STORAGE PUT SUCCESS or STORAGE FAILURE
            con.send(&msg)?;
        }

        Ok(())
    }

    fn handle_peer_find(&self, mut con: Connection, peer_find: PeerFind) -> ::Result<()> {
        let routing = self.lock_routing()?;

        let identifier = peer_find.identifier;

        info!("Received PEER FIND request for identifier {}", identifier);

        // 1. check if given key falls into range
        let socket_addr = if routing.responsible_for(identifier) {
            *routing.current
        } else {
            **routing.closest_peer(identifier)
        };

        info!("Replying with PEER FOUND with address {}", socket_addr);

        // 2. reply with PEER FOUND either with this node or the best next node
        let peer_found = PeerFound { identifier, socket_addr };
        con.send(&Message::PeerFound(peer_found))?;

        Ok(())
    }

    fn handle_predecessor_get(&self, mut con: Connection, _: PredecessorGet) -> ::Result<()> {
        let routing = self.lock_routing()?;

        info!("Received PREDECESSOR GET request");

        let socket_addr = *routing.predecessor;

        info!("Replying with PREDECESSOR REPLY and address {}", socket_addr);

        // 1. return the current predecessor with PREDECESSOR REPLY
        let predecessor_reply = PredecessorReply { socket_addr };
        con.send(&Message::PredecessorReply(predecessor_reply))?;

        Ok(())
    }

    fn handle_predecessor_set(&self, con: Connection, _: PredecessorSet) -> ::Result<()> {
        let mut routing = self.lock_routing()?;

        info!("Received PREDECESSOR SET request");

        let peer_addr = con.peer_addr()?;

        // 1. check if the predecessor is closer than the previous predecessor
        if routing.responsible_for(peer_addr.identifier()) {
            // 2. update the predecessor if necessary
            routing.set_predecessor(peer_addr);

            info!("Updated predecessor to new address {}", peer_addr);
        }

        // TODO maybe check whether predecessor is actually still reachable?

        Ok(())
    }

    fn handle_connection(&self, mut con: Connection) -> ::Result<()> {
        let msg = con.receive()?;

        info!("P2P handler received message of type {}", msg);

        match msg {
            Message::StorageGet(storage_get) =>
                self.handle_storage_get(con, storage_get),
            Message::StoragePut(storage_put) =>
                self.handle_storage_put(con, storage_put),
            Message::PeerFind(peer_find) =>
                self.handle_peer_find(con, peer_find),
            Message::PredecessorGet(predecessor_get) =>
                self.handle_predecessor_get(con, predecessor_get),
            Message::PredecessorSet(predecessor_set) =>
                self.handle_predecessor_set(con, predecessor_set),
            _ =>
                Err(Box::new(MessageError::new(msg)))
        }
    }

    fn handle_error(&self, error: &Error) {
        error!("Error in P2PHandler: {}", error)
    }
}

impl ServerHandler for P2PHandler {
    fn handle_connection(&self, connection: Connection) {
        if let Err(err) = self.handle_connection(connection) {
            self.handle_error(&*err);
        }
    }

    fn handle_error(&self, error: io::Error) {
        self.handle_error(&error)
    }
}
