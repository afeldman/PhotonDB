//! Custom B-Tree storage implementation
//!
//! This module provides a StorageEngine implementation using the custom
//! B-Tree implementation from src/btree/

use crate::btree::btree::{BTree, BTreeBuilder};
use crate::btree::types::KeyValuePair;
use crate::error::{Error, Result};
use crate::reql::Datum;
use crate::storage::engine::{StorageEngine, TableInfo};
use async_trait::async_trait;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, instrument};

/// Custom B-Tree storage backend
///
/// This storage engine uses our custom on-disk B+Tree implementation
/// for key-value storage. It serializes Datum values to JSON strings.
pub struct BTreeStorage {
    /// The underlying B-Tree (wrapped in Mutex for interior mutability)
    tree: Arc<Mutex<BTree>>,
}

impl BTreeStorage {
    /// Create a new B-Tree storage instance
    ///
    /// # Arguments
    /// * `path` - Path to the B-Tree data file
    /// * `b_param` - B-Tree branching factor (default: 200)
    ///
    /// # Example
    /// ```rust,ignore
    /// let storage = BTreeStorage::new("/tmp/rethinkdb.btree", 100)?;
    /// ```
    pub fn new(path: String, b_param: Option<usize>) -> Result<Self> {
        let path_static: &'static str = Box::leak(path.into_boxed_str());
        let path_buf = Path::new(path_static);
        let b = b_param.unwrap_or(200);

        let tree = BTreeBuilder::new()
            .path(path_buf)
            .b_parameter(b)
            .build()
            .map_err(|e| Error::Storage(format!("Failed to create B-Tree: {:?}", e)))?;

        info!(path = %path_static, b_param = %b, "Initialized B-Tree storage");

        Ok(Self {
            tree: Arc::new(Mutex::new(tree)),
        })
    }

    /// Helper to serialize Datum to JSON string
    fn datum_to_json(datum: &Datum) -> Result<String> {
        serde_json::to_string(datum).map_err(|e| Error::Storage(format!("JSON serialization failed: {}", e)))
    }

    /// Helper to deserialize JSON string to Datum
    fn json_to_datum(json: &str) -> Result<Datum> {
        serde_json::from_str(json).map_err(|e| Error::Storage(format!("JSON deserialization failed: {}", e)))
    }

    /// Convert byte key to string
    fn bytes_to_key(bytes: &[u8]) -> Result<String> {
        String::from_utf8(bytes.to_vec())
            .map_err(|e| Error::Storage(format!("Invalid UTF-8 key: {}", e)))
    }
}

#[async_trait]
impl StorageEngine for BTreeStorage {
    #[instrument(skip(self, key))]
    async fn get(&self, key: &[u8]) -> Result<Option<Datum>> {
        let key_str = Self::bytes_to_key(key)?;
        
        let mut tree = self.tree.lock()
            .map_err(|e| Error::Storage(format!("Lock poisoned: {}", e)))?;

        match tree.search(key_str.clone()) {
            Ok(kv) => {
                debug!(key = %key_str, "Found value in B-Tree");
                let datum = Self::json_to_datum(&kv.value)?;
                Ok(Some(datum))
            }
            Err(crate::btree::error::Error::KeyNotFound) => {
                debug!(key = %key_str, "Key not found in B-Tree");
                Ok(None)
            }
            Err(e) => Err(Error::Storage(format!("B-Tree search failed: {:?}", e))),
        }
    }

    #[instrument(skip(self, key, value))]
    async fn set(&self, key: &[u8], value: Datum) -> Result<()> {
        let key_str = Self::bytes_to_key(key)?;
        let value_str = Self::datum_to_json(&value)?;

        let mut tree = self.tree.lock()
            .map_err(|e| Error::Storage(format!("Lock poisoned: {}", e)))?;

        let kv = KeyValuePair::new(key_str.clone(), value_str);
        
        tree.insert(kv)
            .map_err(|e| Error::Storage(format!("B-Tree insert failed: {:?}", e)))?;

        debug!(key = %key_str, "Inserted value into B-Tree");
        Ok(())
    }

    #[instrument(skip(self, key))]
    async fn delete(&self, key: &[u8]) -> Result<()> {
        let key_str = Self::bytes_to_key(key)?;

        let mut tree = self.tree.lock()
            .map_err(|e| Error::Storage(format!("Lock poisoned: {}", e)))?;

        tree.delete(crate::btree::types::Key(key_str.clone()))
            .map_err(|e| Error::Storage(format!("B-Tree delete failed: {:?}", e)))?;

        debug!(key = %key_str, "Deleted key from B-Tree");
        Ok(())
    }

    async fn list_tables(&self) -> Result<Vec<String>> {
        // B-Tree doesn't have built-in table namespace support
        Ok(Vec::new())
    }

    async fn get_table_info(&self, _table_name: &str) -> Result<Option<TableInfo>> {
        // Not implemented for basic B-Tree storage
        Ok(None)
    }

    async fn list_databases(&self) -> Result<Vec<String>> {
        // Not implemented for basic B-Tree storage
        Ok(Vec::new())
    }

    async fn create_database(&self, _name: &str) -> Result<()> {
        // Not implemented for basic B-Tree storage
        Ok(())
    }

    async fn drop_database(&self, _name: &str) -> Result<()> {
        // Not implemented for basic B-Tree storage
        Ok(())
    }

    async fn list_tables_in_db(&self, _db: &str) -> Result<Vec<String>> {
        // Not implemented for basic B-Tree storage
        Ok(Vec::new())
    }

    async fn create_table(&self, _db: &str, _table: &str, _primary_key: &str) -> Result<()> {
        // Not implemented for basic B-Tree storage
        Ok(())
    }

    async fn drop_table(&self, _db: &str, _table: &str) -> Result<()> {
        // Not implemented for basic B-Tree storage
        Ok(())
    }

    async fn scan_table(&self, _db: &str, _table: &str) -> Result<Vec<Datum>> {
        // Not implemented for basic B-Tree storage
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_btree_storage_basic_ops() -> Result<()> {
        let temp_path = format!("/tmp/photondb_test_{}.btree", std::process::id());
        let storage = BTreeStorage::new(temp_path, Some(10))?;

        // Test set
        let datum = Datum::String("Hello B-Tree!".to_string());
        storage.set(b"test_key", datum.clone()).await?;

        // Test get
        let retrieved = storage.get(b"test_key").await?;
        assert_eq!(retrieved, Some(datum));

        // Test delete
        storage.delete(b"test_key").await?;
        let after_delete = storage.get(b"test_key").await?;
        assert_eq!(after_delete, None);

        Ok(())
    }

    #[tokio::test]
    async fn test_btree_storage_complex_datum() -> Result<()> {
        let temp_path = format!("/tmp/photondb_test_complex_{}.btree", std::process::id());
        let storage = BTreeStorage::new(temp_path, Some(10))?;

        // Test with complex datum
        let mut obj = HashMap::new();
        obj.insert("name".to_string(), Datum::String("Alice".to_string()));
        obj.insert("age".to_string(), Datum::Number(30.0));
        obj.insert("active".to_string(), Datum::Number(1.0)); // Use Number for boolean

        let datum = Datum::Object(obj.clone());
        storage.set(b"user:123", datum.clone()).await?;

        let retrieved = storage.get(b"user:123").await?;
        assert_eq!(retrieved, Some(datum));

        Ok(())
    }
}
