// src/lsmtree.rs

use crate::memtable::Memtable;
use crate::sstable::SSTable;
use std::fs;
use std::io;
use std::sync::Arc;

const MAX_MEMTABLE_SIZE: usize = 1000;

pub struct LsmTree {
    memtable:      Arc<Memtable>,
    sst_directory: String,
}

impl LsmTree {
    pub fn new(sst_directory: &str) -> Self {
        LsmTree {
            memtable:      Arc::new(Memtable::new()),
            sst_directory: sst_directory.to_owned(),
        }
    }

    pub fn put(&self, key: &[u8], value: &[u8]) -> io::Result<()> {
        self.memtable.put(key, value);
        if self.memtable.is_full(MAX_MEMTABLE_SIZE) {
            self.flush()?;
        }
        Ok(())
    }

    pub fn get(&self, key: &[u8]) -> io::Result<Option<Vec<u8>>> {
        if let Some(value) = self.memtable.get(key) {
            return Ok(Some(value));
        }

        for sst_file in self.get_sst_files()? {
            let sstable = SSTable::new(&sst_file);
            if let Some(value) = sstable.read()?.get(key) {
                return Ok(Some(value.clone()));
            }
        }

        Ok(None)
    }

    pub fn delete(&self, key: &[u8]) -> io::Result<()> {
        self.memtable.delete(key);
        Ok(())
    }

    fn flush(&self) -> io::Result<()> {
        let data = self.memtable.data.read().clone();
        let file_name = format!("{}/sst-{}.sst", self.sst_directory, data.len());
        let sstable = SSTable::new(&file_name);
        sstable.create(&data)?;

        self.memtable.data.write().clear();

        Ok(())
    }

    fn get_sst_files(&self) -> io::Result<Vec<String>> {
        let mut sst_files = Vec::new();
        for entry in fs::read_dir(&self.sst_directory)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("sst") {
                sst_files.push(path.to_str().unwrap().to_owned());
            }
        }

        sst_files.sort();
        Ok(sst_files)
    }
}
