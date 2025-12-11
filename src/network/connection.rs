//! TCP connection management for RethinkDB protocol.
//!
//! Handles the complete lifecycle of a RethinkDB client connection:
//!
//! 1. **Handshake**: Protocol version and authentication
//! 2. **Query Loop**: Receive queries, execute, send responses
//! 3. **Error Handling**: Graceful error responses
//!
//! # Query Types
//!
//! - **START**: Execute a new query
//! - **CONTINUE**: Fetch more results from a cursor
//! - **STOP**: Cancel an ongoing query
//! - **NOREPLY_WAIT**: Wait for all noreply queries to complete
//! - **SERVER_INFO**: Get server information
//!
//! # Architecture
//!
//! ```text
//! Client → TCP → Handshake → Connection → QueryCompiler → QueryExecutor → Storage
//!                               ↓              ↓              ↓             ↓
//!                            Protocol      JSON→AST      Execute AST    Sled DB
//!                            Handling       Parse         Operations     CRUD
//! ```

use super::protocol::{
    read_query, write_response, Handshake, ProtocolVersion, QueryMessage, ResponseMessage,
    WireProtocol,
};
use crate::query::compiler::QueryCompiler;
use crate::query::executor::QueryExecutor;
use crate::storage::Storage;
use anyhow::{anyhow, Result};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Connection state
#[derive(Debug)]
pub struct Connection {
    handshake: Handshake,
    executor: Arc<QueryExecutor>,
    active_queries: Arc<Mutex<std::collections::HashMap<i64, tokio::sync::oneshot::Sender<()>>>>,
}

