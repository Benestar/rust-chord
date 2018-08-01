//! A collection of routines used in various places.

use error::MessageError;
use message::Message;
use message::p2p::{PeerFind, PredecessorGet, PredecessorSet};
use network::Connection;
use routing::identifier::Identifier;
use std::net::SocketAddr;

pub struct Routines {
    timeout: u64
}

impl Routines {
    pub fn new(timeout: u64) -> Self {
        Self { timeout }
    }

    /// Get the socket address of the peer responsible for a given identifier.
    ///
    /// This iteratively sends PEER FIND messages to successive peers,
    /// beginning with `peer_addr` which could be taken from a finger table.
    pub fn find_peer(&self, identifier: Identifier, mut peer_addr: SocketAddr)
                     -> ::Result<SocketAddr>
    {
        // TODO do not fail if one peer does not reply correctly
        loop {
            let mut con = Connection::open(peer_addr, self.timeout)?;
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

    pub fn get_predecessor(&self, peer_addr: SocketAddr)
                           -> ::Result<SocketAddr>
    {
        let mut con = Connection::open(peer_addr, self.timeout)?;

        con.send(&Message::PredecessorGet(PredecessorGet))?;

        let msg = con.receive()?;

        if let Message::PredecessorReply(predecessor_reply) = msg {
            Ok(predecessor_reply.socket_addr)
        } else {
            Err(Box::new(MessageError::new(msg)))
        }
    }

    pub fn set_predecessor(&self, peer_addr: SocketAddr) -> ::Result<()> {
        let mut con = Connection::open(peer_addr, self.timeout)?;

        con.send(&Message::PredecessorSet(PredecessorSet))?;

        Ok(())
    }
}
