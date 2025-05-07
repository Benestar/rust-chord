use super::MessagePayload;
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use std::io;
use std::io::prelude::*;

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
#[derive(Debug, PartialEq)]
pub struct DhtPut {
    pub ttl: u16,
    pub replication: u8,
    pub key: [u8; 32],
    pub value: Vec<u8>,
}

/// This message is used to ask the DHT method to search for a given key and
/// provide the value if a value for the corresponding is found in the network.
///
/// No immediate is reply is expected after sending this message to the DHT
/// module. The module should however start with its best effort to search for
/// the given key.
#[derive(Debug, PartialEq)]
pub struct DhtGet {
    pub key: [u8; 32],
}

/// This message is sent when a previous [`DhtGet`] operation found a value
/// corresponding to the requested key in the network.
///
/// [`DhtGet`]: struct.DhtGet.html
#[derive(Debug, PartialEq)]
pub struct DhtSuccess {
    pub key: [u8; 32],
    pub value: Vec<u8>,
}

/// This message is sent when a previous [`DhtGet`] operation did not find any
/// value for the requested key.
///
/// [`DhtGet`]: struct.DhtGet.html
#[derive(Debug, PartialEq)]
pub struct DhtFailure {
    pub key: [u8; 32],
}

impl MessagePayload for DhtPut {
    fn parse(reader: &mut dyn Read) -> io::Result<Self> {
        let ttl = reader.read_u16::<NetworkEndian>()?;
        let replication = reader.read_u8()?;

        // Skip reserved field
        reader.read_u8()?;

        let mut key = [0; 32];
        reader.read_exact(&mut key)?;

        let mut value = Vec::new();
        reader.read_to_end(&mut value)?;

        Ok(DhtPut {
            ttl,
            replication,
            key,
            value,
        })
    }

    fn write_to(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_u16::<NetworkEndian>(self.ttl)?;
        writer.write_u8(self.replication)?;
        writer.write_u8(0)?;
        writer.write_all(&self.key)?;
        writer.write_all(&self.value)?;

        Ok(())
    }
}

impl MessagePayload for DhtGet {
    fn parse(reader: &mut dyn Read) -> io::Result<Self> {
        let mut key = [0; 32];
        reader.read_exact(&mut key)?;

        Ok(DhtGet { key })
    }

    fn write_to(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(&self.key)?;

        Ok(())
    }
}

impl MessagePayload for DhtSuccess {
    fn parse(reader: &mut dyn Read) -> io::Result<Self> {
        let mut key = [0; 32];
        reader.read_exact(&mut key)?;

        let mut value = Vec::new();
        reader.read_to_end(&mut value)?;

        Ok(DhtSuccess { key, value })
    }

    fn write_to(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(&self.key)?;
        writer.write_all(&self.value)?;

        Ok(())
    }
}

impl MessagePayload for DhtFailure {
    fn parse(reader: &mut dyn Read) -> io::Result<Self> {
        let mut key = [0; 32];
        reader.read_exact(&mut key)?;

        Ok(DhtFailure { key })
    }

    fn write_to(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_all(&self.key)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::test_message_payload;
    use super::*;

    #[test]
    fn dht_put() {
        #[rustfmt::skip]
        let buf = [
            // TTL, replication and reserved
            0, 12, 4, 0,
            // 32 bytes for key
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            // value
            1, 2, 3, 4, 5,
        ];

        let msg = DhtPut {
            ttl: 12,
            replication: 4,
            key: [3; 32],
            value: vec![1, 2, 3, 4, 5],
        };

        test_message_payload(&buf, msg);
    }

    #[test]
    fn dht_get() {
        #[rustfmt::skip]
        let buf = [
            // 32 bytes for key
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        ];

        let msg = DhtGet { key: [3; 32] };

        test_message_payload(&buf, msg);
    }

    #[test]
    fn dht_success() {
        #[rustfmt::skip]
        let buf = [
            // 32 bytes for key
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            // value
            1, 2, 3, 4, 5,
        ];

        let msg = DhtSuccess {
            key: [3; 32],
            value: vec![1, 2, 3, 4, 5],
        };

        test_message_payload(&buf, msg);
    }

    #[test]
    fn dht_failure() {
        #[rustfmt::skip]
        let buf = [
            // 32 bytes for key
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
            3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        ];

        let msg = DhtFailure { key: [3; 32] };

        test_message_payload(&buf, msg);
    }
}
