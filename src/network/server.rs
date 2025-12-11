//! TCP server for RethinkDB protocol

use super::connection::ConnectionHandler;
use crate::storage::Storage;
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Bind address
    pub bind_addr: SocketAddr,
    
    /// Maximum concurrent connections
    pub max_connections: usize,
    
    /// Enable TLS
    pub tls_enabled: bool,
    
    /// TLS certificate path
    pub tls_cert_path: Option<String>,
    
    /// TLS key path
    pub tls_key_path: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:28015".parse().unwrap(),
            max_connections: 1024,
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}

/// RethinkDB protocol server
pub struct ProtocolServer {
    config: ServerConfig,
    handler: Arc<ConnectionHandler>,
    connection_semaphore: Arc<Semaphore>,
}

impl ProtocolServer {
    /// Create a new protocol server
    pub fn new(config: ServerConfig, storage: Arc<Storage>) -> Self {
        let handler = Arc::new(ConnectionHandler::new(storage));
        let connection_semaphore = Arc::new(Semaphore::new(config.max_connections));

        Self {
            config,
            handler,
            connection_semaphore,
        }
    }

    /// Start the server
    pub async fn serve(&self) -> Result<()> {
        let listener = TcpListener::bind(self.config.bind_addr).await?;
        tracing::info!(
            "RethinkDB protocol server listening on {}",
            self.config.bind_addr
        );

        loop {
            // Acquire connection permit
            let permit = self.connection_semaphore.clone().acquire_owned().await?;

            match listener.accept().await {
                Ok((stream, addr)) => {
                    let handler = self.handler.clone();
                    
                    tokio::spawn(async move {
                        tracing::debug!("Accepted connection from {}", addr);
                        
                        if let Err(e) = handler.handle(stream).await {
                            tracing::error!("Connection error from {}: {}", addr, e);
                        }
                        
                        // Permit automatically released when dropped
                        drop(permit);
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to accept connection: {}", e);
                    // Don't break the loop, keep accepting new connections
                }
            }
        }
    }

    /// Get server address
    pub fn addr(&self) -> SocketAddr {
        self.config.bind_addr
    }

    /// Get max connections
    pub fn max_connections(&self) -> usize {
        self.config.max_connections
    }

    /// Get available connection slots
    pub fn available_connections(&self) -> usize {
        self.connection_semaphore.available_permits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::slab::SlabStorageEngine;

    #[tokio::test]
    async fn test_server_creation() {
        let temp_dir = std::env::temp_dir().join(format!("rethinkdb_test_{}", std::process::id()));
        let storage = Arc::new(Storage::new(Box::new(
            SlabStorageEngine::with_defaults(temp_dir.to_str().unwrap()).expect("Failed to create storage")
        )));
        let config = ServerConfig::default();
        
        let server = ProtocolServer::new(config.clone(), storage);
        assert_eq!(server.addr(), config.bind_addr);
        assert_eq!(server.max_connections(), config.max_connections);
    }

    #[tokio::test]
    async fn test_connection_limit() {
        let temp_dir = std::env::temp_dir().join(format!("rethinkdb_test2_{}", std::process::id()));
        let storage = Arc::new(Storage::new(Box::new(
            SlabStorageEngine::with_defaults(temp_dir.to_str().unwrap()).expect("Failed to create storage")
        )));
        let config = ServerConfig {
            max_connections: 5,
            ..Default::default()
        };
        
        let server = ProtocolServer::new(config, storage);
        assert_eq!(server.available_connections(), 5);
    }
}
