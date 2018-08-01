//! A collection of routines used in various places.

use error::MessageError;
use message::Message;
use message::p2p::{PeerFind, PredecessorGet};
use network::Connection;
use routing::identifier::{Identifier, Identify};
use routing::Routing;
use std::net::SocketAddr;

/// Get the socket address of the peer responsible for a given identifier.
///
/// This iteratively sends PEER FIND messages to successive peers,
/// beginning with `peer_addr` which could be taken form a finger table.
pub fn find_peer(identifier: Identifier, mut peer_addr: SocketAddr)
    -> ::Result<SocketAddr>
{
    // TODO do not fail if one peer does not reply correctly
    loop {
        // TODO don't hardcode timeout, put this in setting / config file.
        let mut con = Connection::open(peer_addr, 3600)?;
        let peer_find = PeerFind { identifier };
        con.send(&Message::PeerFind(peer_find))?;
        let msg = con.receive()?;

        let reply_addr = if let Message::PeerFound(peer_found) = msg {
            peer_found.socket_addr
        } else {
            return Err(Box::new(MessageError::new(msg)));
        };

        if reply_addr == peer_addr {
            return Ok(reply_addr);
        }

        peer_addr = reply_addr;
    }
}

/// Struct to create a [`Routing`] table from a bootstrapping peer
///
/// [`Routing`]: ../routing/struct.Routing.html
pub struct Bootstrap {
    own_addr: SocketAddr,
    peer_addr: SocketAddr,
    fingers: usize
}

impl Bootstrap {
    /// Initializes the bootstrap algorithm by providing the peer's own address,
    /// the address of a bootstrapping peer and the number of fingers that
    /// should be stored.
    pub fn new(own_addr: SocketAddr, peer_addr: SocketAddr, fingers: usize)
       -> Self
    {
        Self { own_addr, peer_addr, fingers }
    }

    fn get_successor(&self) -> ::Result<SocketAddr> {
        find_peer(self.own_addr.get_identifier(), self.peer_addr)
    }

    fn get_predecessor(&self, successor: SocketAddr) -> ::Result<SocketAddr> {
        let mut conn = Connection::open(successor, 64000)?;

        conn.send(&Message::PredecessorGet(PredecessorGet))?;

        let msg = conn.receive()?;

        if let Message::PredecessorReply(predecessor_reply) = msg {
            Ok(predecessor_reply.socket_addr)
        } else {
            Err(Box::new(MessageError::new(msg)))
        }
    }

    fn get_finger_table(&self, successor: SocketAddr) -> ::Result<Vec<SocketAddr>> {
        let own_identifier = self.own_addr.get_identifier();

        let mut finger_table = Vec::with_capacity(self.fingers);

        for i in 0..self.fingers {
            // TODO do not hardcode for 256 bits here
            let identifier = own_identifier + Identifier::with_bit(255 - i);
            let peer = find_peer(identifier, successor)?;

            finger_table.push(peer);
        }

        Ok(finger_table)
    }

    /// Run the bootstrapping algorithm and create a new routing table.
    pub fn run(&self) -> ::Result<Routing<SocketAddr>> {
        let successor = self.get_successor()?;
        let predecessor = self.get_predecessor(successor)?;
        let finger_table = self.get_finger_table(successor)?;

        Ok(Routing::new(self.own_addr, predecessor, successor, finger_table))
    }
}
