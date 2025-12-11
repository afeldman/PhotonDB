//! Storage engine trait

use crate::error::Result;
use crate::reql::Datum;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Table metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub db: String,
    pub primary_key: String,
    pub doc_count: u64,
    pub indexes: Vec<String>,
}

/// Storage engine trait
#[async_trait]
pub trait StorageEngine: Send + Sync {
    async fn get(&self, key: &[u8]) -> Result<Option<Datum>>;
    async fn set(&self, key: &[u8], value: Datum) -> Result<()>;
    async fn delete(&self, key: &[u8]) -> Result<()>;

    /// List all tables in the database
    async fn list_tables(&self) -> Result<Vec<String>>;

    /// Get table metadata
    async fn get_table_info(&self, table_name: &str) -> Result<Option<TableInfo>>;
    
    /// List all databases
    async fn list_databases(&self) -> Result<Vec<String>>;
    
    /// Create a database
    async fn create_database(&self, name: &str) -> Result<()>;
    
    /// Drop a database
    async fn drop_database(&self, name: &str) -> Result<()>;
    
    /// List tables in a database
    async fn list_tables_in_db(&self, db: &str) -> Result<Vec<String>>;
    
    /// Create a table
    async fn create_table(&self, db: &str, table: &str, primary_key: &str) -> Result<()>;
    
    /// Drop a table
    async fn drop_table(&self, db: &str, table: &str) -> Result<()>;
    
    /// Scan all documents in a table
    async fn scan_table(&self, db: &str, table: &str) -> Result<Vec<Datum>>;
}

/// Main storage interface
pub struct Storage {
    engine: Box<dyn StorageEngine>,
}

impl std::fmt::Debug for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Storage").finish()
    }
}

impl Storage {
    pub fn new(engine: Box<dyn StorageEngine>) -> Self {
        Self { engine }
    }

    pub async fn get(&self, key: &[u8]) -> Result<Option<Datum>> {
        self.engine.get(key).await
    }

    pub async fn set(&self, key: &[u8], value: Datum) -> Result<()> {
        self.engine.set(key, value).await
    }

    pub async fn delete(&self, key: &[u8]) -> Result<()> {
        self.engine.delete(key).await
    }

    pub async fn list_tables(&self) -> Result<Vec<String>> {
        self.engine.list_tables().await
    }

    pub async fn get_table_info(&self, table_name: &str) -> Result<Option<TableInfo>> {
        self.engine.get_table_info(table_name).await
    }
    
    pub async fn list_databases(&self) -> Result<Vec<String>> {
        self.engine.list_databases().await
    }
    
    pub async fn create_database(&self, name: &str) -> Result<()> {
        self.engine.create_database(name).await
    }
    
    pub async fn drop_database(&self, name: &str) -> Result<()> {
        self.engine.drop_database(name).await
    }
    
    pub async fn list_tables_in_db(&self, db: &str) -> Result<Vec<String>> {
        self.engine.list_tables_in_db(db).await
    }
    
    pub async fn create_table(&self, db: &str, table: &str, primary_key: &str) -> Result<()> {
        self.engine.create_table(db, table, primary_key).await
    }
    
    pub async fn drop_table(&self, db: &str, table: &str) -> Result<()> {
        self.engine.drop_table(db, table).await
    }
    
    pub async fn scan_table(&self, db: &str, table: &str) -> Result<Vec<Datum>> {
        self.engine.scan_table(db, table).await
    }
}
