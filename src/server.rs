use std::error::Error;
use std::net::TcpListener;

use message::Message;

/// 
///
///
pub struct Server {
    listener: TcpListener
}

impl Server {
    pub fn bind() -> Result<Server, Box<Error>> {
        panic!("")
    }
}
