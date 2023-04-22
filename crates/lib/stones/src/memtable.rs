// src/memtable.rs

use parking_lot::RwLock;
use std::collections::BTreeMap;

pub struct Memtable {
    pub data: RwLock<BTreeMap<Vec<u8>, Vec<u8>>>,
}

impl Memtable {
    pub fn new() -> Self {
        Memtable { data: RwLock::new(BTreeMap::new()) }
    }

    pub fn put(&self, key: &[u8], value: &[u8]) {
        let mut data = self.data.write();
        data.insert(key.to_vec(), value.to_vec());
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let data = self.data.read();
        data.get(key).cloned()
    }

    pub fn delete(&self, key: &[u8]) {
        let mut data = self.data.write();
        data.remove(key);
    }

    pub fn is_full(&self, max_size: usize) -> bool {
        let data = self.data.read();
        data.len() >= max_size
    }
}
