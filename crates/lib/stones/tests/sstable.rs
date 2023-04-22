#[cfg(test)]
mod sstable_test_suite {
    // tests/sstable_tests.rs

    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
    use std::{
        collections::BTreeMap,
        path::PathBuf,
    };

    use stones::sstable::SSTable;

    #[test]
    fn test_sstable_create_and_read() -> Result<()> {
        let mut data = BTreeMap::new();
        data.insert(b"key1".to_vec(), b"value1".to_vec());
        data.insert(b"key2".to_vec(), b"value2".to_vec());

        let file_path = PathBuf::from("test_sstable_create_and_read.sst");
        let file_path_str = file_path.to_str().expect("Unable to convert path to string");

        let sstable = SSTable::new(file_path_str);
        sstable.create(&data)?;

        let read_data = sstable.read().unwrap();
        assert_eq!(read_data, data);

        Ok(())
    }

    #[test]
    fn test_sstable_delete() -> Result<()> {
        let file_path = PathBuf::from("test_sstable_delete.sst");
        let file_path_str = file_path.to_str().expect("Unable to convert path to string");

        let sstable = SSTable::new(file_path_str);
        assert!(sstable.exists());

        sstable.delete()?;
        assert!(!sstable.exists());

        Ok(())
    }
}
