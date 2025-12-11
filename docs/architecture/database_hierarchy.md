# Database Hierarchy

This document describes RethinkDB's hierarchical database architecture.

## Visual Overview

**Quick Reference:** See the [architecture visualizations](README.md) for graphical representations:

- [Complete Architecture Diagram](database_hierarchy.png) - Full system overview
- [Simplified Hierarchy](database_hierarchy_simple.png) - Quick reference
- [Storage Layout](storage_layout.png) - Key structure and operations

## Architecture Overview

RethinkDB organizes data in a three-level hierarchy:

```
Databases (UUID → DatabaseConfig)
  └─→ Tables (UUID → TableConfig with database_id)
       └─→ Documents (JSON/Datum with primary key)
```

![Database Hierarchy](database_hierarchy_simple.png)

This is similar to the original C++ implementation where:

- Databases contain multiple tables
- Each table belongs to exactly one database
- Documents are stored within tables

## Data Structures

### Database

A database is identified by a UUID and has a name:

```rust
pub struct DatabaseConfig {
    pub id: DatabaseId,           // UUID (128-bit)
    pub name: String,              // Unique name
    pub created_at: u64,           // Unix timestamp
}
```

**Example:**

```rust
let db_config = DatabaseConfig {
    id: DatabaseId::from_uuid(uuid!("550e8400-e29b-41d4-a716-446655440000")),
    name: "my_application".to_string(),
    created_at: 1704067200,
};
```

### Table

A table is identified by a UUID and belongs to a database:

```rust
pub struct TableConfig {
    pub id: TableId,               // UUID (also called namespace_id in C++)
    pub name: String,              // Table name (unique within database)
    pub database_id: DatabaseId,   // Parent database
    pub primary_key: String,       // Primary key field (default: "id")
    pub created_at: u64,           // Unix timestamp
    pub doc_count: u64,            // Cached document count
    pub indexes: Vec<String>,      // Secondary indexes
}
```

**Example:**

```rust
let table_config = TableConfig {
    id: TableId::from_uuid(uuid!("6ba7b810-9dad-11d1-80b4-00c04fd430c8")),
    name: "users".to_string(),
    database_id: DatabaseId::from_uuid(uuid!("550e8400-e29b-41d4-a716-446655440000")),
    primary_key: "id".to_string(),
    created_at: 1704067200,
    doc_count: 1523,
    indexes: vec!["email".to_string(), "username".to_string()],
};
```

### Document

Documents are JSON objects stored within tables. Each document has a primary key that uniquely identifies it within the table.

**Example:**

```json
{
  "id": "user_123",
  "name": "Alice",
  "email": "alice@example.com",
  "age": 30
}
```

## Storage Layout

The Sled B-Tree storage engine uses key prefixing for namespace isolation:

### Metadata Keys

```
__meta__:databases:{db_name}           → DatabaseConfig (JSON)
__meta__:database_ids:{db_id}          → database_name (string)
__meta__:tables:{db_name}.{table_name} → TableConfig (JSON)
__meta__:table_ids:{table_id}          → "{db_name}.{table_name}" (string)
```

### Document Keys

```
db:{db_id}:table:{table_id}:docs:{primary_key} → Document (JSON)
```

**Example keys:**

```
__meta__:databases:my_app
__meta__:database_ids:550e8400-e29b-41d4-a716-446655440000
__meta__:tables:my_app.users
__meta__:table_ids:6ba7b810-9dad-11d1-80b4-00c04fd430c8
db:550e8400-e29b-41d4-a716-446655440000:table:6ba7b810-9dad-11d1-80b4-00c04fd430c8:docs:user_123
```

## Database Operations

### Create Database

```rust
use rethinkdb::storage::{SledDatabaseEngine, DatabaseEngine};

let engine = SledDatabaseEngine::new("./data")?;
let db_id = engine.create_database("my_app").await?;
```

**Validation:**

- Name must start with letter or underscore
- Name can only contain alphanumeric characters and underscores
- Name must be 1-128 characters long
- Name must be unique

### List Databases

```rust
let databases = engine.list_databases().await?;
// Returns: ["my_app", "test_db", "analytics"]
```

### Get Database Config

```rust
let config = engine.get_database_config("my_app").await?;
println!("Database ID: {}", config.id);
println!("Created at: {}", config.created_at);
```

### Drop Database

```rust
engine.drop_database("my_app").await?;
```

**Note:** Dropping a database will cascade delete all tables and documents within it.

## Table Operations

### Create Table

```rust
// Create with default primary key ("id")
let table_id = engine.create_table("my_app", "users").await?;

// Create with custom primary key
let table_id = engine.create_table_with_pk("my_app", "sessions", "session_id").await?;
```

**Validation:**

- Database must exist
- Table name follows same rules as database names
- Table name must be unique within the database

### List Tables

