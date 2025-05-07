//! Implementation of the specified message types, split into api and
//! peer-to-peer messages.
//!
//! The [`Message`] enum combines these messages and provides an abstraction
//! for sending messages over a TCP stream using the [`Connection`] struct.
//!
//! [`Message`]: enum.Message.html
//! [`Connection`]: ../network/struct.Connection.html

use self::api::*;
use self::p2p::*;
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use std::fmt;
use std::io;
use std::io::prelude::*;

pub mod api;
pub mod p2p;

/// This enum contains the different message types supported by this module.
///
/// For each message type, there exists a corresponding struct holding the
/// contents of this message.
///
/// # Api message types
///
/// The following message types are relevant for the api interface:
///
/// * [`DhtPut`](#variant.DhtPut)
/// * [`DhtGet`](#variant.DhtGet)
/// * [`DhtSuccess`](#variant.DhtSuccess)
/// * [`DhtFailure`](#variant.DhtFailure)
///
/// # P2P message types
///
/// The following message types are relevent for the peer-to-peer interface:
///
/// * [`StorageGet`](#variant.StorageGet)
/// * [`StoragePut`](#variant.StoragePut)
/// * [`StorageGetSuccess`](#variant.StorageGetSuccess)
/// * [`StoragePutSuccess`](#variant.StoragePutSuccess)
/// * [`StorageFailure`](#variant.StorageFailure)
/// * [`PeerFind`](#variant.PeerFind)
/// * [`PeerFound`](#variant.PeerFound)
/// * [`PredecessorGet`](#variant.PredecessorGet)
/// * [`PredecessorReply`](#variant.PredecessorReply)
/// * [`PredecessorSet`](#variant.PredecessorSet)
#[derive(Debug, PartialEq)]
pub enum Message {
    /// The given key-value pair should be stored in the network.
    DhtPut(DhtPut),
    /// Search for a given key and provide the value if a value for the
    /// corresponding is found in the network.
    DhtGet(DhtGet),
    /// A previous `DHT GET` operation found a value corresponding to the
    /// requested key in the network.
    DhtSuccess(DhtSuccess),
    /// A previous DHT GET operation did not find any value for the requested
    /// key.
    DhtFailure(DhtFailure),
    /// Obtain the value for the given key if the peer is responsible for.
    StorageGet(StorageGet),
    /// Store a message at a specific peer which is responsible for the key.
    StoragePut(StoragePut),
    /// Reply to a previous `DHT GET` request with the corresponsindg value.
    StorageGetSuccess(StorageGetSuccess),
    /// Reply to a previous `DHT PUT` request with a hash of the stored value.
    StoragePutSuccess(StoragePutSuccess),
    /// An error occured during a previous `DHT GET` or `DHT PUT` message.
    StorageFailure(StorageFailure),
    /// Initiates a lookup for a node responsible for the given identifier.
    PeerFind(PeerFind),
    /// A peer close to the given identifier has been found.
    PeerFound(PeerFound),
    /// Notify some peer about a potentially new predecessor while requesting
    /// its old predecessor.
    PredecessorNotify(PredecessorNotify),
    /// Reply to `PREDECESSOR GET` with the predecessor's address.
    PredecessorReply(PredecessorReply),
}

impl Message {
    const DHT_PUT: u16 = 650;
    const DHT_GET: u16 = 651;
    const DHT_SUCCESS: u16 = 652;
    const DHT_FAILURE: u16 = 653;

    const STORAGE_GET: u16 = 1000;
    const STORAGE_PUT: u16 = 1001;
    const STORAGE_GET_SUCCESS: u16 = 1002;
    const STORAGE_PUT_SUCCESS: u16 = 1003;
    const STORAGE_FAILURE: u16 = 1004;

    const PEER_FIND: u16 = 1050;
    const PEER_FOUND: u16 = 1051;
    const PREDECESSOR_NOTIFY: u16 = 1052;
    const PREDECESSOR_REPLY: u16 = 1053;

