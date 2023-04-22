// src/sstable.rs

use arrayref::array_ref;
use std::collections::BTreeMap;
use std::fs::{
    self,
    File,
};
use std::io::{
    self,
    Read,
    Write,
};
use std::path::Path;

pub struct SSTable {
    file_path: String,
}

impl SSTable {
    pub fn new(file_path: &str) -> Self {
        SSTable { file_path: file_path.to_owned() }
    }

    pub fn create(&self, data: &BTreeMap<Vec<u8>, Vec<u8>>) -> io::Result<()> {
        let mut file = File::create(&self.file_path)?;
        for (key, value) in data.iter() {
            let key_size = key.len() as u32;
            let value_size = value.len() as u32;
            file.write_all(&key_size.to_le_bytes())?;
            file.write_all(&value_size.to_le_bytes())?;
            file.write_all(key)?;
            file.write_all(value)?;
        }
        Ok(())
    }

    pub fn read(&self) -> io::Result<BTreeMap<Vec<u8>, Vec<u8>>> {
        let mut file = File::open(&self.file_path)?;
        let mut data = BTreeMap::new();

        while file.read_exact(&mut [0; 4]).is_ok() {
            let mut key_size_bytes = [0; 4];
            file.read_exact(&mut key_size_bytes)?;
            let key_size = u32::from_le_bytes(*array_ref!(key_size_bytes, 0, 4));

            let mut value_size_bytes = [0; 4];
            file.read_exact(&mut value_size_bytes)?;
            let value_size = u32::from_le_bytes(*array_ref!(value_size_bytes, 0, 4));

            let mut key = vec![0; key_size as usize];
            let mut value = vec![0; value_size as usize];

            file.read_exact(&mut key)?;
            file.read_exact(&mut value)?;

            data.insert(key, value);
        }

        Ok(data)
    }

    pub fn delete(&self) -> io::Result<()> {
        fs::remove_file(&self.file_path)?;
        Ok(())
    }

    pub fn exists(&self) -> bool {
        Path::new(&self.file_path).exists()
    }
}
