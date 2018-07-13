use std::io::Cursor;
use std::io::prelude::*;
use std::io;
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};

/// This message is used to ask the DHT module that the given key-value pair
/// should be stored.
///
/// The field TTL indicates the time in seconds this key-value pair should be
/// stored in the network before it is considered as expired. Note that this is
/// just a hint. The peers may have their own timeouts configured which may be
/// shorter than the value in TTL. In those cases the content could be expired
/// before hand. The DHT does not make any guarantees about the content
/// availability; however it should exercise best effort to store it for that
/// long. Similarly, the replication field indicates how many times (by storing
/// the content on different peers, or under different keys, etc) this content
/// should be replicated. This value should also be treated as a hint; the DHT
/// may choose replicate more or less according to its parameters.
///
/// It is expected that the DHT module upon receiving this message does its best
/// effort in storing the given key-value pair. No confirmation is needed for
/// the PUT operation.
#[derive(Debug)]
pub struct DhtPut {
    pub ttl: u16,
    pub replication: u8,
    pub key: [u8; 32],
    pub value: Vec<u8>
}

/// This message is used to ask the DHT method to search for a given key and
/// provide the value if a value for the corresponding is found in the network.
///
/// No immediate is reply is expected after sending this message to the DHT
/// module. The module should however start with its best effort to search for
/// the given key.
#[derive(Debug)]
pub struct DhtGet {
    pub key: [u8; 32]
}

/// This message is sent when a previous [`DhtGet`] operation found a value
/// corresponding to the requested key in the network.
///
/// [`DhtGet`]: struct.DhtGet.html
#[derive(Debug)]
pub struct DhtSuccess {
    pub key: [u8; 32],
    pub value: Vec<u8>
}

/// This message is sent when a previous [`DhtGet`] operation did not find any
/// value for the requested key.
///
/// [`DhtGet`]: struct.DhtGet.html
#[derive(Debug)]
pub struct DhtFailure {
    pub key: [u8; 32]
}

impl DhtPut {
    pub fn parse(mut cursor: Cursor<&[u8]>) -> io::Result<Self> {
        let ttl = cursor.read_u16::<NetworkEndian>()?;
        let replication = cursor.read_u8()?;

        // Skip reserved field
        cursor.read_u8()?;

        let mut key = [0; 32];
        cursor.read_exact(&mut key)?;

        let mut value = Vec::new();
        cursor.read_to_end(&mut value)?;

        Ok(DhtPut { ttl, replication, key, value })
    }

    pub fn write_bytes(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        buffer.write_u16::<NetworkEndian>(self.ttl)?;
        buffer.write_u8(self.replication)?;
        buffer.write_u8(0)?;
        buffer.write(&self.key)?;
        buffer.write(&self.value)?;

        Ok(())
    }
}



impl DhtGet {
    pub fn parse(mut cursor: Cursor<&[u8]>) -> io::Result<Self> {
        let mut key = [0; 32];
        cursor.read_exact(&mut key)?;

        Ok(DhtGet { key })
    }

    pub fn write_bytes(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        buffer.write(&self.key)?;

        Ok(())
    }
}

impl DhtSuccess {
    pub fn parse(mut cursor: Cursor<&[u8]>) -> io::Result<Self> {
        let mut key = [0; 32];
        cursor.read_exact(&mut key)?;

        let mut value = Vec::new();
        cursor.read_to_end(&mut value)?;

        Ok(DhtSuccess { key, value })
    }

    pub fn write_bytes(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        buffer.write(&self.key)?;
        buffer.write(&self.value)?;

        Ok(())
    }
}

impl DhtFailure {
    pub fn parse(mut cursor: Cursor<&[u8]>) -> io::Result<Self> {
        let mut key = [0; 32];
        cursor.read_exact(&mut key)?;

        Ok(DhtFailure { key })
    }

    pub fn write_bytes(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        buffer.write(&self.key)?;

        Ok(())
    }
}
