use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Key {
    pub raw_key: [u8; 32],
    pub replication_index: u8,
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut iter = self.raw_key.iter();

        write!(f, "[{}", iter.next().unwrap())?;

        for byte in iter {
            write!(f, ":{}", byte)?;
        }

        write!(f, "]:{}", self.replication_index)
    }
}
