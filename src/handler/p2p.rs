use error::MessageError;
use message::Message;
use message::p2p::*;
use network::{Connection, ServerHandler};
use routing::identifier::{Identifier, Identify};
use routing::Routing;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::str;
use std::sync::{Mutex, MutexGuard};
use storage::Storage;

pub struct P2PHandler {
    routing: Mutex<Routing<SocketAddr>>,
    storage: Mutex<Storage>
}

impl P2PHandler {
    pub fn new(routing: Routing<SocketAddr>, storage: Storage) -> Self {
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

    fn responsible_for(&self, identifier: &Identifier) -> Result<bool, &str> {
        let routing = self.lock_routing()?;
        Ok(routing.responsible_for(identifier))
    }

    fn handle_storage_get(&self, mut con: Connection, storage_get: StorageGet) -> ::Result<()> {
        let key = storage_get.key;

        // 1. check if given key falls into range
        if self.responsible_for(&key.get_identifier())? {
            // TODO the critical region is way too large
            let storage = self.lock_storage()?;

            // 2. find value for given key
            // TODO base64 encode the key
            let enc_key = str::from_utf8(&key)?;

            let msg = if storage.contains_key(enc_key) {
                let value = storage.get(enc_key)?;
                Message::StorageGetSuccess(StorageGetSuccess { key, value })
            } else {
                Message::StorageFailure(StorageFailure { key })
            };

            // 3. reply with STORAGE GET SUCCESS or STORAGE FAILURE
            con.send(&msg)?
        }

        Ok(())
    }

    fn handle_storage_put(&self, mut con: Connection, storage_put: StoragePut) -> ::Result<()> {
        let key = storage_put.key;

        // 1. check if given key falls into range
        if self.responsible_for(&key.get_identifier())? {
            // TODO the critical region is way too large
            let mut storage = self.lock_storage()?;

            // 2. save value for given key
            // TODO base64 encode the key
            let enc_key = str::from_utf8(&key)?;

            let msg = if storage.contains_key(enc_key) {
                Message::StorageFailure(StorageFailure { key })
            } else {
                storage.insert(enc_key, &storage_put.value)?;
                let value_hash = [0; 32];
                Message::StoragePutSuccess(StoragePutSuccess { key, value_hash })
            };

            // 3. reply with STORAGE PUT SUCCESS or STORAGE FAILURE
            con.send(&msg)?;
        }

        Ok(())
    }

    fn handle_peer_find(&self, mut con: Connection, peer_find: PeerFind) -> ::Result<()> {
        let routing = self.lock_routing()?;

        let identifier = peer_find.identifier;

        // 1. check if given key falls into range
        let ip_address = if routing.responsible_for(&identifier) {
            routing.get_current_ip().ip()
        } else {
            routing.get_successor().ip()
        };

        // TODO use the finger table to find a node closer to the requested identifier

        // 2. reply with PEER FOUND either with this node or the best next node
        let peer_found = PeerFound { identifier, ip_address };
        con.send(&Message::PeerFound(peer_found))?;

        Ok(())
    }

    fn handle_predecessor_get(&self, mut con: Connection, predecessor_get: PredecessorGet)
        -> ::Result<()>
    {
        let routing = self.lock_routing()?;

        let pred = routing.get_predecessor();

        // 1. return the current predecessor with PREDECESSOR REPLY
        let predecessor_reply = PredecessorReply { ip_address: pred.ip() };
        con.send(&Message::PredecessorReply(predecessor_reply))?;

        Ok(())
    }

    fn handle_predecessor_set(&self, mut con: Connection, predecessor_set: PredecessorSet)
        -> ::Result<()>
    {
        let mut routing = self.lock_routing()?;

        let peer_addr = con.peer_addr()?;

        // 1. check if the predecessor is closer than the previous predecessor
        if routing.is_closer_predecessor(&peer_addr) {
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
                Err(Box::new(MessageError::new(msg, "unexpected message type")))
        }
    }

    fn handle_error(&self, error: Box<Error>) {
        eprintln!("Error in P2PHandler: {}", error)
    }
}

impl ServerHandler for P2PHandler {
    fn handle_connection(&self, connection: Connection) {
        if let Err(err) = self.handle_connection(connection) {
            self.handle_error(err);
        }
    }

    fn handle_error(&self, error: io::Error) {
        self.handle_error(Box::new(error))
    }
}
