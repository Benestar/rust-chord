use std::io::Cursor;
use std::io::prelude::*;
use std::io;
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};


pub struct DhtPut {
    pub ttl: u16,
    pub replication: u8,
    pub key: [u8; 32],
    pub value: Vec<u8>
}

pub struct DhtGet {
    pub key: [u8; 32]
}

pub struct DhtSuccess {
    pub key: [u8; 32],
    pub value: Vec<u8>
}

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
