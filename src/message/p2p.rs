use std::io::Cursor;
use std::io::prelude::*;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};


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

    pub fn write_bytes(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        buffer.write(&self.key)?;

        Ok(())
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

    pub fn write_bytes(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        buffer.write_u16::<NetworkEndian>(self.ttl)?;
        buffer.write_u8(self.replication)?;
        buffer.write_u8(0)?;
        buffer.write(&self.key)?;
        buffer.write(&self.value)?;

        Ok(())
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

    pub fn write_bytes(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        buffer.write(&self.key)?;
        buffer.write(&self.value)?;

        Ok(())
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

    pub fn write_bytes(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        buffer.write(&self.key)?;
        buffer.write(&self.value_hash)?;

        Ok(())
    }
}

impl StorageFailure {
    pub fn parse(mut cursor: Cursor<&[u8]>) -> io::Result<Self> {
        let mut key = [0; 32];
        cursor.read_exact(&mut key)?;

        Ok(StorageFailure { key })
    }

    pub fn write_bytes(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        buffer.write(&self.key)?;

        Ok(())
    }
}

impl PeerFind {
    pub fn parse(mut cursor: Cursor<&[u8]>) -> io::Result<Self> {
        let mut identifier = [0; 32];
        cursor.read_exact(&mut identifier)?;

        let mut ip_arr = [0; 16];
        cursor.read_exact(&mut ip_arr)?;

        let ipv6 = Ipv6Addr::from(ip_arr);

        let reply_to = match ipv6.to_ipv4() {
            Some(ipv4) => IpAddr::V4(ipv4),
            None => IpAddr::V6(ipv6)
        };

        Ok(PeerFind{ identifier, reply_to })
    }

    pub fn write_bytes(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        buffer.write(&self.identifier)?;

        let ip_address = match self.reply_to {
            IpAddr::V4(ipv4) => ipv4.to_ipv6_mapped(),
            IpAddr::V6(ipv6) => ipv6
        };

        buffer.write(&ip_address.octets())?;

        Ok(())
    }
}

impl PeerFound {
    pub fn parse(mut cursor: Cursor<&[u8]>) -> io::Result<Self> {
        let mut identifier = [0; 32];
        cursor.read_exact(&mut identifier)?;

        Ok(PeerFound{ identifier })
    }

    pub fn write_bytes(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        buffer.write(&self.identifier);

        Ok(())
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

        let ipv6 = Ipv6Addr::from(ip_arr);

        let ip_address = match ipv6.to_ipv4() {
            Some(ipv4) => IpAddr::V4(ipv4),
            None => IpAddr::V6(ipv6)
        };

        Ok(PredecessorReply { ip_address })
    }

    pub fn write_bytes(&self, buffer: &mut Vec<u8>) -> io::Result<()> {
        let ip_address = match self.ip_address {
            IpAddr::V4(ipv4) => ipv4.to_ipv6_mapped(),
            IpAddr::V6(ipv6) => ipv6
        };

        buffer.write(&ip_address.octets())?;

        Ok(())
    }
}

impl PredecessorSet {
    pub fn parse(cursor: Cursor<&[u8]>) -> io::Result<Self> {
        Ok(PredecessorSet)
    }
}
