//! Implementation of the specified message types, split into api and
//! peer-to-peer messages.
//!
//! The [`Message`] enum combines these messages and provides an abstraction
//! for sending messages over a TCP stream using the [`Connection`] struct.
//!
//! [`Message`]: enum.Message.html
//! [`Connection`]: ../network/struct.Connection.html

use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
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
#[derive(Debug)]
pub enum Message {
    /// The given key-value pair should be stored in the network.
    DhtPut(api::DhtPut),
    /// Search for a given key and provide the value if a value for the
    /// corresponding is found in the network.
    DhtGet(api::DhtGet),
    /// A previous `DHT GET` operation found a value corresponding to the
    /// requested key in the network.
    DhtSuccess(api::DhtSuccess),
    /// A previous DHT GET operation did not find any value for the requested
    /// key.
    DhtFailure(api::DhtFailure),
    /// Obtain the value for the given key if the peer is responsible for.
    StorageGet(p2p::StorageGet),
    /// Store a message at a specific peer which is responsible for the key.
    StoragePut(p2p::StoragePut),
    /// Reply to a previous `DHT GET` request with the corresponsindg value.
    StorageGetSuccess(p2p::StorageGetSuccess),
    /// Reply to a previous `DHT PUT` request with a hash of the stored value.
    StoragePutSuccess(p2p::StoragePutSuccess),
    /// An error occured during a previous `DHT GET` or `DHT PUT` message.
    StorageFailure(p2p::StorageFailure),
    /// Initiates a lookup for a node responsible for the given identifier.
    PeerFind(p2p::PeerFind),
    /// A peer close to the given identifier has been found.
    PeerFound(p2p::PeerFound),
    /// Query the predecessor of some other peer.
    PredecessorGet(p2p::PredecessorGet),
    /// Reply to `PREDECESSOR GET` with the predecessor's address.
    PredecessorReply(p2p::PredecessorReply),
    /// Tell some peer about a potentially new predecessor.
    PredecessorSet(p2p::PredecessorSet)
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
    const PREDECESSOR_GET: u16 = 1052;
    const PREDECESSOR_REPLY: u16 = 1053;
    const PREDECESSOR_SET: u16 = 1054;

    pub fn parse<T: Read>(reader: &mut T) -> io::Result<Self> {
        let size = reader.read_u16::<NetworkEndian>()?;
        let msg_type = reader.read_u16::<NetworkEndian>()?;

        reader.take(size as u64 - 4);

        let msg = match msg_type {
            Self::DHT_PUT =>
                Message::DhtPut(api::DhtPut::parse(reader)?),
            Self::DHT_GET =>
                Message::DhtGet(api::DhtGet::parse(reader)?),
            Self::DHT_SUCCESS =>
                Message::DhtSuccess(api::DhtSuccess::parse(reader)?),
            Self::DHT_FAILURE =>
                Message::DhtFailure(api::DhtFailure::parse(reader)?),
            Self::STORAGE_GET =>
                Message::StorageGet(p2p::StorageGet::parse(reader)?),
            Self::STORAGE_PUT =>
                Message::StoragePut(p2p::StoragePut::parse(reader)?),
            Self::STORAGE_GET_SUCCESS =>
                Message::StorageGetSuccess(p2p::StorageGetSuccess::parse(reader)?),
            Self::STORAGE_PUT_SUCCESS =>
                Message::StoragePutSuccess(p2p::StoragePutSuccess::parse(reader)?),
            Self::STORAGE_FAILURE =>
                Message::StorageFailure(p2p::StorageFailure::parse(reader)?),
            Self::PEER_FIND =>
                Message::PeerFind(p2p::PeerFind::parse(reader)?),
            Self::PEER_FOUND =>
                Message::PeerFound(p2p::PeerFound::parse(reader)?),
            Self::PREDECESSOR_GET =>
                Message::PredecessorGet(p2p::PredecessorGet::parse(reader)?),
            Self::PREDECESSOR_REPLY =>
                Message::PredecessorReply(p2p::PredecessorReply::parse(reader)?),
            Self::PREDECESSOR_SET =>
                Message::PredecessorSet(p2p::PredecessorSet::parse(reader)?),
            _ =>
                return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                          "Invalid message type"))
        };

        Ok(msg)
    }

    pub fn write_to<T: Write + Seek>(&self, writer: &mut T) -> io::Result<usize> {
        // reserve two bytes for size
        writer.write_u16::<NetworkEndian>(0)?;

        match self {
            Message::DhtPut(dht_put) => {
                writer.write_u16::<NetworkEndian>(Self::DHT_PUT)?;
                dht_put.write_to(writer)?;
            }
            Message::DhtGet(dht_get) => {
                writer.write_u16::<NetworkEndian>(Self::DHT_GET)?;
                dht_get.write_to(writer)?;
            }
            Message::DhtSuccess(dht_success) => {
                writer.write_u16::<NetworkEndian>(Self::DHT_SUCCESS)?;
                dht_success.write_to(writer)?;
            }
            Message::DhtFailure(dht_failure) => {
                writer.write_u16::<NetworkEndian>(Self::DHT_FAILURE)?;
                dht_failure.write_to(writer)?;
            }
            Message::StorageGet(storage_get) => {
                writer.write_u16::<NetworkEndian>(Self::STORAGE_GET)?;
                storage_get.write_to(writer)?;
            }
            Message::StoragePut(storage_put) => {
                writer.write_u16::<NetworkEndian>(Self::STORAGE_PUT)?;
                storage_put.write_to(writer)?;
            }
            Message::StorageGetSuccess(storage_get_success) => {
                writer.write_u16::<NetworkEndian>(Self::STORAGE_GET_SUCCESS)?;
                storage_get_success.write_to(writer)?;
            }
            Message::StoragePutSuccess(storage_put_success) => {
                writer.write_u16::<NetworkEndian>(Self::STORAGE_PUT_SUCCESS)?;
                storage_put_success.write_to(writer)?;
            }
            Message::StorageFailure(storage_failure) => {
                writer.write_u16::<NetworkEndian>(Self::STORAGE_FAILURE)?;
                storage_failure.write_to(writer)?;
            }
            Message::PeerFind(peer_find) => {
                writer.write_u16::<NetworkEndian>(Self::PEER_FIND)?;
                peer_find.write_to(writer)?;
            }
            Message::PeerFound(peer_found) => {
                writer.write_u16::<NetworkEndian>(Self::PEER_FOUND)?;
                peer_found.write_to(writer)?;
            }
            Message::PredecessorGet(predecessor_get) => {
                writer.write_u16::<NetworkEndian>(Self::PREDECESSOR_GET)?;
                predecessor_get.write_to(writer)?;
            }
            Message::PredecessorReply(predecessor_reply) => {
                writer.write_u16::<NetworkEndian>(Self::PREDECESSOR_REPLY)?;
                predecessor_reply.write_to(writer)?;
            }
            Message::PredecessorSet(predecessor_set) => {
                writer.write_u16::<NetworkEndian>(Self::PREDECESSOR_SET)?;
                predecessor_set.write_to(writer)?;
            }
        }

        // write size at beginning of writer
        let size = writer.seek(io::SeekFrom::End(0))?;

        writer.seek(io::SeekFrom::Start(0))?;
        writer.write_u16::<NetworkEndian>(size as u16)?;

        Ok(size as usize)
    }
}
