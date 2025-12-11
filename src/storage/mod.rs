//! Storage layer
//!
//! # Architecture
//!
//! RethinkDB has a hierarchical storage structure:
//!
//! ```text
//! Databases (UUID → DatabaseConfig)
//!   └─→ Tables (UUID → TableConfig with database_id)
//!        └─→ Documents (JSON/Datum with primary key)
//! ```
//!
//! ## Storage Engine
//!
//! The `DatabaseEngine` trait provides the main interface for database operations:
//! - Database creation, deletion, listing
//! - Table creation, deletion, listing (scoped to databases)
//! - Document CRUD operations (scoped to tables)
//!
//! ## Implementation
//!
//! The default implementation uses:
//! - **Slab Storage** with compression and caching (Phase 1-4 complete)
//! - **UUID** for database and table identifiers
//! - **Key prefixing** for namespace isolation: `db:{db_id}:tables`, `db:{db_id}:table:{table_id}:docs`
//!
//! ### Legacy Storage (Deprecated)
//! - **Sled B-Tree** (deprecated, use migration tools to convert to Slab)

pub mod btree_storage;
pub mod database;
pub mod engine;
pub mod mock;
pub mod slab;

// Default storage engine (Phase 5)
pub use slab::SlabStorageEngine as DefaultStorageEngine;

// All storage implementations
pub use slab::{SlabAllocator, SlabStorage, SlabStorageEngine};
pub use btree_storage::BTreeStorage;
pub use mock::MockStorage;
pub use database::{
    validate_name, DatabaseConfig, DatabaseEngine, DatabaseId, TableConfig, TableId,
};
pub use engine::{Storage, StorageEngine, TableInfo};
