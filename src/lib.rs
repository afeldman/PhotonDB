// RethinkDB - Rust Implementation
// A distributed realtime document database

#![warn(rust_2018_idioms)]
#![allow(dead_code)] // During development

// Generated Cap'n Proto modules (must be at crate root for proper imports)
pub mod types_capnp {
    include!(concat!(env!("OUT_DIR"), "/types_capnp.rs"));
}

pub mod handshake_capnp {
    include!(concat!(env!("OUT_DIR"), "/handshake_capnp.rs"));
}

pub mod term_capnp {
    include!(concat!(env!("OUT_DIR"), "/term_capnp.rs"));
}

pub mod query_capnp {
    include!(concat!(env!("OUT_DIR"), "/query_capnp.rs"));
}

pub mod response_capnp {
    include!(concat!(env!("OUT_DIR"), "/response_capnp.rs"));
}

pub mod btree;
pub mod cluster;
pub mod network;
pub mod plugin;
pub mod query;
pub mod reql;
pub mod server;
pub mod storage;

// Re-exports for convenience
pub use plugin::{Plugin, PluginManager};
pub use reql::Datum;
pub use storage::{Storage, StorageEngine};

/// RethinkDB error types
pub mod error {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum Error {
        #[error("Storage error: {0}")]
        Storage(String),

        #[error("Query error: {0}")]
        Query(String),

        #[error("Network error: {0}")]
        Network(String),

        #[error("Plugin error: {0}")]
        Plugin(String),

        #[error("Internal error: {0}")]
        Internal(String),

        #[error("Not found: {0}")]
        NotFound(String),

        #[error("Already exists: {0}")]
        AlreadyExists(String),

        #[error("Invalid argument: {0}")]
        InvalidArgument(String),

        #[error("Serialization error: {0}")]
        SerializationError(String),

        #[error("Storage error: {0}")]
        StorageError(String),
    }

    pub type Result<T> = std::result::Result<T, Error>;
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_format() {
        // VERSION is a static string, always valid
        let _version: &str = VERSION;
        // Just ensure the constant is accessible
    }
}
