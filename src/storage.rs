use routing::identifier::{Identifier, Identify};

#[derive(Eq, PartialEq, Hash)]
pub struct Key {
    raw_key: [u8; 32],
    replication_index: u8,
}

impl Key {
    pub fn new(raw_key: [u8; 32], replication_index: u8) -> Self {
        Self { raw_key, replication_index }
    }
}

impl Identify for Key {
    fn get_identifier(&self) -> Identifier {
        let mut bytes = [0; 33];
        bytes[..32].copy_from_slice(&self.raw_key);
        bytes[32] = self.replication_index;

        bytes.get_identifier()
    }
}
