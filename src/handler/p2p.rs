use error::MessageError;
use message::Message;
use message::p2p::*;
use network::{Connection, ServerHandler};
use routing::Routing;
use std::error::Error;
use std::io;
use std::sync::Mutex;
use std::net::SocketAddr;

pub struct P2PHandler {
    routing: Mutex<Routing<SocketAddr>>
}

impl P2PHandler {
    pub fn new(routing: Routing<SocketAddr>) -> Self {
        Self { routing: Mutex::new(routing) }
    }

    fn handle_storage_get(&self, mut con: Connection, storage_get: StorageGet) -> ::Result<()> {
        // 1. check if given key falls into range

        // 2. find value for given key

        // 3. reply with STORAGE GET SUCCESS or STORAGE FAILURE
        unimplemented!()
    }

    fn handle_storage_put(&self, mut con: Connection, storage_put: StoragePut) -> ::Result<()> {
        // 1. check if given key falls into range

        // 2. save value for given key

        // 3. reply with STORAGE PUT SUCCESS or STORAGE FAILURE
        unimplemented!()
    }

    fn handle_peer_find(&self, mut con: Connection, peer_find: PeerFind) -> ::Result<()> {
        // 1. check if given key falls into range

        // 2. reply with PEER FOUND either with this node or the best next node
        unimplemented!()
    }

    fn handle_predecessor_get(&self, mut con: Connection, predecessor_get: PredecessorGet)
        -> ::Result<()>
    {
        // 1. return the current predecessor with PREDECESSOR REPLY
        let mut routing = self.routing.lock()
            .or(Err("could not lock mutex"))?;

        let pred = routing.get_predecessor();

        let predecessor_reply = PredecessorReply { ip_address: pred.ip() };
        con.send(&Message::PredecessorReply(predecessor_reply))?;

        Ok(())
    }

    fn handle_predecessor_set(&self, mut con: Connection, predecessor_set: PredecessorSet)
        -> ::Result<()>
    {
        let peer_addr = con.peer_addr()?;

        let mut routing = self.routing.lock()
            .or(Err("could not lock mutex"))?;

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
