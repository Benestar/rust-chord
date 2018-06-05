use std::error::Error;
use std::io;
use std::io::prelude::*;
use std::net::*;
use std::thread;
use std::time::Duration;

use message::Message;

/// Send Message objects over a TCP connection
///
/// # Examples
///
/// ```
/// let mut con = Connection::open("localhost:8080", 3600);
///
/// let msg = con.receive().expect("could not receive message");
/// con.send(&msg).expect("could not send message");
/// ```
pub struct Connection {
    stream: TcpStream,
    buffer: Vec<u8>
}

impl Connection {
    pub fn open<A: ToSocketAddrs>(addrs: A, timeout_ms: u64) -> Result<Self, Box<Error>> {
        let stream = TcpStream::connect(addrs)?;

        let timeout = Duration::from_millis(timeout_ms);
        stream.set_read_timeout(Some (timeout))?;
        stream.set_write_timeout(Some (timeout))?;

        Ok (Self::from_stream(stream))
    }

    fn from_stream(stream: TcpStream) -> Self {
        let buffer = Vec::with_capacity(64000);
        Self { stream, buffer }
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

/// A multithreaded server waiting for connections
///
/// # Examples
///
/// ```
/// let server = Server::new(|con| /* ... */);
///
/// server.listen(8080);
/// ```
pub struct Server<T, U> {
    connection_handler: T,
    error_handler: U
}

impl<T, U> Server<T, U>
    where T: Fn(Connection) + Send + Sync, U: Fn(io::Error) + Send + Sync
{
    pub fn new(connection_handler: T, error_handler: U) -> Self {
        Self { connection_handler, error_handler }
    }

    pub fn listen(&mut self, port: u16) -> Result<thread::JoinHandle<()>, Box<Error>> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = TcpListener::bind(addr)?;

        let handle = thread::spawn(move || {
            for result in listener.incoming() {
                thread::spawn(|| {
                    self.handle_incoming(result);
                });
            }
        });

        Ok (handle)
    }

    fn handle_incoming(&self, result: io::Result<TcpStream>) {
        match result {
            Ok (stream) => {
                let connection = Connection::from_stream(stream);
                // TODO handle connection
            },
            Err (e) => (self.error_handler)(e)
        }
    }
}
