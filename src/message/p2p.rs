use super::MessagePayload;
use crate::routing::identifier::Identifier;
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
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
#[derive(Debug, PartialEq)]
pub struct StorageGet {
    pub replication_index: u8,
    pub raw_key: [u8; 32],
}

/// To store a message at a specific peer of which the ip address is already
/// known, one can send this message. The peer should answer with a
/// [`StoragePutSuccess`] message if the operation succeeded.
///
/// [`StoragePutSuccess`]: struct.StoragePutSuccess.html
#[derive(Debug, PartialEq)]
pub struct StoragePut {
    pub ttl: u16,
    pub replication_index: u8,
    pub raw_key: [u8; 32],
    pub value: Vec<u8>,
}

/// If after a [`StorageGet`] message the key was found, the peer should reply
/// with the corresponding value attached to this message.
///
/// [`StorageGet`]: struct.StorageGet.html
#[derive(Debug, PartialEq)]
pub struct StorageGetSuccess {
    pub raw_key: [u8; 32],
    pub value: Vec<u8>,
}

/// After a successful [`StoragePut`] operation, the peer should reply with this
/// success message.
///
/// The hash of the value should be appended to this message to ensure validity.
/// It is still to be defined which hash function should be used.
///
/// [`StoragePut`]: struct.StoragePut.html
#[derive(Debug, PartialEq)]
pub struct StoragePutSuccess {
    pub raw_key: [u8; 32],
}

/// If a [`StorageGet`] or [`StoragePut`] fails for some reason, this message
/// should be sent back. However, one cannot rely on a failure message being
/// sent back since there can also be timeouts or other issues.
///
/// [`StorageGet`]: struct.StorageGet.html
/// [`StoragePut`]: struct.StoragePut.html
#[derive(Debug, PartialEq)]
pub struct StorageFailure {
    pub raw_key: [u8; 32],
}

/// This message initiates a lookup for a node responsible for the given
/// identifier. The receiving peer is expected to reply with the known peer
/// closest to the requested identifier.
///
/// This can be implemented using finger tables.
#[derive(Debug, PartialEq)]
pub struct PeerFind {
    pub identifier: Identifier,
}

/// If, after a [`PeerFind`] operation, a node has been found which is closest
/// to the given identifier, the address of that peer should be included in this
/// message. If the requested peer itself is responsible for the identifier,
/// it should reply with its own address.
///
/// [`PeerFind`]: struct.PeerFind.html
#[derive(Debug, PartialEq)]
pub struct PeerFound {
    pub identifier: Identifier,
    pub socket_addr: SocketAddr,
}

/// This message allows to notify some other peer of a potentially new predecessor.
///
/// The receiving peer may use the given address to update its predecessor afterwards if applicable.
#[derive(Debug, PartialEq)]
pub struct PredecessorNotify {
    pub socket_addr: SocketAddr,
}

/// When a peer receives a [`PredecessorGet`] message, it is expected to reply
/// with this message and the address of its predecessor.
///
/// [`PredecessorGet`]: struct.PredecessorGet.html
#[derive(Debug, PartialEq)]
pub struct PredecessorReply {
    pub socket_addr: SocketAddr,
}

impl MessagePayload for StorageGet {
    fn parse(reader: &mut dyn Read) -> io::Result<Self> {
        let replication_index = reader.read_u8()?;

        // Skip reserved fields
        reader.read_u8()?;
        reader.read_u8()?;
        reader.read_u8()?;

        let mut raw_key = [0; 32];
        reader.read_exact(&mut raw_key)?;

        Ok(StorageGet {
            replication_index,
            raw_key,
        })
    }

    fn write_to(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_u8(self.replication_index)?;

        // Fill reserved fields
        writer.write_u8(0)?;
        writer.write_u8(0)?;
        writer.write_u8(0)?;

        writer.write_all(&self.raw_key)?;

        Ok(())
    }
}

impl MessagePayload for StoragePut {
    fn parse(reader: &mut dyn Read) -> io::Result<Self> {
        let ttl = reader.read_u16::<NetworkEndian>()?;
        let replication_index = reader.read_u8()?;

        // Skip reserved field
        reader.read_u8()?;

        let mut raw_key = [0; 32];
        reader.read_exact(&mut raw_key)?;

        let mut value = Vec::new();
        reader.read_to_end(&mut value)?;

        Ok(StoragePut {
            ttl,
            replication_index,
            raw_key,
            value,
        })
    }

    fn write_to(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_u16::<NetworkEndian>(self.ttl)?;
        writer.write_u8(self.replication_index)?;

        // Fill reserved field
        writer.write_u8(0)?;

        writer.write_all(&self.raw_key)?;
        writer.write_all(&self.value)?;

        Ok(())
    }
}

