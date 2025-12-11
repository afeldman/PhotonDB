//! Network protocol handling
//!
//! This module implements the RethinkDB wire protocol for client-server communication.
//! It supports the V1_0 protocol with JSON encoding over TCP and optionally QUIC.
//!
//! ## Protocol Flow
//!
//! 1. **Handshake**: Client sends version, auth key, and protocol type
//! 2. **Authentication**: Server validates credentials (bcrypt or certificate)
//! 3. **Query/Response Loop**: Client sends queries, server responds
//!
//! ## Features
//!
//! - Multiple protocol versions (V0_1 to V1_0)
//! - JSON wire protocol (Protobuf deprecated)
//! - TCP transport (default, port 28015)
//! - QUIC transport (optional, port 28016, feature = "quic")
//! - Connection pooling with max connection limits
//! - Authentication: bcrypt password hashing + TLS certificates
//! - Parallel query execution (V0_4+)

pub mod auth;
pub mod connection;
pub mod protocol;
pub mod server;

#[cfg(feature = "quic")]
pub mod quic;

pub use auth::{AuthManager, Permission, User};
pub use connection::{Connection, ConnectionHandler};
pub use protocol::{
    Handshake, ProtocolVersion, QueryMessage, ResponseMessage, WireProtocol,
    VERSION_V1_0, PROTOCOL_JSON,
};
pub use server::{ProtocolServer, ServerConfig};

#[cfg(feature = "quic")]
pub use quic::{QuicProtocolServer, QuicServerConfig};
