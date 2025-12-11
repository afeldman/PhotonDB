//! Mock storage for testing
//!
//! This module provides a simple in-memory storage implementation
//! for testing purposes.

use crate::error::Result;
use crate::reql::Datum;
use crate::storage::engine::{StorageEngine, TableInfo};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// In-memory mock storage for testing
#[derive(Clone, Default)]
pub struct MockStorage {
    data: Arc<Mutex<HashMap<Vec<u8>, Datum>>>,
}

impl MockStorage {
    /// Create a new mock storage instance
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the number of items stored
    pub fn len(&self) -> usize {
        self.data.lock().unwrap().len()
    }

    /// Check if storage is empty
    pub fn is_empty(&self) -> bool {
        self.data.lock().unwrap().is_empty()
    }

    /// Clear all data
    pub fn clear(&self) {
        self.data.lock().unwrap().clear();
    }
}

#[async_trait]
impl StorageEngine for MockStorage {
    async fn get(&self, key: &[u8]) -> Result<Option<Datum>> {
        Ok(self.data.lock().unwrap().get(key).cloned())
    }

    async fn set(&self, key: &[u8], value: Datum) -> Result<()> {
        self.data.lock().unwrap().insert(key.to_vec(), value);
        Ok(())
    }

    async fn delete(&self, key: &[u8]) -> Result<()> {
        self.data.lock().unwrap().remove(key);
        Ok(())
    }

    async fn list_tables(&self) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    async fn get_table_info(&self, _table_name: &str) -> Result<Option<TableInfo>> {
        Ok(None)
    }

    async fn list_databases(&self) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    async fn create_database(&self, _name: &str) -> Result<()> {
        Ok(())
    }

    async fn drop_database(&self, _name: &str) -> Result<()> {
        Ok(())
    }

    async fn list_tables_in_db(&self, _db: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    async fn create_table(&self, _db: &str, _table: &str, _primary_key: &str) -> Result<()> {
        Ok(())
    }

    async fn drop_table(&self, _db: &str, _table: &str) -> Result<()> {
        Ok(())
    }

    async fn scan_table(&self, _db: &str, _table: &str) -> Result<Vec<Datum>> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap as StdHashMap;

    #[tokio::test]
    async fn test_mock_storage_basic_ops() -> Result<()> {
        let storage = MockStorage::new();

        // Test set
        let datum = Datum::String("test value".to_string());
        storage.set(b"key1", datum.clone()).await?;

        // Test get
        let retrieved = storage.get(b"key1").await?;
        assert_eq!(retrieved, Some(datum));

        // Test delete
        storage.delete(b"key1").await?;
        let after_delete = storage.get(b"key1").await?;
        assert_eq!(after_delete, None);

        Ok(())
    }

    #[tokio::test]
    async fn test_mock_storage_complex_datum() -> Result<()> {
        let storage = MockStorage::new();

        // Test with complex datum
        let mut obj = StdHashMap::new();
        obj.insert("name".to_string(), Datum::String("Alice".to_string()));
        obj.insert("age".to_string(), Datum::Number(30.0));
        obj.insert("scores".to_string(), Datum::Array(vec![
            Datum::Number(95.0),
            Datum::Number(87.0),
            Datum::Number(92.0),
        ]));

        let datum = Datum::Object(obj.clone());
        storage.set(b"user:123", datum.clone()).await?;

        let retrieved = storage.get(b"user:123").await?;
        assert_eq!(retrieved, Some(datum));

        // Test len
        assert_eq!(storage.len(), 1);

        // Test clear
        storage.clear();
        assert!(storage.is_empty());

        Ok(())
    }
}
