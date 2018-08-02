//! A collection of procedures used in various places.

use error::MessageError;
use message::Message;
use message::p2p::{PeerFind, PredecessorGet, PredecessorSet, StorageGet, StoragePut};
use network::Connection;
use routing::identifier::Identifier;
use std::net::SocketAddr;
use storage::Key;

pub struct Procedures {
    timeout: u64
}

impl Procedures {
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
        debug!("Finding peer for identifier {:?}", identifier);

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

    pub fn get_value(&self, peer_addr: SocketAddr, key: Key)
        -> ::Result<Option<Vec<u8>>>
    {
        debug!("Get value for key {} from peer {}", key, peer_addr);

        let storage_get = StorageGet {
            replication_index: key.replication_index,
            raw_key: key.raw_key
        };

        let mut p2p_con = Connection::open(peer_addr, 3600)?;
        p2p_con.send(&Message::StorageGet(storage_get))?;

        let msg = p2p_con.receive()?;

        if let Message::StorageGetSuccess(storage_success) = msg {
            info!("Value for key {} successfully received from peer {}", key, peer_addr);

            Ok(Some(storage_success.value))
        } else {
            warn!("No value found for key {} at peer {}", key, peer_addr);

            Ok(None)
        }
    }

    pub fn put_value(&self, peer_addr: SocketAddr, key: Key, ttl: u16, value: Vec<u8>)
        -> ::Result<()>
    {
        debug!("Put value for key {} to peer {}", key, peer_addr);

        let storage_put = StoragePut {
            ttl,
            replication_index: key.replication_index,
            raw_key: key.raw_key,
            value
        };

        let mut p2p_con = Connection::open(peer_addr, 3600)?;
        p2p_con.send(&Message::StoragePut(storage_put))?;

        let msg = p2p_con.receive()?;

        if let Message::StoragePutSuccess(_) = msg {
            info!("Value for key {} successfully stored at peer {}", key, peer_addr);

            return Ok(());
        }

        if let Message::StorageFailure(_) = msg {
            warn!("Key {} exists already in storage of peer {}", key, peer_addr);

            return Ok(());
        }

        Err(Box::new(MessageError::new(msg)))
    }

    pub fn get_predecessor(&self, peer_addr: SocketAddr) -> ::Result<SocketAddr> {
        debug!("Getting predecessor of peer {}", peer_addr);

        let mut con = Connection::open(peer_addr, self.timeout)?;

        con.send(&Message::PredecessorGet(PredecessorGet))?;

        let msg = con.receive()?;

        if let Message::PredecessorReply(predecessor_reply) = msg {
            info!("Predecessor received from peer {}", peer_addr);

            Ok(predecessor_reply.socket_addr)
        } else {
            warn!("No predecessor received from peer {}", peer_addr);

            Err(Box::new(MessageError::new(msg)))
        }
    }

    pub fn set_predecessor(&self, peer_addr: SocketAddr) -> ::Result<()> {
        debug!("Setting predecessor of peer {}", peer_addr);

        let mut con = Connection::open(peer_addr, self.timeout)?;

        con.send(&Message::PredecessorSet(PredecessorSet))?;

        info!("Predecessor of peer {} set", peer_addr);

        Ok(())
    }
}
