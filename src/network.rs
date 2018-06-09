use message::Message;
use std::error::Error;
use std::io;
use std::io::prelude::*;
use std::net::*;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;

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
    pub fn open<A: ToSocketAddrs>(addrs: A, timeout_ms: u64) -> io::Result<Self> {
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

    pub fn receive(&mut self) -> io::Result<Message> {
        self.stream.read_to_end(&mut self.buffer)?;
        Message::new(self.buffer.as_slice())
    }

    pub fn send(&mut self, msg: &Message) -> io::Result<()> {
        msg.write_bytes(&mut self.buffer)?;
        self.stream.write_all(self.buffer.as_slice())
    }

    pub fn shutdown(&mut self) -> io::Result<()> {
        self.stream.shutdown(Shutdown::Both)
    }
}

/// A multithreaded server waiting for connections
///
/// # Examples
///
/// ```
/// let server = Server::new(Box::new(handler));
///
/// server.listen("127.0.0.1:80").expect("could not bind to port");
/// ```
pub struct Server {
    handler: Arc<Box<ServerHandler + Send + Sync>>
}

/// Interface to handle connections or errors from a TcpListener
pub trait ServerHandler {
    fn handle_connection(&self, connection: Connection);

    fn handle_error(&self, error: io::Error);

    fn handle_incoming(&self, result: io::Result<TcpStream>) {
        match result {
            Ok (stream) => {
                let connection = Connection::from_stream(stream);
                self.handle_connection(connection)
            },
            Err (error) => self.handle_error(error)
        }
    }
}

impl Server {
    pub fn new(handler: Box<ServerHandler + Send + Sync>) -> Self {
        Self { handler: Arc::new(handler) }
    }

    pub fn listen<A: ToSocketAddrs>(self, addr: A, num_workers: usize) -> Result<thread::JoinHandle<()>, Box<Error>> {
        let listener = TcpListener::bind(addr)?;

        let handle = thread::spawn(move || {
            let pool = ThreadPool::new(num_workers);

            for result in listener.incoming() {
                let handler = self.handler.clone();
                pool.execute(move || {
                    handler.handle_incoming(result);
                });
            }
        });

        Ok (handle)
    }
}
