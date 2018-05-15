use std::error::Error;
use std::io::prelude::*;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

pub enum Message {
}

impl Message {
    pub fn new(buffer: &[u8]) -> Result<Message, Box<Error>> {
        panic!("not implemented")
    }

    pub fn write_bytes(&self, buffer: &mut Vec<u8>) -> Result<Message, Box<Error>> {
        panic!("not implemented")
    }
}

pub struct Api {
    stream: TcpStream,
    buffer: Vec<u8>
}

impl Api {
    pub fn connect<A: ToSocketAddrs>(addr: A, timeout_ms: u64) -> Result<Api, Box<Error>> {
        let stream = TcpStream::connect(addr)?;
        let buffer = Vec::with_capacity(64000);

        let timeout = Duration::from_millis(timeout_ms);
        stream.set_read_timeout(Some (timeout))?;
        stream.set_write_timeout(Some (timeout))?;

        Ok (Api { stream, buffer })
    }

    pub fn receive(&mut self) -> Result<Message, Box<Error>> {
        let n = self.stream.read_to_end(&mut self.buffer)?;
        Ok (Message::new(self.buffer.as_slice())?)
    }

    pub fn send(&mut self, msg: &Message) -> Result<(), Box<Error>> {
        let n = msg.write_bytes(&mut self.buffer)?;
        Ok (self.stream.write_all(self.buffer.as_slice())?)
    }
}
