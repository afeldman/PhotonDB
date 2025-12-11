//! StorageEngine trait implementation for SlabStorage

use super::storage::SlabStorage as InnerSlabStorage;
use crate::error::{Error, Result};
use crate::reql::Datum;
use crate::storage::engine::{StorageEngine, TableInfo};
use async_trait::async_trait;
use std::path::Path;
use tracing::{debug, warn};

/// Slab storage engine that implements StorageEngine trait
///
/// This is a wrapper around the core SlabStorage that provides
/// async compatibility and Datum serialization.
pub struct SlabStorageEngine {
    inner: InnerSlabStorage,
}

impl SlabStorageEngine {
    /// Create a new slab storage engine
    pub fn new<P: AsRef<Path>>(
        base_path: P,
        min_slot_size: Option<usize>,
        max_slot_size: Option<usize>,
    ) -> Result<Self> {
        let inner = InnerSlabStorage::new(base_path, min_slot_size, max_slot_size)?;
        Ok(Self { inner })
    }

    /// Create with default settings (64B - 64KB)
    pub fn with_defaults<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        Self::new(base_path, None, None)
    }

    /// Serialize Datum to bytes
    fn datum_to_bytes(datum: &Datum) -> Result<Vec<u8>> {
        serde_json::to_vec(datum)
            .map_err(|e| Error::Storage(format!("Failed to serialize Datum: {}", e)))
    }

    /// Deserialize bytes to Datum
    fn bytes_to_datum(bytes: &[u8]) -> Result<Datum> {
        serde_json::from_slice(bytes)
            .map_err(|e| Error::Storage(format!("Failed to deserialize Datum: {}", e)))
    }
}

#[async_trait]
impl StorageEngine for SlabStorageEngine {
    async fn get(&self, key: &[u8]) -> Result<Option<Datum>> {
        match self.inner.get(key)? {
            Some(bytes) => {
                let datum = Self::bytes_to_datum(&bytes)?;
                Ok(Some(datum))
            }
            None => Ok(None),
        }
    }

    async fn set(&self, key: &[u8], value: Datum) -> Result<()> {
        let bytes = Self::datum_to_bytes(&value)?;
        self.inner.set(key, &bytes)?;
        debug!(key_len = key.len(), value_len = bytes.len(), "Set key-value");
        Ok(())
    }

    async fn delete(&self, key: &[u8]) -> Result<()> {
        self.inner.delete(key)?;
        Ok(())
    }

    async fn list_tables(&self) -> Result<Vec<String>> {
        // Scan for keys with prefix "__meta__:tables:"
        let prefix = b"__meta__:tables:";
        let keys = self.inner.keys();
        
        let tables: Vec<String> = keys
            .into_iter()
            .filter(|k| k.starts_with(prefix))
            .filter_map(|k| {
                String::from_utf8(k[prefix.len()..].to_vec()).ok()
            })
            .collect();
        
        Ok(tables)
    }

