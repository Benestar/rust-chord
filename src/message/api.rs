use std::error::Error;
use std::io::Cursor;
use std::io::prelude::*;
use std::io;
use byteorder::{ReadBytesExt, WriteBytesExt, NetworkEndian};

pub struct DhtPut {
    ttl: u16,
    replication: u8,
    key: [u8; 32],
    value: Vec<u8>
}

pub struct DhtGet {
    key: [u8; 32]
}

pub struct DhtSuccess {
    key: [u8; 32],
    value: Vec<u8>
}

pub struct DhtFailure {
    key: [u8; 32]
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
}

impl DhtGet {
    pub fn parse(mut cursor: Cursor<&[u8]>) -> io::Result<Self> {
        let mut key = [0; 32];
        cursor.read_exact(&mut key);

        Ok(DhtGet { key })
    }
}

impl DhtSuccess {
    pub fn parse(mut cursor: Cursor<&[u8]>) -> io::Result<Self> {
        let mut key = [0; 32];
        cursor.read_exact(&mut key);

        let mut value = Vec::new();
        cursor.read_to_end(&mut value);

        Ok(DhtSuccess { key, value })
    }
}

impl DhtFailure {
    pub fn parse(mut cursor: Cursor<&[u8]>) -> io::Result<Self> {
        let mut key = [0; 32];
        cursor.read_exact(&mut key);

        Ok(DhtFailure { key })
    }
}