```rust
let tables = engine.list_tables("my_app").await?;
// Returns: ["users", "posts", "comments"]
```

### Get Table Config

```rust
let config = engine.get_table_config("my_app", "users").await?;
println!("Table ID: {}", config.id);
println!("Primary key: {}", config.primary_key);
println!("Document count: {}", config.doc_count);
```

### Drop Table

```rust
engine.drop_table("my_app", "users").await?;
```

**Note:** Dropping a table will delete all documents within it.

## Document Operations

### Insert/Update Document

```rust
let key = b"user_123";
let doc = serde_json::json!({
    "id": "user_123",
    "name": "Alice",
    "email": "alice@example.com",
    "age": 30
});
let value = serde_json::to_vec(&doc)?;

engine.set_document("my_app", "users", key, value).await?;
```

### Get Document

```rust
let doc_bytes = engine.get_document("my_app", "users", b"user_123").await?;
if let Some(bytes) = doc_bytes {
    let doc: serde_json::Value = serde_json::from_slice(&bytes)?;
    println!("User: {:?}", doc);
}
```

### Delete Document

```rust
engine.delete_document("my_app", "users", b"user_123").await?;
```

### Count Documents

```rust
let count = engine.count_documents("my_app", "users").await?;
println!("Total users: {}", count);
```

## ReQL Integration

The database hierarchy is integrated with ReQL queries:

```javascript
// JavaScript/ReQL syntax
r.db("my_app").table("users").get("user_123");
r.db("my_app").table("users").filter({ age: 30 });
r.db("my_app").tableCreate("posts");
r.dbCreate("analytics");
```

**Rust equivalent:**

```rust
// This will be implemented in the query executor
let query = r.db("my_app").table("users").get("user_123");
let result = execute_query(&engine, query).await?;
```

## Comparison with Original C++ Implementation

| Component         | C++ Implementation                | Rust Implementation  |
| ----------------- | --------------------------------- | -------------------- |
| Database ID       | `uuid_u` (16 bytes)               | `DatabaseId(Uuid)`   |
| Table ID          | `namespace_id_t` (uuid_u)         | `TableId(Uuid)`      |
| Database metadata | `database_semilattice_metadata_t` | `DatabaseConfig`     |
| Table metadata    | `table_basic_config_t`            | `TableConfig`        |
| Storage           | Custom B-Tree                     | Sled B-Tree          |
| Clustering        | Semilattice replication           | Raft-based (planned) |

## Performance Characteristics

### Database Operations

- **Create/Drop Database:** O(1) - Single key write
- **List Databases:** O(n) - Scan all database entries
- **Get Database Config:** O(1) - Single key lookup

### Table Operations

- **Create/Drop Table:** O(1) - Single key write
- **List Tables:** O(n) - Scan tables in database (prefix scan)
- **Get Table Config:** O(1) - Single key lookup

### Document Operations

- **Get/Set/Delete Document:** O(log n) - B-Tree operation
- **Count Documents:** O(n) - Scan all documents in table
- **Range Query:** O(log n + m) - where m is result size

## Future Enhancements

### 1. Graph Database Structure (Planned)

For complex relationships between databases:

```rust
pub struct DatabaseGraph {
    nodes: HashMap<DatabaseId, DatabaseConfig>,
    edges: Vec<(DatabaseId, DatabaseId, Relationship)>,
}
```

Use case: Multi-tenant applications with shared reference data.

### 2. Merkle Tree for Replication (Planned)

Content-addressable storage for efficient replication:

```rust
pub struct MerkleDatabase {
    root_hash: [u8; 32],
    nodes: HashMap<[u8; 32], MerkleNode>,
}
```

Use case: Distributed systems with conflict resolution.

### 3. Associative Storage (Planned)

Hash-based storage for ultra-fast lookups:

```rust
pub struct AssociativeStorage {
    databases: DashMap<String, DatabaseId>,
    tables: DashMap<(DatabaseId, String), TableId>,
}
```

Use case: High-throughput key-value workloads.

## Migration from Flat Structure

If you have an existing RethinkDB installation without database hierarchy, migrate as follows:

1. **Backup your data:**

   ```bash
   rethinkdb dump -f backup.tar.gz
   ```

2. **Create default database:**

   ```rust
   engine.create_database("rethinkdb").await?;
   ```

3. **Move tables to database:**

   ```rust
   for table_name in old_engine.list_tables().await? {
       engine.create_table("rethinkdb", &table_name).await?;
       // Copy documents...
   }
   ```

4. **Update application code:**
   ```javascript
   // Old: r.table("users")
   // New: r.db("rethinkdb").table("users")
   ```

## See Also

- [Storage Engine](../storage/README.md)
- [ReQL Query Language](../query/README.md)
- [Clustering & Replication](../clustering/README.md)
- [HTTP API Reference](../api/http.md)
