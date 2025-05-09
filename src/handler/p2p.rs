use crate::error::MessageError;
use crate::message::p2p::*;
use crate::message::Message;
use crate::network::{Connection, ServerHandler};
use crate::routing::identifier::{Identifier, Identify};
use crate::routing::Routing;
use crate::storage::Key;
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

type Storage = HashMap<Key, Vec<u8>>;

/// Handler for peer-to-peer requests
///
/// The supported incoming peer-to-peer messages are `STORAGE GET`,
/// `STORAGE PUT`, `PEER FIND`, `PREDECESSOR GET` and `PREDECESSOR SET`.
pub struct P2PHandler {
    routing: Arc<Mutex<Routing<SocketAddr>>>,
    storage: Mutex<Storage>,
}

impl P2PHandler {
    /// Creates a new `P2PHandler` instance.
    pub fn new(routing: Arc<Mutex<Routing<SocketAddr>>>) -> Self {
        let storage = Mutex::new(Storage::new());

        Self { routing, storage }
    }

    fn responsible_for(&self, identifier: Identifier) -> bool {
        let routing = self.routing.lock().unwrap();

        routing.responsible_for(identifier)
    }

    fn closest_peer(&self, identifier: Identifier) -> SocketAddr {
        let routing = self.routing.lock().unwrap();

        **routing.closest_peer(identifier)
    }

    fn notify_predecessor(&self, predecessor_addr: SocketAddr) -> SocketAddr {
        let mut routing = self.routing.lock().unwrap();

        let old_predecessor_addr = *routing.predecessor;

        // 1. check if the predecessor is closer than the previous predecessor
        if routing.responsible_for(predecessor_addr.identifier()) {
            // 2. update the predecessor if necessary
            routing.set_predecessor(predecessor_addr);

            log::info!("Updated predecessor to new address {}", predecessor_addr);

            // TODO maybe check whether old predecessor is actually still reachable?
            // TODO give data to new predecessor!!!
        }

        if *routing.predecessor == *routing.current {
            // if predecessor points to ourselves, update it to this peer.
            routing.set_predecessor(predecessor_addr);

            log::info!("Updated predecessor to new address {}", predecessor_addr);
        }

        if *routing.successor == *routing.current {
            // If successor points to ourselves, update it to this peer.
            routing.set_successor(predecessor_addr);

            log::info!("Updated successor to new address {}", predecessor_addr);
        }

        old_predecessor_addr
    }

    fn get_from_storage(&self, key: Key) -> Option<Vec<u8>> {
        let storage = self.storage.lock().unwrap();

        storage.get(&key).map(Vec::clone)
    }

    fn put_to_storage(&self, key: Key, value: Vec<u8>) -> bool {
        let mut storage = self.storage.lock().unwrap();

        if storage.contains_key(&key) {
            return false;
        }

        storage.insert(key, value);

        true
    }

    fn handle_storage_get(
        &self,
        mut con: Connection,
        storage_get: StorageGet,
    ) -> crate::Result<()> {
        let raw_key = storage_get.raw_key;
        let replication_index = storage_get.replication_index;

        let key = Key {
            raw_key,
            replication_index,
        };

        log::info!("Received STORAGE GET request for key {}", key);

        // 1. check if given key falls into range
        if self.responsible_for(key.identifier()) {
            // 2. find value for given key
            let value_opt = self.get_from_storage(key);

            let msg = if let Some(value) = value_opt {
                log::info!(
                    "Found value for key {} and replying with STORAGE GET SUCCESS",
                    key
                );

                Message::StorageGetSuccess(StorageGetSuccess { raw_key, value })
            } else {
                log::info!(
                    "Did not find value for key {} and replying with STORAGE FAILURE",
                    key
                );

                Message::StorageFailure(StorageFailure { raw_key })
            };

            // 3. reply with STORAGE GET SUCCESS or STORAGE FAILURE
            con.send(&msg)?
        }

        Ok(())
    }

    fn handle_storage_put(
        &self,
        mut con: Connection,
        storage_put: StoragePut,
    ) -> crate::Result<()> {
        let raw_key = storage_put.raw_key;
        let replication_index = storage_put.replication_index;

        let key = Key {
            raw_key,
            replication_index,
        };

        log::info!("Received STORAGE PUT request for key {}", key);

        // 1. check if given key falls into range
        if self.responsible_for(key.identifier()) {
            // 2. save value for given key
            let msg = if self.put_to_storage(key, storage_put.value) {
                log::info!(
                    "Stored value for key {} and replying with STORAGE PUT SUCCESS",
                    key
                );

                Message::StoragePutSuccess(StoragePutSuccess { raw_key })
            } else {
                log::info!(
                    "Value for key {} already exists, thus replying with STORAGE FAILURE",
                    key
                );

                Message::StorageFailure(StorageFailure { raw_key })
            };

            // 3. reply with STORAGE PUT SUCCESS or STORAGE FAILURE
            con.send(&msg)?;
        }

        Ok(())
    }

    fn handle_peer_find(&self, mut con: Connection, peer_find: PeerFind) -> crate::Result<()> {
        let identifier = peer_find.identifier;

        log::info!("Received PEER FIND request for identifier {}", identifier);

        // 1. check if given key falls into range
        let socket_addr = self.closest_peer(identifier);

        log::info!("Replying with PEER FOUND with address {}", socket_addr);

        // 2. reply with PEER FOUND either with this node or the best next node
        let peer_found = PeerFound {
            identifier,
            socket_addr,
        };
        con.send(&Message::PeerFound(peer_found))?;

        Ok(())
    }

    fn handle_predecessor_notify(
        &self,
        mut con: Connection,
        predecessor_notify: PredecessorNotify,
    ) -> crate::Result<()> {
        let predecessor_addr = predecessor_notify.socket_addr;

        log::info!("Received PREDECESSOR GET request from {}", predecessor_addr);

        let socket_addr = self.notify_predecessor(predecessor_addr);

        log::info!(
            "Replying with PREDECESSOR REPLY and address {}",
            socket_addr
        );

        // 3. return the current predecessor with PREDECESSOR REPLY
        let predecessor_reply = PredecessorReply { socket_addr };
        con.send(&Message::PredecessorReply(predecessor_reply))?;

        Ok(())
    }

    fn handle_connection(&self, mut con: Connection) -> crate::Result<()> {
        let msg = con.receive()?;

        log::info!("P2P handler received message of type {}", msg);

        match msg {
            Message::StorageGet(storage_get) => self.handle_storage_get(con, storage_get),
            Message::StoragePut(storage_put) => self.handle_storage_put(con, storage_put),
            Message::PeerFind(peer_find) => self.handle_peer_find(con, peer_find),
            Message::PredecessorNotify(predecessor_notify) => {
                self.handle_predecessor_notify(con, predecessor_notify)
            }
            _ => Err(Box::new(MessageError::new(msg))),
        }
    }

    fn handle_error(&self, error: &dyn Error) {
        log::error!("Error in P2PHandler: {}", error)
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
