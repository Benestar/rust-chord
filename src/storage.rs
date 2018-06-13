use std::collections::HashSet;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::iter::FromIterator;

/// Store key-value pairs in file on the disk
///
/// # Examples
///
/// ```
/// let mut storage = Storage::new("/tmp/dht_storage");
///
/// storage.insert("foo", [42; 1000]);
/// assert!(storage.contains_key("foo"));
///
/// let value = storage.get("foo");
/// assert_eq!(value, [42; 1000]);
/// ```
pub struct Storage {
    path: String,
    keys: HashSet<String>
}

impl Storage {
    pub fn new(path: String) -> io::Result<Self> {
        let keys = HashSet::new();

        // create the relevant directory (if necessary)
        fs::create_dir_all(&path)?;

        Ok (Self { path, keys })
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.keys.contains(key)
    }

    pub fn insert(&mut self, key: &str, value: &[u8]) -> io::Result<()> {
        fs::write(self.file_path(key), value)?;
        self.keys.insert(key.to_owned());

        Ok (())
    }

    pub fn get(&self, key: &str) -> io::Result<Vec<u8>> {
        self.check_key(key)?;

        fs::read(self.file_path(key))
    }

    pub fn remove(&mut self, key: &str) -> io::Result<()> {
        self.check_key(key)?;

        fs::remove_file(self.file_path(key))?;
        self.keys.remove(key);

        Ok (())
    }

    fn check_key(&self, key: &str) -> io::Result<()> {
        if !self.contains_key(key) {
            return Err(io::Error::new(io::ErrorKind::NotFound, "key not found"));
        }

        Ok (())
    }

    fn file_path(&self, key: &str) -> String {
        format!("{}/{}", self.path, key)
    }
}

impl Drop for Storage {
    fn drop(&mut self) {
        // clean up files after dropping this object
        for key in &self.keys {
            fs::remove_file(self.file_path(key))
                .unwrap_or_else(|err| eprintln!("Error deleting file: {}", err))
        }
    }
}
