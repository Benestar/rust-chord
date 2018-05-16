use std::error::Error;
use std::io::prelude::*;
use std::net::{TcpStream, ToSocketAddrs, Shutdown};
use std::time::Duration;

use message::Message;

/// Client to send messages over TCP
///
/// # Examples
///
/// ```
/// let mut client = Client::connect("localhost:8080", 3600);
///
/// let msg = client.receive().expect("could not receive message");
/// client.send(&msg).expect("could not send message");
/// ```
pub struct Client {
    stream: TcpStream,
    buffer: Vec<u8>
}

impl Client {
    pub fn connect<A: ToSocketAddrs>(addrs: A, timeout_ms: u64) -> Result<Client, Box<Error>> {
        let stream = TcpStream::connect(addrs)?;
        let buffer = Vec::with_capacity(64000);

        let timeout = Duration::from_millis(timeout_ms);
        stream.set_read_timeout(Some (timeout))?;
        stream.set_write_timeout(Some (timeout))?;

        Ok (Client { stream, buffer })
    }

    pub fn receive(&mut self) -> Result<Message, Box<Error>> {
        let n = self.stream.read_to_end(&mut self.buffer)?;
        Ok (Message::new(self.buffer.as_slice())?)
    }

    pub fn send(&mut self, msg: &Message) -> Result<(), Box<Error>> {
        let n = msg.write_bytes(&mut self.buffer)?;
        Ok (self.stream.write_all(self.buffer.as_slice())?)
    }

    pub fn shutdown(&mut self) -> Result<(), Box<Error>> {
        Ok (self.stream.shutdown(Shutdown::Both)?)
    }
}
