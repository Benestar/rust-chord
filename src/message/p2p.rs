use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use routing::identifier::Identifier;
use std::io;
use std::io::prelude::*;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};

/// This message can be sent to a peer which is responsible for the given key
/// to obtain the value for the given key.
///
/// Its ip address has to be known already. The peer looks whether it has stored
/// a value for the given key and returns it in a [`StorageGetSuccess`] message.
///
/// [`StorageGetSuccess`]: struct.StorageGetSuccess.html
#[derive(Debug)]
pub struct StorageGet {
    pub key: [u8; 32]
}

/// To store a message at a specific peer of which the ip address is already
/// known, one can send this message. The peer should answer with a
/// [`StoragePutSuccess`] message if the operation succeeded.
///
/// [`StoragePutSuccess`]: struct.StoragePutSuccess.html
#[derive(Debug)]
pub struct StoragePut {
    pub ttl: u16,
    pub replication: u8,
    pub key: [u8; 32],
    pub value: Vec<u8>
}

/// If after a [`StorageGet`] message the key was found, the peer should reply
/// with the corresponding value attached to this message.
///
/// [`StorageGet`]: struct.StorageGet.html
#[derive(Debug)]
pub struct StorageGetSuccess {
    pub key: [u8; 32],
    pub value: Vec<u8>
}

/// After a successful [`StoragePut`] operation, the peer should reply with this
/// success message.
///
/// The hash of the value should be appended to this message to ensure validity.
/// It is still to be defined which hash function should be used.
///
/// [`StoragePut`]: struct.StoragePut.html
#[derive(Debug)]
pub struct StoragePutSuccess {
    pub key: [u8; 32],
    // TODO objective: fast hash algorithm
    pub value_hash: [u8; 32]
}

/// If a [`StorageGet`] or [`StoragePut`] fails for some reason, this message
/// should be sent back. However, one cannot rely on a failure message being
/// sent back since there can also be timeouts or other issues.
///
/// [`StorageGet`]: struct.StorageGet.html
/// [`StoragePut`]: struct.StoragePut.html
#[derive(Debug)]
pub struct StorageFailure {
    pub key: [u8; 32]
}

/// This message initiates a lookup for a node responsible for the given
/// identifier. The receiving peer is expected to reply with the known peer
/// closest to the requested identifier.
///
/// This can be implemented using finger tables.
#[derive(Debug)]
pub struct PeerFind {
    pub identifier: Identifier
}

/// If, after a [`PeerFind`] operation, a node has been found which is closest
/// to the given identifier, the address of that peer should be included in this
/// message. If the requested peer itself is responsible for the identifier,
/// it should reply with its own address.
///
/// [`PeerFind`]: struct.PeerFind.html
#[derive(Debug)]
pub struct PeerFound {
    pub identifier: Identifier,
    pub socket_addr: SocketAddr
}

/// This message allows to query the predecessor of some other peer.
#[derive(Debug)]
pub struct PredecessorGet;

/// When a peer receives a [`PredecessorGet`] message, it is expected to reply
/// with this message and the address of its predecessor.
///
/// [`PredecessorGet`]: struct.PredecessorGet.html
#[derive(Debug)]
pub struct PredecessorReply {
    pub socket_addr: SocketAddr
}

/// To tell some peer about a new predecessor, this message can be used.
/// The receiving peer is required to check whether it actually should update
/// its predecessor value.
#[derive(Debug)]
pub struct PredecessorSet;

impl StorageGet {
    pub fn parse<T: Read>(reader: &mut T) -> io::Result<Self> {
        let mut key = [0; 32];
        reader.read_exact(&mut key)?;

        Ok(StorageGet { key })
    }

    pub fn write_to<T: Write>(&self, writer: &mut T) -> io::Result<()> {
        writer.write(&self.key)?;

        Ok(())
    }
}

impl StoragePut {
    pub fn parse<T: Read>(reader: &mut T) -> io::Result<Self> {
        let ttl = reader.read_u16::<NetworkEndian>()?;
        let replication = reader.read_u8()?;

        // Skip reserved field
        reader.read_u8()?;

        let mut key = [0; 32];
        reader.read_exact(&mut key)?;

        let mut value = Vec::new();
        reader.read_to_end(&mut value)?;

        Ok(StoragePut { ttl, replication, key, value })
    }

    pub fn write_to<T: Write>(&self, writer: &mut T) -> io::Result<()> {
        writer.write_u16::<NetworkEndian>(self.ttl)?;
        writer.write_u8(self.replication)?;
        writer.write_u8(0)?;
        writer.write(&self.key)?;
        writer.write(&self.value)?;

        Ok(())
    }
}