impl MessagePayload for StorageGetSuccess {
    fn parse(reader: &mut dyn Read) -> io::Result<Self> {
        let mut raw_key = [0; 32];
        reader.read_exact(&mut raw_key)?;

        let mut value = Vec::new();
        reader.read_to_end(&mut value)?;

        Ok(StorageGetSuccess { raw_key, value })
    }

    fn write_to(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(&self.raw_key)?;
        writer.write_all(&self.value)?;

        Ok(())
    }
}

impl MessagePayload for StoragePutSuccess {
    fn parse(reader: &mut dyn Read) -> io::Result<Self> {
        let mut raw_key = [0; 32];
        reader.read_exact(&mut raw_key)?;

        Ok(StoragePutSuccess { raw_key })
    }

    fn write_to(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(&self.raw_key)?;

        Ok(())
    }
}

impl MessagePayload for StorageFailure {
    fn parse(reader: &mut dyn Read) -> io::Result<Self> {
        let mut raw_key = [0; 32];
        reader.read_exact(&mut raw_key)?;

        Ok(StorageFailure { raw_key })
    }

    fn write_to(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(&self.raw_key)?;

        Ok(())
    }
}

impl MessagePayload for PeerFind {
    fn parse(reader: &mut dyn Read) -> io::Result<Self> {
        let mut id_arr = [0; 32];
        reader.read_exact(&mut id_arr)?;
        let identifier = Identifier::new(&id_arr);

        Ok(PeerFind { identifier })
    }

    fn write_to(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(&self.identifier.as_bytes())?;

        Ok(())
    }
}

impl MessagePayload for PeerFound {
    fn parse(reader: &mut dyn Read) -> io::Result<Self> {
        let mut id_arr = [0; 32];
        reader.read_exact(&mut id_arr)?;
        let identifier = Identifier::new(&id_arr);

        let mut ip_arr = [0; 16];
        reader.read_exact(&mut ip_arr)?;

        let ipv6 = Ipv6Addr::from(ip_arr);

        let ip_address = match ipv6.to_ipv4() {
            Some(ipv4) => IpAddr::V4(ipv4),
            None => IpAddr::V6(ipv6),
        };

        let port = reader.read_u16::<NetworkEndian>()?;

        let socket_addr = SocketAddr::new(ip_address, port);

        Ok(PeerFound {
            identifier,
            socket_addr,
        })
    }

    fn write_to(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(&self.identifier.as_bytes())?;

        let ip_address = match self.socket_addr.ip() {
            IpAddr::V4(ipv4) => ipv4.to_ipv6_mapped(),
            IpAddr::V6(ipv6) => ipv6,
        };

        writer.write_all(&ip_address.octets())?;
        writer.write_u16::<NetworkEndian>(self.socket_addr.port())?;

        Ok(())
    }
}

impl MessagePayload for PredecessorNotify {
    fn parse(reader: &mut dyn Read) -> io::Result<Self> {
        let mut ip_arr = [0; 16];
        reader.read_exact(&mut ip_arr)?;

        let ipv6 = Ipv6Addr::from(ip_arr);

        let ip_address = match ipv6.to_ipv4() {
            Some(ipv4) => IpAddr::V4(ipv4),
            None => IpAddr::V6(ipv6),
        };

        let port = reader.read_u16::<NetworkEndian>()?;

        let socket_addr = SocketAddr::new(ip_address, port);

        Ok(PredecessorNotify { socket_addr })
    }

    fn write_to(&self, writer: &mut dyn Write) -> io::Result<()> {
        let ip_address = match self.socket_addr.ip() {
            IpAddr::V4(ipv4) => ipv4.to_ipv6_mapped(),
            IpAddr::V6(ipv6) => ipv6,
        };

        writer.write_all(&ip_address.octets())?;
        writer.write_u16::<NetworkEndian>(self.socket_addr.port())?;

        Ok(())
    }
}

impl MessagePayload for PredecessorReply {
    fn parse(reader: &mut dyn Read) -> io::Result<Self> {
        let mut ip_arr = [0; 16];
        reader.read_exact(&mut ip_arr)?;

        let ipv6 = Ipv6Addr::from(ip_arr);

        let ip_address = match ipv6.to_ipv4() {
            Some(ipv4) => IpAddr::V4(ipv4),
            None => IpAddr::V6(ipv6),
        };

        let port = reader.read_u16::<NetworkEndian>()?;

        let socket_addr = SocketAddr::new(ip_address, port);

        Ok(PredecessorReply { socket_addr })
    }

