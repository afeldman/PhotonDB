//! Database hierarchy and management.
//!
//! # Overview
//!
//! RethinkDB organizes data in a three-level hierarchy:
//! ```text
//! Databases (UUID → DatabaseConfig)
//!   └─→ Tables (UUID → TableConfig with database_id)
//!        └─→ Documents (JSON/Datum with primary key)
//! ```
//!
//! # Architecture
//!
//! Each database and table is identified by a 128-bit UUID, similar to the
//! original C++ implementation where `database_id_t` and `namespace_id_t`
//! were used.
//!
//! # Examples
//!
//! ```rust
//! use rethinkdb::storage::{DatabaseEngine, DefaultStorageEngine};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create storage engine
//! let engine = DefaultStorageEngine::new("./data")?;
//!
//! // Create a database
//! let db_id = engine.create_database("my_app").await?;
//! println!("Database created with ID: {}", db_id);
//!
//! // Create a table
//! let table_id = engine.create_table("my_app", "users").await?;
//! println!("Table created with ID: {}", table_id);
//!
//! // Insert a document
//! let doc = br#"{"id": "user_123", "name": "Alice"}"#;
//! engine.set_document("my_app", "users", b"user_123", doc.to_vec()).await?;
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use crate::error::{Error, Result};

/// A unique identifier for a database.
///
/// DatabaseId is a 128-bit UUID that uniquely identifies a database in the cluster.
/// This is similar to `database_id_t` in the original C++ implementation.
///
/// # Examples
///
/// ```rust
/// use rethinkdb::storage::DatabaseId;
/// use uuid::Uuid;
///
/// // Create a new random database ID
/// let id = DatabaseId::new();
/// println!("Database ID: {}", id);
///
/// // Create from existing UUID
/// let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
/// let id = DatabaseId::from_uuid(uuid);
/// assert_eq!(id.as_uuid(), uuid);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DatabaseId(Uuid);

impl DatabaseId {
    /// Creates a new random database ID using UUIDv4.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rethinkdb::storage::DatabaseId;
    ///
    /// let id1 = DatabaseId::new();
    /// let id2 = DatabaseId::new();
    /// assert_ne!(id1, id2); // UUIDs are unique
    /// ```
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a DatabaseId from an existing UUID.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The UUID to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rethinkdb::storage::DatabaseId;
    /// use uuid::Uuid;
    ///
    /// let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    /// let id = DatabaseId::from_uuid(uuid);
    /// ```
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the underlying UUID.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rethinkdb::storage::DatabaseId;
    ///
    /// let id = DatabaseId::new();
    /// let uuid = id.as_uuid();
    /// assert_eq!(uuid.get_version_num(), 4); // UUIDv4
    /// ```
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Returns the UUID as a byte array.
    ///
    /// # Returns
    ///
    /// A reference to a 16-byte array representing the UUID.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rethinkdb::storage::DatabaseId;
    ///
    /// let id = DatabaseId::new();
    /// let bytes = id.as_bytes();
    /// assert_eq!(bytes.len(), 16); // 128 bits = 16 bytes
    /// ```
    pub fn as_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }
}

impl fmt::Display for DatabaseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for DatabaseId {
    fn default() -> Self {
        Self::new()
    }
}

/// A unique identifier for a table.
///
/// TableId is a 128-bit UUID that uniquely identifies a table within a database.
/// This corresponds to `namespace_id_t` in the original C++ implementation.
///
/// # Examples
///
/// ```rust
/// use rethinkdb::storage::TableId;
///
/// let id = TableId::new();
/// println!("Table ID: {}", id);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TableId(Uuid);

impl TableId {
    /// Creates a new random table ID using UUIDv4.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rethinkdb::storage::TableId;
    ///
    /// let id = TableId::new();
    /// println!("Created table: {}", id);
    /// ```
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a TableId from an existing UUID.
    ///
    /// # Arguments
    ///
    /// * `uuid` - The UUID to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rethinkdb::storage::TableId;
    /// use uuid::Uuid;
    ///
    /// let uuid = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();
    /// let id = TableId::from_uuid(uuid);
    /// ```
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the underlying UUID.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// Returns the UUID as a byte array.
    pub fn as_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }
}

impl fmt::Display for TableId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for TableId {
    fn default() -> Self {
        Self::new()
    }
}