impl StorageGetSuccess {
    pub fn parse<T: Read>(reader: &mut T) -> io::Result<Self> {
        let mut key = [0; 32];
        reader.read_exact(&mut key)?;

        let mut value = Vec::new();
        reader.read_to_end(&mut value)?;

        Ok(StorageGetSuccess { key, value })
    }

    pub fn write_to<T: Write>(&self, writer: &mut T) -> io::Result<()> {
        writer.write(&self.key)?;
        writer.write(&self.value)?;

        Ok(())
    }
}

impl StoragePutSuccess {
    pub fn parse<T: Read>(reader: &mut T) -> io::Result<Self> {
        let mut key = [0; 32];
        reader.read_exact(&mut key)?;

        let mut value_hash = [0; 32];
        reader.read_exact(&mut value_hash)?;

        Ok(StoragePutSuccess { key, value_hash })
    }

    pub fn write_to<T: Write>(&self, writer: &mut T) -> io::Result<()> {
        writer.write(&self.key)?;
        writer.write(&self.value_hash)?;

        Ok(())
    }
}

impl StorageFailure {
    pub fn parse<T: Read>(reader: &mut T) -> io::Result<Self> {
        let mut key = [0; 32];
        reader.read_exact(&mut key)?;

        Ok(StorageFailure { key })
    }

    pub fn write_to<T: Write>(&self, writer: &mut T) -> io::Result<()> {
        writer.write(&self.key)?;

        Ok(())
    }
}

impl PeerFind {
    pub fn parse<T: Read>(reader: &mut T) -> io::Result<Self> {
        let mut id_arr = [0; 32];
        reader.read_exact(&mut id_arr)?;
        let identifier = Identifier::new(&id_arr);

        Ok(PeerFind{ identifier })
    }

    pub fn write_to<T: Write>(&self, writer: &mut T) -> io::Result<()> {
        writer.write(&self.identifier.as_bytes())?;

        Ok(())
    }
}

impl PeerFound {
    pub fn parse<T: Read>(reader: &mut T) -> io::Result<Self> {
        let mut id_arr = [0; 32];
        reader.read_exact(&mut id_arr)?;
        let identifier = Identifier::new(&id_arr);

        let mut ip_arr = [0; 16];
        reader.read_exact(&mut ip_arr)?;

        let ipv6 = Ipv6Addr::from(ip_arr);

        let ip_address = match ipv6.to_ipv4() {
            Some(ipv4) => IpAddr::V4(ipv4),
            None => IpAddr::V6(ipv6)
        };

        let port = reader.read_u16::<NetworkEndian>()?;

        let socket_addr = SocketAddr::new(ip_address, port);

        Ok(PeerFound{ identifier, socket_addr })
    }

    pub fn write_to<T: Write>(&self, writer: &mut T) -> io::Result<()> {
        writer.write(&self.identifier.as_bytes())?;

        let ip_address = match self.socket_addr.ip() {
            IpAddr::V4(ipv4) => ipv4.to_ipv6_mapped(),
            IpAddr::V6(ipv6) => ipv6
        };

        writer.write(&ip_address.octets())?;
        writer.write_u16::<NetworkEndian>(self.socket_addr.port())?;

        Ok(())
    }
}

impl PredecessorGet {
    pub fn parse<T: Read>(_reader: &mut T) -> io::Result<Self> {
        Ok(PredecessorGet)
    }

    pub fn write_to<T: Write>(&self, _writer: &mut T) -> io::Result<()> {
        Ok(())
    }
}

impl PredecessorReply {
    pub fn parse<T: Read>(reader: &mut T) -> io::Result<Self> {
        let mut ip_arr = [0; 16];
        reader.read_exact(&mut ip_arr)?;

        let ipv6 = Ipv6Addr::from(ip_arr);

        let ip_address = match ipv6.to_ipv4() {
            Some(ipv4) => IpAddr::V4(ipv4),
            None => IpAddr::V6(ipv6)
        };

        let port = reader.read_u16::<NetworkEndian>()?;

        let socket_addr = SocketAddr::new(ip_address, port);

        Ok(PredecessorReply { socket_addr })
    }

    pub fn write_to<T: Write>(&self, writer: &mut T) -> io::Result<()> {
        let ip_address = match self.socket_addr.ip() {
            IpAddr::V4(ipv4) => ipv4.to_ipv6_mapped(),
            IpAddr::V6(ipv6) => ipv6
        };

        writer.write(&ip_address.octets())?;
        writer.write_u16::<NetworkEndian>(self.socket_addr.port())?;

        Ok(())
    }
}

impl PredecessorSet {
    pub fn parse<T: Read>(_reader: &mut T) -> io::Result<Self> {
        Ok(PredecessorSet)
    }

    pub fn write_to<T: Write>(&self, _writer: &mut T) -> io::Result<()> {
        Ok(())
    }
}