    fn write_to(&self, writer: &mut dyn Write) -> io::Result<()> {
        let ip_address = match self.socket_addr.ip() {
            IpAddr::V4(ipv4) => ipv4.to_ipv6_mapped(),
            IpAddr::V6(ipv6) => ipv6,
        };

        writer.write_all(&ip_address.octets())?;
        writer.write_u16::<NetworkEndian>(self.socket_addr.port())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::test_message_payload;
    use super::*;

    #[test]
    fn storage_get() {
        #[rustfmt::skip]
        let buf = [
            // replication index and reserved
            4, 0, 0, 0,
            // 32 bytes for key
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        ];

        let msg = StorageGet {
            replication_index: 4,
            raw_key: [3; 32],
        };

        test_message_payload(&buf, msg);
    }

    #[test]
    fn storage_put() {
        #[rustfmt::skip]
        let buf = [
            // TTL, replication index and reserved
            0, 12, 4, 0,
            // 32 bytes for key
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            // value
            1, 2, 3, 4, 5
        ];

        let msg = StoragePut {
            ttl: 12,
            replication_index: 4,
            raw_key: [3; 32],
            value: vec![1, 2, 3, 4, 5],
        };

        test_message_payload(&buf, msg);
    }

    #[test]
    fn storage_get_success() {
        #[rustfmt::skip]
        let buf = [
            // 32 bytes for key
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            // value
            1, 2, 3, 4, 5
        ];

        let msg = StorageGetSuccess {
            raw_key: [3; 32],
            value: vec![1, 2, 3, 4, 5],
        };

        test_message_payload(&buf, msg);
    }

    #[test]
    fn storage_put_success() {
        #[rustfmt::skip]
        let buf = [
            // 32 bytes for key
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        ];

        let msg = StoragePutSuccess { raw_key: [3; 32] };

        test_message_payload(&buf, msg);
    }

    #[test]
    fn storage_failure() {
        #[rustfmt::skip]
        let buf = [
            // 32 bytes for key
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        ];

        let msg = StorageFailure { raw_key: [3; 32] };

        test_message_payload(&buf, msg);
    }

    #[test]
    fn peer_find() {
        #[rustfmt::skip]
        let buf = [
            // 32 bytes for identifier
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
        ];

        let msg = PeerFind {
            identifier: Identifier::new(&[5; 32]),
        };

        test_message_payload(&buf, msg);
    }

    #[test]
    fn peer_found_ipv4() {
        #[rustfmt::skip]
        let buf = [
            // 32 bytes for identifier
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            // 16 bytes for ip address
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 127, 0, 0, 1,
            // port
            31, 144,
        ];

        let msg = PeerFound {
            identifier: Identifier::new(&[5; 32]),
            socket_addr: "127.0.0.1:8080".parse().unwrap(),
        };

        test_message_payload(&buf, msg);
    }

    #[test]
    fn peer_found_ipv6() {
        #[rustfmt::skip]
        let buf = [
            // 32 bytes for identifier
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
            // 16 bytes for ip address
            32, 1, 13, 184, 133, 163, 0, 0, 0, 0, 138, 35, 3, 112, 115, 52,
            // port
            31, 144,
        ];

        let msg = PeerFound {
            identifier: Identifier::new(&[5; 32]),
            socket_addr: "[2001:db8:85a3::8a23:370:7334]:8080".parse().unwrap(),
        };

        test_message_payload(&buf, msg);
    }

    #[test]
    fn predecessor_notify_ipv4() {
        #[rustfmt::skip]
        let buf = [
            // 16 bytes for ip address
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 127, 0, 0, 1,
            // port
            31, 144,
        ];

        let msg = PredecessorNotify {
            socket_addr: "127.0.0.1:8080".parse().unwrap(),
        };

        test_message_payload(&buf, msg);
    }

    #[test]
    fn predecessor_notify_ipv6() {
        #[rustfmt::skip]
        let buf = [
            // 16 bytes for ip address
            32, 1, 13, 184, 133, 163, 0, 0, 0, 0, 138, 35, 3, 112, 115, 52,
            // port
            31, 144,
        ];

        let msg = PredecessorNotify {
            socket_addr: "[2001:db8:85a3::8a23:370:7334]:8080".parse().unwrap(),
        };

        test_message_payload(&buf, msg);
    }

    #[test]
    fn predecessor_reply_ipv4() {
        #[rustfmt::skip]
        let buf = [
            // 16 bytes for ip address
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 127, 0, 0, 1,
            // port
            31, 144,
        ];

        let msg = PredecessorReply {
            socket_addr: "127.0.0.1:8080".parse().unwrap(),
        };

        test_message_payload(&buf, msg);
    }

    #[test]
    fn predecessor_reply_ipv6() {
        #[rustfmt::skip]
        let buf = [
            // 16 bytes for ip address
            32, 1, 13, 184, 133, 163, 0, 0, 0, 0, 138, 35, 3, 112, 115, 52,
            // port
            31, 144,
        ];

        let msg = PredecessorReply {
            socket_addr: "[2001:db8:85a3::8a23:370:7334]:8080".parse().unwrap(),
        };

        test_message_payload(&buf, msg);
    }
}