/// Database configuration and metadata.
///
/// Contains all metadata for a database including its unique ID, name,
/// and creation timestamp. This structure is serialized to JSON and stored
/// in the Sled B-Tree under the key `__meta__:databases:{name}`.
///
/// # Examples
///
/// ```rust
/// use rethinkdb::storage::DatabaseConfig;
///
/// // Create a new database configuration
/// let config = DatabaseConfig::new("my_app".to_string());
/// println!("Database: {} (ID: {})", config.name, config.id);
/// println!("Created at: {}", config.created_at);
/// ```
///
/// # Storage Format
///
/// Stored as JSON in Sled:
/// ```json
/// {
///   "id": "550e8400-e29b-41d4-a716-446655440000",
///   "name": "my_app",
///   "created_at": 1704067200
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Unique database identifier (128-bit UUID).
    pub id: DatabaseId,

    /// Human-readable database name.
    ///
    /// Must be unique across the cluster and follow naming rules:
    /// - Start with letter or underscore
    /// - Contain only alphanumeric characters and underscores
    /// - Be 1-128 characters long
    pub name: String,

    /// Unix timestamp (seconds since epoch) when database was created.
    pub created_at: u64,
}

impl DatabaseConfig {
    /// Creates a new database configuration with a random UUID.
    ///
    /// # Arguments
    ///
    /// * `name` - The database name (will be validated separately)
    ///
    /// # Returns
    ///
    /// A new DatabaseConfig with a random ID and current timestamp.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rethinkdb::storage::DatabaseConfig;
    ///
    /// let config = DatabaseConfig::new("production".to_string());
    /// assert_eq!(config.name, "production");
    /// assert!(config.created_at > 0);
    /// ```
    pub fn new(name: String) -> Self {
        Self {
            id: DatabaseId::new(),
            name,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Table configuration and metadata.
///
/// Contains all metadata for a table including its unique ID, name, parent database,
/// primary key field, and creation timestamp. Similar to `table_basic_config_t` in
/// the original C++ implementation.
///
/// # Examples
///
/// ```rust
/// use rethinkdb::storage::{TableConfig, DatabaseId};
/// use uuid::Uuid;
///
/// let db_id = DatabaseId::from_uuid(
///     Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
/// );
///
/// // Create table with default primary key ("id")
/// let config = TableConfig::new("users".to_string(), db_id);
/// assert_eq!(config.primary_key, "id");
///
/// // Create table with custom primary key
/// let config = TableConfig::new("sessions".to_string(), db_id)
///     .with_primary_key("session_id".to_string());
/// assert_eq!(config.primary_key, "session_id");
/// ```
///
/// # Storage Format
///
/// Stored as JSON in Sled under `__meta__:tables:{db_name}.{table_name}`:
/// ```json
/// {
///   "id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
///   "name": "users",
///   "database_id": "550e8400-e29b-41d4-a716-446655440000",
///   "primary_key": "id",
///   "created_at": 1704067200,
///   "doc_count": 1523,
///   "indexes": ["email", "username"]
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableConfig {
    /// Unique table identifier (128-bit UUID).
    ///
    /// Called `namespace_id` in the C++ implementation.
    pub id: TableId,

    /// Human-readable table name.
    ///
    /// Must be unique within the database and follow the same naming rules
    /// as database names.
    pub name: String,

    /// Parent database identifier.
    ///
    /// This creates the hierarchical relationship: Database → Table.
    pub database_id: DatabaseId,

    /// Primary key field name.
    ///
    /// Defaults to "id". Every document in the table must have this field.
    pub primary_key: String,

    /// Unix timestamp (seconds since epoch) when table was created.
    pub created_at: u64,

    /// Cached document count.
    ///
    /// Updated during table operations. May be stale in distributed setups.
    pub doc_count: u64,

    /// List of secondary index names.
    ///
    /// Indexes are stored separately under keys like:
    /// `db:{db_id}:table:{table_id}:idx:{index_name}:{value}`
    pub indexes: Vec<String>,
}

impl TableConfig {
    /// Creates a new table configuration with default settings.
    ///
    /// # Arguments
    ///
    /// * `name` - The table name
    /// * `database_id` - The parent database ID
    ///
    /// # Returns
    ///
    /// A new TableConfig with:
    /// - Random TableId
    /// - Primary key set to "id"
    /// - Current timestamp
    /// - Empty document count and index list
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rethinkdb::storage::{TableConfig, DatabaseId};
    ///
    /// let db_id = DatabaseId::new();
    /// let config = TableConfig::new("users".to_string(), db_id);
    ///
    /// assert_eq!(config.name, "users");
    /// assert_eq!(config.database_id, db_id);
    /// assert_eq!(config.primary_key, "id");
    /// assert_eq!(config.doc_count, 0);
    /// ```
    pub fn new(name: String, database_id: DatabaseId) -> Self {
        Self {
            id: TableId::new(),
            name,
            database_id,
            primary_key: "id".to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            doc_count: 0,
            indexes: Vec::new(),
        }
    }

    /// Sets a custom primary key for the table (builder pattern).
    ///
    /// By default, tables use "id" as the primary key field. This method
    /// allows customization, useful when importing data with different
    /// primary key conventions.
    ///
    /// # Arguments
    ///
    /// * `primary_key` - The field name to use as primary key
    ///
    /// # Returns
    ///
    /// Self for method chaining
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rethinkdb::storage::{TableConfig, DatabaseId};
    ///
    /// let db_id = DatabaseId::new();
    ///
    /// // Create table with custom primary key
    /// let config = TableConfig::new("sessions".to_string(), db_id)
    ///     .with_primary_key("session_id".to_string());
    ///
    /// assert_eq!(config.primary_key, "session_id");
    ///
    /// // Documents in this table must have a "session_id" field:
    /// // {
    /// //   "session_id": "abc123",
    /// //   "user_id": 42,
    /// //   "created_at": 1704067200
    /// // }
    /// ```
    ///
    /// # Note
    ///
    /// The primary key field must exist in all documents inserted into
    /// the table. Missing primary keys will cause insertion errors.
    pub fn with_primary_key(mut self, primary_key: String) -> Self {
        self.primary_key = primary_key;
        self
    }
}

/// Database engine trait - manages the database hierarchy.
///
/// This trait defines the core operations for managing databases and tables
/// in RethinkDB. It provides a hierarchical storage model where databases
/// contain tables, and tables contain documents.
///
/// # Hierarchy
///
/// ```text
/// DatabaseEngine
///   ├─ Database 1
///   │   ├─ Table A (documents)
///   │   └─ Table B (documents)
///   └─ Database 2
///       └─ Table C (documents)
/// ```
///
/// # Storage Implementations
///
/// Different backends can be used for metadata storage:
/// - **Sled B-Tree** (default): Embedded, ACID-compliant, ordered
/// - **RocksDB**: High-performance LSM tree
/// - **PostgreSQL**: SQL-based with full ACID guarantees
///
/// # Examples
///
/// ## Basic Database Operations
///
/// ```rust
/// use rethinkdb::storage::{DatabaseEngine, DefaultStorageEngine};
///
/// # async fn example() -> anyhow::Result<()> {
/// let engine = DefaultStorageEngine::new("test.db").await?;
///
/// // Create database
/// let db_id = engine.create_database("myapp").await?;
///
/// // List databases
/// let dbs = engine.list_databases().await?;
/// assert!(dbs.contains(&"myapp".to_string()));
///
/// // Drop database
/// engine.drop_database("myapp").await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Table Operations
///
/// ```rust
/// # use rethinkdb::storage::{DatabaseEngine, DefaultStorageEngine};
/// # async fn example() -> anyhow::Result<()> {
/// # let engine = DefaultStorageEngine::new("test.db").await?;
/// # engine.create_database("myapp").await?;
///
/// // Create table with default primary key ("id")
/// let table_id = engine.create_table("myapp", "users").await?;
///
/// // Create table with custom primary key
/// engine.create_table_with_pk("myapp", "sessions", "session_id").await?;
///
/// // List tables
/// let tables = engine.list_tables("myapp").await?;
/// assert_eq!(tables.len(), 2);
/// # Ok(())
/// # }
/// ```
///
/// ## Document Operations
///
/// ```rust
/// # use rethinkdb::storage::{DatabaseEngine, DefaultStorageEngine};
/// # use serde_json::json;
/// # async fn example() -> anyhow::Result<()> {
/// # let engine = DefaultStorageEngine::new("test.db").await?;
/// # engine.create_database("myapp").await?;
/// # engine.create_table("myapp", "users").await?;
///
/// // Insert document
/// let doc = json!({"id": "user123", "name": "Alice"});
/// engine.set_document(
///     "myapp",
///     "users",
///     b"user123",
///     serde_json::to_vec(&doc)?
/// ).await?;
///
/// // Retrieve document
/// if let Some(data) = engine.get_document("myapp", "users", b"user123").await? {
///     let doc: serde_json::Value = serde_json::from_slice(&data)?;
///     assert_eq!(doc["name"], "Alice");
/// }
///
/// // Delete document
/// engine.delete_document("myapp", "users", b"user123").await?;
/// # Ok(())
/// # }
/// ```
///
/// # Thread Safety
///
/// All implementations must be `Send + Sync` for use in async contexts.
/// Multiple concurrent operations on the same database/table are safe.
///
/// # Error Handling
///
/// Methods return `Result<T>` where errors indicate:
/// - Database/table not found
/// - Name validation failures
/// - I/O errors
/// - Corruption or inconsistency
#[async_trait]
pub trait DatabaseEngine: Send + Sync {
    // ===== Database Operations =====

    /// Creates a new database with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - Database name (must be valid per `validate_name`)
    ///
    /// # Returns
    ///
    /// The newly created DatabaseId
    ///
    /// # Errors
    ///
    /// - Database already exists
    /// - Invalid name (see `validate_name`)
    /// - Storage I/O error
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rethinkdb::storage::{DatabaseEngine, DefaultStorageEngine};
    /// # async fn example() -> anyhow::Result<()> {
    /// # let engine = DefaultStorageEngine::new("test.db").await?;
    /// let db_id = engine.create_database("production").await?;
    /// assert!(engine.database_exists("production").await?);
    /// # Ok(())
    /// # }
    /// ```
    async fn create_database(&self, name: &str) -> Result<DatabaseId>;

    /// Drops (deletes) a database and all its tables.
    ///
    /// This is a destructive operation that cannot be undone. All tables
    /// and documents within the database are permanently deleted.
    ///
    /// # Arguments
    ///
    /// * `name` - Database name to drop
    ///
    /// # Errors
    ///
    /// - Database not found
    /// - Storage I/O error
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rethinkdb::storage::{DatabaseEngine, DefaultStorageEngine};
    /// # async fn example() -> anyhow::Result<()> {
    /// # let engine = DefaultStorageEngine::new("test.db").await?;
    /// # engine.create_database("temp").await?;
    /// engine.drop_database("temp").await?;
    /// assert!(!engine.database_exists("temp").await?);
    /// # Ok(())
    /// # }
    /// ```
    async fn drop_database(&self, name: &str) -> Result<()>;

    /// Drops a database by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - DatabaseId to drop
    ///
    /// # Errors
    ///
    /// - Database not found
    /// - Storage I/O error
    async fn drop_database_by_id(&self, id: DatabaseId) -> Result<()>;

    /// Lists all database names in the system.
    ///
    /// # Returns
    ///
    /// Vector of database names, sorted alphabetically
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rethinkdb::storage::{DatabaseEngine, DefaultStorageEngine};
    /// # async fn example() -> anyhow::Result<()> {
    /// # let engine = DefaultStorageEngine::new("test.db").await?;
    /// # engine.create_database("app1").await?;
    /// # engine.create_database("app2").await?;
    /// let dbs = engine.list_databases().await?;
    /// assert_eq!(dbs, vec!["app1", "app2"]);
    /// # Ok(())
    /// # }
    /// ```
    async fn list_databases(&self) -> Result<Vec<String>>;

    /// Retrieves database configuration by name.
    ///
    /// # Arguments
    ///
    /// * `name` - Database name
    ///
    /// # Returns
    ///
    /// `Some(DatabaseConfig)` if found, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rethinkdb::storage::{DatabaseEngine, DefaultStorageEngine};
    /// # async fn example() -> anyhow::Result<()> {
    /// # let engine = DefaultStorageEngine::new("test.db").await?;
    /// # engine.create_database("mydb").await?;
    /// if let Some(config) = engine.get_database_config("mydb").await? {
    ///     println!("Database ID: {}", config.id);
    ///     println!("Created: {}", config.created_at);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn get_database_config(&self, name: &str) -> Result<Option<DatabaseConfig>>;

    /// Retrieves database configuration by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - DatabaseId
    ///
    /// # Returns
    ///
    /// `Some(DatabaseConfig)` if found, `None` otherwise
    async fn get_database_config_by_id(&self, id: DatabaseId) -> Result<Option<DatabaseConfig>>;

    /// Checks if a database exists.
    ///
    /// # Arguments
    ///
    /// * `name` - Database name
    ///
    /// # Returns
    ///
    /// `true` if database exists, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rethinkdb::storage::{DatabaseEngine, DefaultStorageEngine};
    /// # async fn example() -> anyhow::Result<()> {
    /// # let engine = DefaultStorageEngine::new("test.db").await?;
    /// if !engine.database_exists("mydb").await? {
    ///     engine.create_database("mydb").await?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn database_exists(&self, name: &str) -> Result<bool>;

    // ===== Table Operations (scoped to database) =====

    /// Creates a new table in a database.
    ///
    /// Uses "id" as the default primary key field.
    ///
    /// # Arguments
    ///
    /// * `db_name` - Parent database name
    /// * `table_name` - New table name (must be unique in database)
    ///
    /// # Returns
    ///
    /// The newly created TableId
    ///
    /// # Errors
    ///
    /// - Database not found
    /// - Table already exists
    /// - Invalid name
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rethinkdb::storage::{DatabaseEngine, DefaultStorageEngine};
    /// # async fn example() -> anyhow::Result<()> {
    /// # let engine = DefaultStorageEngine::new("test.db").await?;
    /// # engine.create_database("mydb").await?;
    /// let table_id = engine.create_table("mydb", "users").await?;
    /// assert!(engine.table_exists("mydb", "users").await?);
    /// # Ok(())
    /// # }
    /// ```
    async fn create_table(&self, db_name: &str, table_name: &str) -> Result<TableId>;

    /// Creates a table with a custom primary key field.
    ///
    /// # Arguments
    ///
    /// * `db_name` - Parent database name
    /// * `table_name` - New table name
    /// * `primary_key` - Custom primary key field name
    ///
    /// # Returns
    ///
    /// The newly created TableId
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rethinkdb::storage::{DatabaseEngine, DefaultStorageEngine};
    /// # async fn example() -> anyhow::Result<()> {
    /// # let engine = DefaultStorageEngine::new("test.db").await?;
    /// # engine.create_database("mydb").await?;
    /// // Create table with "user_id" as primary key
    /// engine.create_table_with_pk("mydb", "users", "user_id").await?;
    ///
    /// // Documents must have a "user_id" field
    /// # Ok(())
    /// # }
    /// ```
    async fn create_table_with_pk(
        &self,
        db_name: &str,
        table_name: &str,
        primary_key: &str,
    ) -> Result<TableId>;

    /// Drops (deletes) a table and all its documents.
    ///
    /// # Arguments
    ///
    /// * `db_name` - Parent database name
    /// * `table_name` - Table name to drop
    ///
    /// # Errors
    ///
    /// - Database not found
    /// - Table not found
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rethinkdb::storage::{DatabaseEngine, DefaultStorageEngine};
    /// # async fn example() -> anyhow::Result<()> {
    /// # let engine = DefaultStorageEngine::new("test.db").await?;
    /// # engine.create_database("mydb").await?;
    /// # engine.create_table("mydb", "temp").await?;
    /// engine.drop_table("mydb", "temp").await?;
    /// assert!(!engine.table_exists("mydb", "temp").await?);
    /// # Ok(())
    /// # }
    /// ```
    async fn drop_table(&self, db_name: &str, table_name: &str) -> Result<()>;

    /// Lists all tables in a database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - Database name
    ///
    /// # Returns
    ///
    /// Vector of table names, sorted alphabetically
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rethinkdb::storage::{DatabaseEngine, DefaultStorageEngine};
    /// # async fn example() -> anyhow::Result<()> {
    /// # let engine = DefaultStorageEngine::new("test.db").await?;
    /// # engine.create_database("mydb").await?;
    /// # engine.create_table("mydb", "users").await?;
    /// # engine.create_table("mydb", "posts").await?;
    /// let tables = engine.list_tables("mydb").await?;
    /// assert_eq!(tables, vec!["posts", "users"]);
    /// # Ok(())
    /// # }
    /// ```
    async fn list_tables(&self, db_name: &str) -> Result<Vec<String>>;

    /// Retrieves table configuration.
    ///
    /// # Arguments
    ///
    /// * `db_name` - Parent database name
    /// * `table_name` - Table name
    ///
    /// # Returns
    ///
    /// `Some(TableConfig)` if found, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rethinkdb::storage::{DatabaseEngine, DefaultStorageEngine};
    /// # async fn example() -> anyhow::Result<()> {
    /// # let engine = DefaultStorageEngine::new("test.db").await?;
    /// # engine.create_database("mydb").await?;
    /// # engine.create_table("mydb", "users").await?;
    /// if let Some(config) = engine.get_table_config("mydb", "users").await? {
    ///     println!("Primary key: {}", config.primary_key);
    ///     println!("Document count: {}", config.doc_count);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn get_table_config(
        &self,
        db_name: &str,
        table_name: &str,
    ) -> Result<Option<TableConfig>>;

    /// Retrieves table configuration by ID.
    ///
    /// # Arguments
    ///
    /// * `table_id` - TableId
    ///
    /// # Returns
    ///
    /// `Some(TableConfig)` if found, `None` otherwise
    async fn get_table_config_by_id(&self, table_id: TableId) -> Result<Option<TableConfig>>;

    /// Checks if a table exists in a database.
    ///
    /// # Arguments
    ///
    /// * `db_name` - Parent database name
    /// * `table_name` - Table name
    ///
    /// # Returns
    ///
    /// `true` if table exists, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rethinkdb::storage::{DatabaseEngine, DefaultStorageEngine};
    /// # async fn example() -> anyhow::Result<()> {
    /// # let engine = DefaultStorageEngine::new("test.db").await?;
    /// # engine.create_database("mydb").await?;
    /// if !engine.table_exists("mydb", "users").await? {
    ///     engine.create_table("mydb", "users").await?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn table_exists(&self, db_name: &str, table_name: &str) -> Result<bool>;

    // ===== Document Operations (scoped to table) =====

    /// Retrieves a document by its primary key.
    ///
    /// # Arguments
    ///
    /// * `db_name` - Database name
    /// * `table_name` - Table name
    /// * `key` - Primary key value (as bytes)
    ///
    /// # Returns
    ///
    /// `Some(Vec<u8>)` containing the document bytes if found, `None` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rethinkdb::storage::{DatabaseEngine, DefaultStorageEngine};
    /// # use serde_json::json;
    /// # async fn example() -> anyhow::Result<()> {
    /// # let engine = DefaultStorageEngine::new("test.db").await?;
    /// # engine.create_database("mydb").await?;
    /// # engine.create_table("mydb", "users").await?;
    /// if let Some(data) = engine.get_document("mydb", "users", b"user123").await? {
    ///     let doc: serde_json::Value = serde_json::from_slice(&data)?;
    ///     println!("User: {}", doc);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn get_document(
        &self,
        db_name: &str,
        table_name: &str,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>>;

    /// Inserts or updates a document.
    ///
    /// If a document with the same primary key exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `db_name` - Database name
    /// * `table_name` - Table name
    /// * `key` - Primary key value (as bytes)
    /// * `value` - Document data (typically JSON encoded)
    ///
    /// # Errors
    ///
    /// - Database not found
    /// - Table not found
    /// - Storage I/O error
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rethinkdb::storage::{DatabaseEngine, DefaultStorageEngine};
    /// # use serde_json::json;
    /// # async fn example() -> anyhow::Result<()> {
    /// # let engine = DefaultStorageEngine::new("test.db").await?;
    /// # engine.create_database("mydb").await?;
    /// # engine.create_table("mydb", "users").await?;
    /// let doc = json!({"id": "user123", "name": "Bob", "email": "bob@example.com"});
    /// engine.set_document(
    ///     "mydb",
    ///     "users",
    ///     b"user123",
    ///     serde_json::to_vec(&doc)?
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn set_document(
        &self,
        db_name: &str,
        table_name: &str,
        key: &[u8],
        value: Vec<u8>,
    ) -> Result<()>;

    /// Deletes a document by its primary key.
    ///
    /// If the document doesn't exist, this is a no-op (no error).
    ///
    /// # Arguments
    ///
    /// * `db_name` - Database name
    /// * `table_name` - Table name
    /// * `key` - Primary key value (as bytes)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use rethinkdb::storage::{DatabaseEngine, DefaultStorageEngine};
    /// # async fn example() -> anyhow::Result<()> {
    /// # let engine = DefaultStorageEngine::new("test.db").await?;
    /// # engine.create_database("mydb").await?;
    /// # engine.create_table("mydb", "users").await?;
    /// engine.delete_document("mydb", "users", b"user123").await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn delete_document(&self, db_name: &str, table_name: &str, key: &[u8]) -> Result<()>;

    /// Count documents in a table
    async fn count_documents(&self, db_name: &str, table_name: &str) -> Result<u64>;
}

/// Name validation for databases and tables
/// Validates a database or table name.
///
/// Names must follow these rules:
/// 1. Not empty
/// 2. Maximum 128 characters
/// 3. Start with letter (a-z, A-Z) or underscore (_)
/// 4. Contain only letters, numbers, and underscores
///
/// # Arguments
///
/// * `name` - The name to validate
///
/// # Returns
///
/// `Ok(())` if valid, `Err` with explanation if invalid
///
/// # Examples
///
/// ## Valid Names
///
/// ```rust
/// use rethinkdb::storage::validate_name;
///
/// // Good names
/// assert!(validate_name("users").is_ok());
/// assert!(validate_name("Users2024").is_ok());
/// assert!(validate_name("_internal").is_ok());
/// assert!(validate_name("user_sessions").is_ok());
/// ```
///
/// ## Invalid Names
///
/// ```rust
/// use rethinkdb::storage::validate_name;
///
/// // Empty
/// assert!(validate_name("").is_err());
///
/// // Starts with number
/// assert!(validate_name("123users").is_err());
///
/// // Contains special characters
/// assert!(validate_name("user-sessions").is_err());
/// assert!(validate_name("user.sessions").is_err());
/// assert!(validate_name("user@host").is_err());
///
/// // Too long (> 128 chars)
/// let long_name = "a".repeat(129);
/// assert!(validate_name(&long_name).is_err());
/// ```
///
/// # Design Rationale
///
/// These rules match the C++ RethinkDB implementation and ensure:
/// - Names are safe for filesystem paths
/// - Names are valid in ReQL queries without escaping
/// - Compatibility with most programming language identifiers
pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(Error::InvalidArgument("Name cannot be empty".to_string()));
    }