impl Connection {
    /// Create a new connection after handshake
    pub fn new(handshake: Handshake, storage: Arc<Storage>) -> Self {
        Self {
            handshake,
            executor: Arc::new(QueryExecutor::new(storage)),
            active_queries: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Get protocol version
    pub fn version(&self) -> ProtocolVersion {
        self.handshake.version
    }

    /// Get wire protocol
    pub fn protocol(&self) -> WireProtocol {
        self.handshake.protocol
    }

    /// Check if connection is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.handshake.auth_key.is_some()
    }

    /// Get auth key if present
    pub fn auth_key(&self) -> Option<&str> {
        self.handshake.auth_key.as_deref()
    }

    /// Handle a single query
    pub async fn handle_query(&self, query: QueryMessage) -> Result<ResponseMessage> {
        let start = std::time::Instant::now();
        let query_type = query
            .query
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing query type"))?
            .to_string();
        
        tracing::debug!(
            token = query.token,
            query_type = %query_type,
            "Processing query"
        );

        let result = match query_type.as_str() {
            "START" => self.handle_start_query(query).await,
            "CONTINUE" => self.handle_continue_query(query).await,
            "STOP" => self.handle_stop_query(query).await,
            "NOREPLY_WAIT" => self.handle_noreply_wait(query).await,
            "SERVER_INFO" => self.handle_server_info(query).await,
            _ => Err(anyhow!("Unknown query type: {}", query_type)),
        };

        let elapsed = start.elapsed();
        tracing::debug!(
            query_type = %query_type,
            elapsed_ms = elapsed.as_millis(),
            success = result.is_ok(),
            "Query completed"
        );

        result
    }

    /// Handle START query
    async fn handle_start_query(&self, query: QueryMessage) -> Result<ResponseMessage> {
        let query_term = query
            .query
            .get("query")
            .ok_or_else(|| anyhow!("Missing query term"))?;

        // Compile JSON query to AST
        tracing::trace!("Compiling query to AST");
        let ast_term = QueryCompiler::compile(query_term)
            .map_err(|e| anyhow!("Query compilation failed: {}", e))?;

        // Execute query through executor
        tracing::trace!(term_type = ?ast_term.term_type, "Executing query");
        let result = self.executor.execute(&ast_term).await
            .map_err(|e| anyhow!("Query execution failed: {}", e))?;

        // Convert result back to JSON
        let result_json = QueryCompiler::datum_to_json(&result);
        tracing::trace!("Query executed successfully, returning result");

        Ok(ResponseMessage {
            token: query.token,
            response: serde_json::json!({
                "t": 1, // SUCCESS_ATOM
                "r": [result_json]
            }),
        })
    }

    /// Handle CONTINUE query (fetch more results)
    async fn handle_continue_query(&self, query: QueryMessage) -> Result<ResponseMessage> {
        // TODO: Implement cursor continuation
        Ok(ResponseMessage {
            token: query.token,
            response: serde_json::json!({
                "t": 2, // SUCCESS_SEQUENCE
                "r": []
            }),
        })
    }

    /// Handle STOP query (cancel ongoing query)
    async fn handle_stop_query(&self, query: QueryMessage) -> Result<ResponseMessage> {
        let mut queries = self.active_queries.lock().await;
        if let Some(cancel_tx) = queries.remove(&query.token) {
            let _ = cancel_tx.send(());
            tracing::debug!("Cancelled query token={}", query.token);
        }

        Ok(ResponseMessage {
            token: query.token,
            response: serde_json::json!({
                "t": 2, // SUCCESS_SEQUENCE
                "r": []
            }),
        })
    }

    /// Handle NOREPLY_WAIT (wait for all noreply queries to complete)
    async fn handle_noreply_wait(&self, query: QueryMessage) -> Result<ResponseMessage> {
        // TODO: Track noreply queries
        Ok(ResponseMessage {
            token: query.token,
            response: serde_json::json!({
                "t": 3, // WAIT_COMPLETE
                "r": []
            }),
        })
    }

    /// Handle SERVER_INFO query
    async fn handle_server_info(&self, query: QueryMessage) -> Result<ResponseMessage> {
        Ok(ResponseMessage {
            token: query.token,
            response: serde_json::json!({
                "t": 4, // SERVER_INFO
                "r": [{
                    "id": "rethinkdb-3.0",
                    "name": "PhotonDB",
                    "version": env!("CARGO_PKG_VERSION"),
                }]
            }),
        })
    }
}

/// Connection handler for TCP streams
pub struct ConnectionHandler {
    storage: Arc<Storage>,
}

impl ConnectionHandler {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    /// Handle a new TCP connection
    pub async fn handle(&self, mut stream: TcpStream) -> Result<()> {
        let peer_addr = stream.peer_addr()?;
        tracing::info!("New connection from {}", peer_addr);

        // Perform handshake
        let handshake = match Handshake::accept(&mut stream).await {
            Ok(h) => h,
            Err(e) => {
                tracing::error!("Handshake failed from {}: {}", peer_addr, e);
                return Err(e);
            }
        };

        // Create connection state
        let connection = Connection::new(handshake, self.storage.clone());
        tracing::info!("Connection established from {} (authenticated: {})", 
            peer_addr, connection.is_authenticated());

        // Query/response loop
        loop {
            match read_query(&mut stream).await {
                Ok(query) => {
                    let token = query.token;
                    match connection.handle_query(query).await {
                        Ok(response) => {
                            if let Err(e) = write_response(&mut stream, &response).await {
                                tracing::error!("Failed to write response: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!("Query execution error: {}", e);
                            // Send error response
                            let error_response = ResponseMessage {
                                token,
                                response: serde_json::json!({
                                    "t": 18, // RUNTIME_ERROR
                                    "r": [],
                                    "e": 1000000, // INTERNAL
                                    "b": [],
                                    "m": e.to_string()
                                }),
                            };
                            if let Err(e) = write_response(&mut stream, &error_response).await {
                                tracing::error!("Failed to write error response: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    if e.to_string().contains("UnexpectedEof") {
                        tracing::info!("Client disconnected: {}", peer_addr);
                    } else {
                        tracing::error!("Failed to read query: {}", e);
                    }
                    break;
                }
            }
        }

        tracing::info!("Connection closed from {}", peer_addr);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_creation() {
        use crate::storage::slab::SlabStorageEngine;

        let temp_dir = std::env::temp_dir().join(format!("connection_test_{}", std::process::id()));
        let storage = Arc::new(Storage::new(Box::new(SlabStorageEngine::with_defaults(&temp_dir).unwrap())));
        let handshake = Handshake {
            version: ProtocolVersion::V1_0,
            protocol: WireProtocol::Json,
            auth_key: Some("test_key".to_string()),
        };

        let conn = Connection::new(handshake, storage);
        assert_eq!(conn.version(), ProtocolVersion::V1_0);
        assert_eq!(conn.protocol(), WireProtocol::Json);
        assert!(conn.is_authenticated());
        assert_eq!(conn.auth_key(), Some("test_key"));
    }

    #[tokio::test]
    async fn test_execute_query() {
        use crate::storage::slab::SlabStorageEngine;

        let temp_dir = std::env::temp_dir().join(format!("query_test_{}", std::process::id()));
        let storage = Arc::new(Storage::new(Box::new(SlabStorageEngine::with_defaults(&temp_dir).unwrap())));
        let handshake = Handshake {
            version: ProtocolVersion::V1_0,
            protocol: WireProtocol::Json,
            auth_key: None,
        };

        let conn = Connection::new(handshake, storage);

        let query = QueryMessage {
            token: 1,
            query: serde_json::json!({
                "type": "SERVER_INFO"
            }),
        };

        let response = conn.handle_query(query).await.unwrap();
        assert_eq!(response.token, 1);
        assert_eq!(response.response["t"], 4); // SERVER_INFO
    }
}