    pub fn parse<T: Read>(mut reader: T) -> io::Result<Self> {
        let size = reader.read_u16::<NetworkEndian>()?;
        let msg_type = reader.read_u16::<NetworkEndian>()?;

        if size < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Size must include header",
            ));
        }

        let reader = &mut reader.take(u64::from(size) - 4);

        match msg_type {
            Self::DHT_PUT => {
                // parse DhtPut payload
                MessagePayload::parse(reader).map(Message::DhtPut)
            }
            Self::DHT_GET => {
                // parse DhtGet payload
                MessagePayload::parse(reader).map(Message::DhtGet)
            }
            Self::DHT_SUCCESS => {
                // parse DhtSuccess payload
                MessagePayload::parse(reader).map(Message::DhtSuccess)
            }
            Self::DHT_FAILURE => {
                // parse DhtFailure payload
                MessagePayload::parse(reader).map(Message::DhtFailure)
            }
            Self::STORAGE_GET => {
                // parse StorageGet payload
                MessagePayload::parse(reader).map(Message::StorageGet)
            }
            Self::STORAGE_PUT => {
                // parse StoragePut payload
                MessagePayload::parse(reader).map(Message::StoragePut)
            }
            Self::STORAGE_GET_SUCCESS => {
                // parse StorageGetSuccess payload
                MessagePayload::parse(reader).map(Message::StorageGetSuccess)
            }
            Self::STORAGE_PUT_SUCCESS => {
                // parse StoragePutSuccess payload
                MessagePayload::parse(reader).map(Message::StoragePutSuccess)
            }
            Self::STORAGE_FAILURE => {
                // parse StorageFailure payload
                MessagePayload::parse(reader).map(Message::StorageFailure)
            }
            Self::PEER_FIND => {
                // parse PeerFind payload
                MessagePayload::parse(reader).map(Message::PeerFind)
            }
            Self::PEER_FOUND => {
                // parse PeerFound payload
                MessagePayload::parse(reader).map(Message::PeerFound)
            }
            Self::PREDECESSOR_NOTIFY => {
                // parse PredecessorNotify payload
                MessagePayload::parse(reader).map(Message::PredecessorNotify)
            }
            Self::PREDECESSOR_REPLY => {
                // parse PredecessorReply payload
                MessagePayload::parse(reader).map(Message::PredecessorReply)
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid message type",
            )),
        }
    }

    pub fn write_to<T: Write + Seek>(&self, mut writer: T) -> io::Result<usize> {
        // reserve two bytes for size
        writer.write_u16::<NetworkEndian>(0)?;

        match self {
            Message::DhtPut(dht_put) => {
                writer.write_u16::<NetworkEndian>(Self::DHT_PUT)?;
                dht_put.write_to(&mut writer)?;
            }
            Message::DhtGet(dht_get) => {
                writer.write_u16::<NetworkEndian>(Self::DHT_GET)?;
                dht_get.write_to(&mut writer)?;
            }
            Message::DhtSuccess(dht_success) => {
                writer.write_u16::<NetworkEndian>(Self::DHT_SUCCESS)?;
                dht_success.write_to(&mut writer)?;
            }
            Message::DhtFailure(dht_failure) => {
                writer.write_u16::<NetworkEndian>(Self::DHT_FAILURE)?;
                dht_failure.write_to(&mut writer)?;
            }
            Message::StorageGet(storage_get) => {
                writer.write_u16::<NetworkEndian>(Self::STORAGE_GET)?;
                storage_get.write_to(&mut writer)?;
            }
            Message::StoragePut(storage_put) => {
                writer.write_u16::<NetworkEndian>(Self::STORAGE_PUT)?;
                storage_put.write_to(&mut writer)?;
            }
            Message::StorageGetSuccess(storage_get_success) => {
                writer.write_u16::<NetworkEndian>(Self::STORAGE_GET_SUCCESS)?;
                storage_get_success.write_to(&mut writer)?;
            }
            Message::StoragePutSuccess(storage_put_success) => {
                writer.write_u16::<NetworkEndian>(Self::STORAGE_PUT_SUCCESS)?;
                storage_put_success.write_to(&mut writer)?;
            }
            Message::StorageFailure(storage_failure) => {
                writer.write_u16::<NetworkEndian>(Self::STORAGE_FAILURE)?;
                storage_failure.write_to(&mut writer)?;
            }
            Message::PeerFind(peer_find) => {
                writer.write_u16::<NetworkEndian>(Self::PEER_FIND)?;
                peer_find.write_to(&mut writer)?;
            }
            Message::PeerFound(peer_found) => {
                writer.write_u16::<NetworkEndian>(Self::PEER_FOUND)?;
                peer_found.write_to(&mut writer)?;
            }
            Message::PredecessorNotify(predecessor_get) => {
                writer.write_u16::<NetworkEndian>(Self::PREDECESSOR_NOTIFY)?;
                predecessor_get.write_to(&mut writer)?;
            }
            Message::PredecessorReply(predecessor_reply) => {
                writer.write_u16::<NetworkEndian>(Self::PREDECESSOR_REPLY)?;
                predecessor_reply.write_to(&mut writer)?;
            }
        }

        // write size at beginning of writer
        let size = writer.seek(io::SeekFrom::Current(0))?;

        writer.seek(io::SeekFrom::Start(0))?;
        writer.write_u16::<NetworkEndian>(size as u16)?;

        Ok(size as usize)
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Message::DhtPut(_) => "DHT PUT",
            Message::DhtGet(_) => "DHT GET",
            Message::DhtSuccess(_) => "DHT SUCCESS",
            Message::DhtFailure(_) => "DHT FAILURE",
            Message::StorageGet(_) => "STORAGE GET",
            Message::StoragePut(_) => "STORAGE PUT",
            Message::StorageGetSuccess(_) => "STORAGE GET SUCCESS",
            Message::StoragePutSuccess(_) => "STORAGE PUT SUCCESS",
            Message::StorageFailure(_) => "STORAGE FAILURE",
            Message::PeerFind(_) => "PEER FIND",
            Message::PeerFound(_) => "PEER FOUND",
            Message::PredecessorNotify(_) => "PREDECESSOR GET",
            Message::PredecessorReply(_) => "PREDECESSOR REPLY",
        };

        name.fmt(f)
    }
}

