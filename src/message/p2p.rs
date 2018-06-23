use std::io::Cursor;
use std::io::prelude::*;
use std::io;
use std::net::IpAddr;
use byteorder::{ReadBytesExt, NetworkEndian};


pub struct StorageGet {
    pub key: [u8; 32]
}

pub struct StoragePut {
    pub ttl: u16,
    pub replication: u8,
    pub key: [u8; 32],
    pub value: Vec<u8>
}

pub struct StorageGetSuccess {
    pub key: [u8; 32],
    pub value: Vec<u8>
}

pub struct StoragePutSuccess {
    pub key: [u8; 32],
    //todo objective: fast hash algorithm
    pub value_hash: [u8; 32]
}

pub struct StorageFailure {
    pub key: [u8; 32]
}

pub struct PeerFind {
    pub identifier: [u8; 32],
    pub reply_to: IpAddr
}

pub struct PeerFound {
    pub identifier: [u8; 32]
}

pub struct PredecessorGet;

pub struct PredecessorReply {
    pub ip_address: IpAddr
}

pub struct PredecessorSet;

impl StorageGet {
    pub fn parse(mut cursor: Cursor<&[u8]>) -> io::Result<Self> {
        let mut key = [0; 32];
        cursor.read_exact(&mut key)?;

        Ok(StorageGet { key })
    }
}

impl StoragePut {
    pub fn parse(mut cursor: Cursor<&[u8]>) -> io::Result<Self> {
        let ttl = cursor.read_u16::<NetworkEndian>()?;
        let replication = cursor.read_u8()?;

        // Skip reserved field
        cursor.read_u8()?;

        let mut key = [0; 32];
        cursor.read_exact(&mut key)?;

        let mut value = Vec::new();
        cursor.read_to_end(&mut value)?;

        Ok(StoragePut { ttl, replication, key, value })
    }
}

impl StorageGetSuccess {
    pub fn parse(mut cursor: Cursor<&[u8]>) -> io::Result<Self> {
        let mut key = [0; 32];
        cursor.read_exact(&mut key)?;

        let mut value = Vec::new();
        cursor.read_to_end(&mut value)?;

        Ok(StorageGetSuccess { key, value })
    }
}

impl StoragePutSuccess {
    pub fn parse(mut cursor: Cursor<&[u8]>) -> io::Result<Self> {
        let mut key = [0; 32];
        cursor.read_exact(&mut key)?;

        let mut value_hash = [0; 32];
        cursor.read_exact(&mut value_hash)?;

        Ok(StoragePutSuccess { key, value_hash })
    }
}

impl StorageFailure {
    pub fn parse(mut cursor: Cursor<&[u8]>) -> io::Result<Self> {
        let mut key = [0; 32];
        cursor.read_exact(&mut key)?;

        Ok(StorageFailure { key })
    }
}

impl PeerFind {
    pub fn parse(mut cursor: Cursor<&[u8]>) -> io::Result<Self> {
        let mut identifier = [0; 32];
        cursor.read_exact(&mut identifier)?;

        let mut ip_arr = [0; 16];
        cursor.read_exact(&mut ip_arr)?;

        let reply_to = IpAddr::from(ip_arr);

        Ok(PeerFind{ identifier, reply_to })
    }
}

impl PeerFound {
    pub fn parse(mut cursor: Cursor<&[u8]>) -> io::Result<Self> {
        let mut identifier = [0; 32];
        cursor.read_exact(&mut identifier)?;

        Ok(PeerFound{ identifier })
    }
}

impl PredecessorGet {
    pub fn parse(cursor: Cursor<&[u8]>) -> io::Result<Self> {
        Ok(PredecessorGet)
    }
}

impl PredecessorReply {
    pub fn parse(mut cursor: Cursor<&[u8]>) -> io::Result<Self> {
        let mut ip_arr = [0; 16];
        cursor.read_exact(&mut ip_arr)?;

        let ip_address = IpAddr::from(ip_arr);

        Ok(PredecessorReply { ip_address })
    }
}

impl PredecessorSet {
    pub fn parse(cursor: Cursor<&[u8]>) -> io::Result<Self> {
        Ok(PredecessorSet)
    }
}
