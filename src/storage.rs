#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Key {
    pub raw_key: [u8; 32],
    pub replication_index: u8,
}