    if name.len() > 128 {
        return Err(Error::InvalidArgument(
            "Name cannot be longer than 128 characters".to_string(),
        ));
    }

    // Must start with letter or underscore
    let first_char = name.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return Err(Error::InvalidArgument(
            "Name must start with a letter or underscore".to_string(),
        ));
    }

    // Must contain only alphanumeric characters and underscores
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(Error::InvalidArgument(
            "Name can only contain letters, numbers, and underscores".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_id() {
        let id1 = DatabaseId::new();
        let id2 = DatabaseId::new();
        assert_ne!(id1, id2);

        let uuid = Uuid::new_v4();
        let id3 = DatabaseId::from_uuid(uuid);
        assert_eq!(id3.as_uuid(), uuid);
    }

    #[test]
    fn test_table_id() {
        let id1 = TableId::new();
        let id2 = TableId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_database_config() {
        let config = DatabaseConfig::new("my_database".to_string());
        assert_eq!(config.name, "my_database");
        assert!(config.created_at > 0);
    }

    #[test]
    fn test_table_config() {
        let db_id = DatabaseId::new();
        let config = TableConfig::new("users".to_string(), db_id);
        assert_eq!(config.name, "users");
        assert_eq!(config.database_id, db_id);
        assert_eq!(config.primary_key, "id");
    }

    #[test]
    fn test_table_config_custom_pk() {
        let db_id = DatabaseId::new();
        let config =
            TableConfig::new("users".to_string(), db_id).with_primary_key("email".to_string());
        assert_eq!(config.primary_key, "email");
    }

    #[test]
    fn test_validate_name() {
        // Valid names
        assert!(validate_name("database").is_ok());
        assert!(validate_name("my_database_123").is_ok());
        assert!(validate_name("_internal").is_ok());
        assert!(validate_name("Database1").is_ok());

        // Invalid names
        assert!(validate_name("").is_err());
        assert!(validate_name("123database").is_err());
        assert!(validate_name("my-database").is_err());
        assert!(validate_name("my.database").is_err());
        assert!(validate_name("my database").is_err());
        assert!(validate_name(&"a".repeat(129)).is_err());
    }
}
