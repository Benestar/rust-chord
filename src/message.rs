use std::error::Error;
use std::io::Cursor;

use byteorder::{ReadBytesExt, WriteBytesExt, NetworkEndian};

/// The different message types supported by this module
///
/// For each message type, there exists a corresponding
/// struct holding the contents of this message.
pub enum Message {
    // TODO
}

impl Message {
    pub fn new(buffer: &[u8]) -> Result<Self, Box<Error>> {
        let mut cursor = Cursor::new(&buffer);
        let size = cursor.read_u16::<NetworkEndian>()? as usize;
        let msg_type = cursor.read_u16::<NetworkEndian>()?;

        assert_eq!(buffer.len(), size);

        match msg_type {
            _ => panic!("not implemented")
        }
    }

    pub fn write_bytes(&self, buffer: &mut Vec<u8>) -> Result<(), Box<Error>> {
        panic!("not implemented")
    }
}