    async fn get_table_info(&self, table_name: &str) -> Result<Option<TableInfo>> {
        let key = format!("__meta__:tables:{}", table_name);
        match self.get(key.as_bytes()).await? {
            Some(datum) => {
                // Manual extraction to handle type conversions
                if let Datum::Object(ref obj) = datum {
                    let name = obj.get("name")
                        .and_then(|d| d.as_string())
                        .ok_or_else(|| Error::Storage("Missing 'name' field".to_string()))?
                        .to_string();
                    
                    let db = obj.get("db")
                        .and_then(|d| d.as_string())
                        .ok_or_else(|| Error::Storage("Missing 'db' field".to_string()))?
                        .to_string();
                    
                    let primary_key = obj.get("primary_key")
                        .and_then(|d| d.as_string())
                        .ok_or_else(|| Error::Storage("Missing 'primary_key' field".to_string()))?
                        .to_string();
                    
                    let doc_count = obj.get("doc_count")
                        .and_then(|d| d.as_number())
                        .unwrap_or(0.0) as u64;
                    
                    let indexes = obj.get("indexes")
                        .and_then(|d| d.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|d| d.as_string().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();
                    
                    let info = TableInfo {
                        name,
                        db,
                        primary_key,
                        doc_count,
                        indexes,
                    };
                    
                    Ok(Some(info))
                } else {
                    Err(Error::Storage("Table info is not an object".to_string()))
                }
            }
            None => Ok(None),
        }
    }

    async fn list_databases(&self) -> Result<Vec<String>> {
        let prefix = b"__meta__:databases:";
        let keys = self.inner.keys();
        
        let dbs: Vec<String> = keys
            .into_iter()
            .filter(|k| k.starts_with(prefix))
            .filter_map(|k| {
                String::from_utf8(k[prefix.len()..].to_vec()).ok()
            })
            .collect();
        
        Ok(dbs)
    }

    async fn create_database(&self, name: &str) -> Result<()> {
        let key = format!("__meta__:databases:{}", name);
        let value = Datum::Object(vec![(
            "name".to_string(),
            Datum::String(name.to_string()),
        )].into_iter().collect());
        
        self.set(key.as_bytes(), value).await?;
        debug!(db = name, "Created database");
        Ok(())
    }

    async fn drop_database(&self, name: &str) -> Result<()> {
        // Delete database metadata
        let key = format!("__meta__:databases:{}", name);
        self.delete(key.as_bytes()).await?;
        
        // Delete all tables in this database
        let tables = self.list_tables_in_db(name).await?;
        for table in tables {
            self.drop_table(name, &table).await?;
        }
        
        debug!(db = name, "Dropped database");
        Ok(())
    }

    async fn list_tables_in_db(&self, db: &str) -> Result<Vec<String>> {
        let prefix_str = format!("__meta__:tables:{}.", db);
        let prefix = prefix_str.as_bytes();
        let keys = self.inner.keys();
        
        let tables: Vec<String> = keys
            .into_iter()
            .filter(|k| k.starts_with(prefix))
            .filter_map(|k| {
                // Extract "table" from "__meta__:tables:db.table"
                String::from_utf8(k.clone()).ok().and_then(|full_key| {
                    full_key
                        .strip_prefix(&prefix_str)
                        .map(|table_name| table_name.to_string())
                })
            })
            .collect();
        
        Ok(tables)
    }

    async fn create_table(&self, db: &str, table: &str, primary_key: &str) -> Result<()> {
        let key = format!("__meta__:tables:{}.{}", db, table);
        
        // Direct serialization to Datum (avoid double serialization)
        let datum = Datum::Object(vec![
            ("name".to_string(), Datum::String(table.to_string())),
            ("db".to_string(), Datum::String(db.to_string())),
            ("primary_key".to_string(), Datum::String(primary_key.to_string())),
            ("doc_count".to_string(), Datum::Number(0.0)),
            ("indexes".to_string(), Datum::Array(vec![])),
        ].into_iter().collect());
        
        self.set(key.as_bytes(), datum).await?;
        debug!(db, table, "Created table");
        Ok(())
    }

    async fn drop_table(&self, db: &str, table: &str) -> Result<()> {
        // Delete table metadata
        let key = format!("__meta__:tables:{}.{}", db, table);
        self.delete(key.as_bytes()).await?;
        
        // Delete all documents (would need to scan with prefix)
        warn!(db, table, "drop_table: document deletion not yet implemented");
        
        debug!(db, table, "Dropped table");
        Ok(())
    }

    async fn scan_table(&self, db: &str, table: &str) -> Result<Vec<Datum>> {
        let prefix = format!("doc:{}:{}:", db, table);
        let keys = self.inner.keys();
        
        let mut docs = Vec::new();
        for key in keys {
            if String::from_utf8_lossy(&key).starts_with(&prefix) {
                if let Some(datum) = self.get(&key).await? {
                    docs.push(datum);
                }
            }
        }
        
        Ok(docs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reql::Datum;

    #[tokio::test]
    async fn test_slab_engine_basic() -> Result<()> {
        let temp_dir = std::env::temp_dir().join(format!("slab_engine_{}", std::process::id()));
        let engine = SlabStorageEngine::with_defaults(&temp_dir)?;

        // Set and get
        engine.set(b"key1", Datum::Number(42.0)).await?;
        let value = engine.get(b"key1").await?;
        assert_eq!(value, Some(Datum::Number(42.0)));

        // Delete
        engine.delete(b"key1").await?;
        assert_eq!(engine.get(b"key1").await?, None);

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
        Ok(())
    }

    #[tokio::test]
    async fn test_slab_engine_databases() -> Result<()> {
        let temp_dir =
            std::env::temp_dir().join(format!("slab_engine_dbs_{}", std::process::id()));
        let engine = SlabStorageEngine::with_defaults(&temp_dir)?;

        // Create databases
        engine.create_database("db1").await?;
        engine.create_database("db2").await?;

        // List databases
        let dbs = engine.list_databases().await?;
        assert_eq!(dbs.len(), 2);
        assert!(dbs.contains(&"db1".to_string()));
        assert!(dbs.contains(&"db2".to_string()));

        // Drop database
        engine.drop_database("db1").await?;
        let dbs = engine.list_databases().await?;
        assert_eq!(dbs.len(), 1);

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
        Ok(())
    }

    #[tokio::test]
    async fn test_slab_engine_tables() -> Result<()> {
        let temp_dir =
            std::env::temp_dir().join(format!("slab_engine_tables_{}", std::process::id()));
        let engine = SlabStorageEngine::with_defaults(&temp_dir)?;

        // Create database
        engine.create_database("test").await?;

        // Create tables
        engine.create_table("test", "users", "id").await?;
        engine.create_table("test", "posts", "id").await?;

        // List tables
        let tables = engine.list_tables_in_db("test").await?;
        assert_eq!(tables.len(), 2);

        // Get table info
        let info = engine.get_table_info("test.users").await?;
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.name, "users");
        assert_eq!(info.primary_key, "id");

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
        Ok(())
    }
}
