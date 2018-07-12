//! Networking abstraction layer for TCP connections
//!
//! This module provides some nice abstraction from raw TCP sockets to
//! connections allowing to send and receive [`Message`] objects.
//! Furthermore, it includes parallel handling of incoming connections using
//! a thread pool and the abstraction of handlers.
//!
//! [`Message`]: ../message/enum.Message.html

use message::Message;
use std::io;
use std::io::prelude::*;
use std::net::*;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;

/// A connection between two peers to send Message objects via TCP
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
    /// Opens a TCP connection to a remote peer.
    ///
    /// This uses [`TcpStream::connect`] to create a new TCP socket to the
    /// remote peer with address `addr`.
    ///
    /// `timeout_ms` is the timeout in milliseconds for both read and write
    /// operations. See [`TcpStream::set_read_timeout`] and
    /// [`TcpStream::set_write_timeout`] for further documentation.
    ///
    /// [`TcpStream::connect`]:
    /// ../../std/net/struct.TcpStream.html#method.connect
    /// [`TcpStream::set_read_timeout`]:
    /// ../../std/net/struct.TcpStream.html#method.set_read_timeout
    /// [`TcpStream::set_write_timeout`]:
    /// ../../std/net/struct.TcpStream.html#method.set_write_timeout
    pub fn open<A: ToSocketAddrs>(addr: A, timeout_ms: u64)
        -> io::Result<Self>
    {
        let stream = TcpStream::connect(addr)?;

        let timeout = Duration::from_millis(timeout_ms);
        stream.set_read_timeout(Some (timeout))?;
        stream.set_write_timeout(Some (timeout))?;

        Ok (Self::from_stream(stream))
    }

    fn from_stream(stream: TcpStream) -> Self {
        let buffer = Vec::with_capacity(Message::MAX_LENGTH);
        Self { stream, buffer }
    }

    /// Receives a message from the remote peer.
    ///
    /// This operation is blocking until a message has been received.
    pub fn receive(&mut self) -> io::Result<Message> {
        self.stream.read_to_end(&mut self.buffer)?;
        Message::parse(self.buffer.as_slice())
    }

    /// Sends a message to the remote peer.
    ///
    /// This operation is blocking until the message has been sent.
    pub fn send(&mut self, msg: &Message) -> io::Result<()> {
        msg.write_bytes(&mut self.buffer)?;
        self.stream.write_all(self.buffer.as_slice())
    }

    /// Returns the socket address of the remote peer of this TCP connection.
    ///
    /// See [`TcpStream::peer_addr`] for further documentation.
    ///
    /// [`TcpStream::peer_addr`]:
    /// ../../std/net/struct.TcpStream.html#method.peer_addr
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.stream.peer_addr()
    }

    /// Returns the socket address of the local half of this TCP connection.
    ///
    /// See [`TcpStream::local_addr`] for further documentation.
    ///
    /// [`TcpStream::local_addr`]:
    /// ../../std/net/struct.TcpStream.html#method.local_addr
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.stream.local_addr()
    }

    /// Shuts down the read and write part of this connection.
    ///
    /// See [`TcpStream::shutdown`] for further documentation.
    ///
    /// [`TcpStream::shutdown`]:
    /// ../../std/net/struct.TcpStream.html#method.shutdown
    pub fn shutdown(&mut self) -> io::Result<()> {
        self.stream.shutdown(Shutdown::Both)
    }
}

/// A trait to handle incoming requests from a [`Server`].
///
/// The methods [`handle_connection`] and [`handle_error`] are called based on
/// the success of the incoming request.
///
/// [`Server`]: struct.Server.html
/// [`handle_connection`]: #tymethod.handle_connection
/// [`handle_error`]: #tymethod.handle_error
pub trait ServerHandler {
    /// A connection has been established with some remote peer.
    ///
    /// The handler can exchange messages with the peer via the given
    /// `connection` object.
    fn handle_connection(&self, connection: Connection);

    /// The incoming request was unsuccessful and an error was raised.
    ///
    /// The given `error` should be handled appropiately.
    fn handle_error(&self, error: io::Error);

    /// Handles an incomming connection.
    ///
    /// Depending on the `result` this either calls [`handle_error`] or
    /// creates a new [`Connection`] from the given [`TcpStream`] and
    /// calls [`handle_connection`].
    ///
    /// [`handle_error`]: #tymethod.handle_error
    /// [`Connection`]: struct.Connection.html
    /// [`TcpStream`]: ../../std/net/struct.TcpStream.html
    /// [`handle_connection`]: #tymethod.handle_connection
    fn handle_incoming(&self, result: io::Result<TcpStream>) {
        match result {
            Ok (stream) => {
                // TODO handle time outs
                let connection = Connection::from_stream(stream);
                self.handle_connection(connection)
            },
            Err (error) => self.handle_error(error)
        }
    }
}

/// A multithreaded server waiting for connections
///
/// # Examples
///
/// ```
/// let server = Server::new(Box::new(handler));
///
/// server.listen("127.0.0.1:80", 4)
///     .expect("could not bind to port");
/// ```
pub struct Server {
    handler: Arc<Box<ServerHandler + Send + Sync>>
}

impl Server {
    /// Creates a new server for the given handler.
    ///
    /// The [`ServerHandler`] must also implement [`Send`] and [`Sync`] to
    /// ensure it can be shared between threads.
    ///
    /// [`ServerHandler`]: trait.ServerHandler.html
    /// [`Send`]: ../../std/marker/trait.Send.html
    /// [`Sync`]: ../../std/marker/trait.Sync.html
    pub fn new(handler: Box<ServerHandler + Send + Sync>) -> Self {
        Self { handler: Arc::new(handler) }
    }

    /// Listens on the given socket address.
    ///
    /// `num_workers` defines the number of worker threads which handle
    /// incoming requests in parallel.
    pub fn listen<A: ToSocketAddrs>(self, addr: A, num_workers: usize)
        -> io::Result<thread::JoinHandle<()>>
    {
        let listener = TcpListener::bind(addr)?;

        let handle = thread::spawn(move || {
            let pool = ThreadPool::new(num_workers);

            for result in listener.incoming() {
                let handler = Arc::clone(&self.handler);
                pool.execute(move || {
                    handler.handle_incoming(result);
                });
            }
        });

        Ok (handle)
    }
}
