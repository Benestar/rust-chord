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

type Storage = HashMap<Key, Vec<u8>>;

/// Handler for peer-to-peer requests
///
/// The supported incoming peer-to-peer messages are `STORAGE GET`,
/// `STORAGE PUT`, `PEER FIND`, `PREDECESSOR GET` and `PREDECESSOR SET`.
pub struct P2PHandler {
    routing: Mutex<Routing<SocketAddr>>,
    storage: Mutex<Storage>
}

impl P2PHandler {
    /// Creates a new `P2PHandler` instance.
    pub fn new(routing: Routing<SocketAddr>, storage: Storage) -> Self {
        // TODO request mutexes right away
        Self {
            routing: Mutex::new(routing),
            storage: Mutex::new(storage)
        }
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

    fn handle_storage_get(&self, mut con: Connection, storage_get: StorageGet)
        -> ::Result<()>
    {
        let raw_key = storage_get.raw_key;
        let replication_index = storage_get.replication_index;

        let key = Key { raw_key, replication_index };

        // 1. check if given key falls into range
        if self.responsible_for(key.identifier())? {
            // TODO the critical region is way too large
            let storage = self.lock_storage()?;

            // 2. find value for given key
            let value_opt = storage.get(&key).map(Vec::clone);

            let msg = if let Some(value) = value_opt {
                Message::StorageGetSuccess(StorageGetSuccess { raw_key, value })
            } else {
                Message::StorageFailure(StorageFailure { raw_key })
            };

            // 3. reply with STORAGE GET SUCCESS or STORAGE FAILURE
            con.send(&msg)?
        }

        Ok(())
    }

    fn handle_storage_put(&self, mut con: Connection, storage_put: StoragePut)
        -> ::Result<()>
    {
        let raw_key = storage_put.raw_key;
        let replication_index = storage_put.replication_index;

        let key = Key { raw_key, replication_index };

        // 1. check if given key falls into range
        if self.responsible_for(key.identifier())? {
            // TODO the critical region is way too large
            let mut storage = self.lock_storage()?;

            // 2. save value for given key
            let msg = if storage.contains_key(&key) {
                Message::StorageFailure(StorageFailure { raw_key })
            } else {
                storage.insert(key, storage_put.value);
                Message::StoragePutSuccess(StoragePutSuccess { raw_key })
            };

            // 3. reply with STORAGE PUT SUCCESS or STORAGE FAILURE
            con.send(&msg)?;
        }

        Ok(())
    }

    fn handle_peer_find(&self, mut con: Connection, peer_find: PeerFind)
        -> ::Result<()>
    {
        let routing = self.lock_routing()?;

        let identifier = peer_find.identifier;

        // 1. check if given key falls into range
        let socket_addr = if routing.responsible_for(identifier) {
            *routing.current
        } else {
            *routing.successor
        };

        // TODO use the finger table to find a node closer to the requested identifier

        // 2. reply with PEER FOUND either with this node or the best next node
        let peer_found = PeerFound { identifier, socket_addr };
        con.send(&Message::PeerFound(peer_found))?;

        Ok(())
    }

    fn handle_predecessor_get(&self, mut con: Connection, _: PredecessorGet)
        -> ::Result<()>
    {
        let routing = self.lock_routing()?;

        let socket_addr = *routing.predecessor;

        // 1. return the current predecessor with PREDECESSOR REPLY
        let predecessor_reply = PredecessorReply { socket_addr };
        con.send(&Message::PredecessorReply(predecessor_reply))?;

        Ok(())
    }

    fn handle_predecessor_set(&self, con: Connection, _: PredecessorSet)
        -> ::Result<()>
    {
        let mut routing = self.lock_routing()?;

        let peer_addr = con.peer_addr()?;

        // 1. check if the predecessor is closer than the previous predecessor
        if routing.responsible_for(peer_addr.identifier()) {
            // 2. update the predecessor if necessary
            routing.set_predecessor(peer_addr)
        }

        // TODO maybe check whether predecessor is actually still reachable?

        Ok(())
    }

    fn handle_connection(&self, mut con: Connection) -> ::Result<()> {
        let msg = con.receive()?;

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
        eprintln!("Error in P2PHandler: {}", error)
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
