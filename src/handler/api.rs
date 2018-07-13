use error::MessageError;
use message::api::*;
use message::Message;
use network::{Connection, ServerHandler};
use std::error::Error;
use std::io;

/// Handler for api requests
///
/// The supported incoming api messages are `DHT GET` and `DHT PUT`.
pub struct ApiHandler {

}

impl ApiHandler {
    fn handle_dht_get(&self, mut con: Connection, dht_get: DhtGet)
        -> ::Result<()>
    {
        // 1. find peer for id obtained from key

        // 2. send STORAGE GET message to that peer

        // 3. wait for STORAGE GET SUCCESS response

        // 4. send DHT SUCCESS message to api client
        unimplemented!()
    }

    fn handle_dht_put(&self, mut con: Connection, dht_put: DhtPut)
        -> ::Result<()>
    {
        // 1. find peer for id obtained from key

        // 2. send STORAGE PUT message to that peer
        unimplemented!()
    }

    fn handle_connection(&self, mut con: Connection) -> ::Result<()> {
        let msg = con.receive()?;

        match msg {
            Message::DhtGet(dht_get) =>
                self.handle_dht_get(con, dht_get),
            Message::DhtPut(dht_put) =>
                self.handle_dht_put(con, dht_put),
            _ =>
                Err(Box::new(MessageError::new(msg, "unexpected message type")))
        }
    }

    fn handle_error(&self, error: Box<Error>) {
        eprintln!("Error in ApiHandler: {}", error)
    }
}

impl ServerHandler for ApiHandler {
    fn handle_connection(&self, connection: Connection) {
        if let Err(err) = self.handle_connection(connection) {
            self.handle_error(err);
        }
    }

    fn handle_error(&self, error: io::Error) {
        self.handle_error(Box::new(error))
    }
}