pub trait MessagePayload: Sized {
    fn parse(reader: &mut dyn Read) -> io::Result<Self>;

    fn write_to(&self, writer: &mut dyn Write) -> io::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;
    use std::io::{self, Cursor};

    pub fn test_message_payload<T>(buf: &[u8], msg: T)
    where
        T: MessagePayload + Debug + PartialEq,
    {
        let mut cursor = Cursor::new(&buf[..]);
        let parsed = T::parse(&mut cursor).unwrap();
        assert_eq!(msg, parsed);

        let mut vec = Vec::new();
        msg.write_to(&mut vec).unwrap();
        assert_eq!(&buf[..], &vec[..]);
    }

    #[test]
    fn message_parse() {
        #[rustfmt::skip]
        let buf = [
            // header
            0, 45, 2, 138,
            // TTL, replication and reserved
            0, 12, 4, 0,
            // 32 bytes for key
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            // value
            1, 2, 3, 4, 5,
            // some noise
            6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
        ];

        let msg = Message::DhtPut(DhtPut {
            ttl: 12,
            replication: 4,
            key: [3; 32],
            value: vec![1, 2, 3, 4, 5],
        });

        let parsed = Message::parse(Cursor::new(&buf[..])).unwrap();

        assert_eq!(msg, parsed);
    }

    #[test]
    fn message_parse_empty_buffer() {
        let buf = [];

        let err = Message::parse(Cursor::new(&buf[..])).err().unwrap();

        assert_eq!(io::ErrorKind::UnexpectedEof, err.kind());
    }

    #[test]
    fn message_parse_wrong_size() {
        #[rustfmt::skip]
        let buf = [
            // header
            0, 36, 2, 139,
            // only 16 bytes for key
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        ];

        let err = Message::parse(Cursor::new(&buf[..])).err().unwrap();

        assert_eq!(io::ErrorKind::UnexpectedEof, err.kind());
    }

    #[test]
    fn message_parse_zero_size() {
        #[rustfmt::skip]
        let buf = [
            // header
            0, 0, 4, 28,
        ];

        let err = Message::parse(Cursor::new(&buf[..])).err().unwrap();

        assert_eq!(io::ErrorKind::InvalidInput, err.kind());
        assert_eq!("Size must include header", err.to_string());
    }

    #[test]
    fn message_parse_invalid_message_type() {
        #[rustfmt::skip]
        let buf = [
            // header
            0, 4, 2, 14
        ];

        let err = Message::parse(Cursor::new(&buf[..])).err().unwrap();

        assert_eq!(io::ErrorKind::InvalidInput, err.kind());
        assert_eq!("Invalid message type", err.to_string());
    }

    #[test]
    fn message_write_to() {
        #[rustfmt::skip]
        let buf = [
            // header
            0, 45, 2, 138,
            // TTL, replication and reserved
            0, 12, 4, 0,
            // 32 bytes for key
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            // value
            1, 2, 3, 4, 5,
        ];

        let msg = Message::DhtPut(DhtPut {
            ttl: 12,
            replication: 4,
            key: [3; 32],
            value: vec![1, 2, 3, 4, 5],
        });

        let mut buffer = [0; 64000];
        let size = msg.write_to(Cursor::new(&mut buffer[..])).unwrap();

        assert_eq!(45, size);
        assert_eq!(&buf[..], &buffer[..size]);
    }
}
