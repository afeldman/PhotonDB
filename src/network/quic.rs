//! QUIC server for RethinkDB protocol

#[cfg(feature = "quic")]
use super::connection::Connection;
#[cfg(feature = "quic")]
use super::protocol::{Handshake, ProtocolVersion, WireProtocol};
#[cfg(feature = "quic")]
use crate::storage::Storage;
#[cfg(feature = "quic")]
use anyhow::{anyhow, Result};
#[cfg(feature = "quic")]
use quinn::{Endpoint, ServerConfig};
#[cfg(feature = "quic")]
use std::net::SocketAddr;
#[cfg(feature = "quic")]
use std::sync::Arc;
#[cfg(feature = "quic")]
use tokio::sync::Semaphore;

#[cfg(feature = "quic")]
/// QUIC server configuration
#[derive(Debug, Clone)]
pub struct QuicServerConfig {
    /// Bind address
    pub bind_addr: SocketAddr,
    
    /// Maximum concurrent connections
    pub max_connections: usize,
    
    /// Server certificate path (PEM format)
    pub cert_path: Option<String>,
    
    /// Server private key path (PEM format)
    pub key_path: Option<String>,
    
    /// Auto-generate self-signed certificate for development
    pub auto_cert: bool,
}

#[cfg(feature = "quic")]
impl Default for QuicServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:28016".parse().unwrap(), // Port 28016 für QUIC
            max_connections: 1024,
            cert_path: None,
            key_path: None,
            auto_cert: true,
        }
    }
}

#[cfg(feature = "quic")]
/// RethinkDB QUIC protocol server
pub struct QuicProtocolServer {
    config: QuicServerConfig,
    storage: Arc<Storage>,
    connection_semaphore: Arc<Semaphore>,
}

#[cfg(feature = "quic")]
impl QuicProtocolServer {
    /// Create a new QUIC protocol server
    pub fn new(config: QuicServerConfig, storage: Arc<Storage>) -> Self {
        let connection_semaphore = Arc::new(Semaphore::new(config.max_connections));

        Self {
            config,
            storage,
            connection_semaphore,
        }
    }

    /// Generate a self-signed certificate for development
    fn generate_self_signed_cert() -> Result<(rustls::pki_types::CertificateDer<'static>, rustls::pki_types::PrivateKeyDer<'static>)> {
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
        let key = rustls::pki_types::PrivateKeyDer::Pkcs8(cert.key_pair.serialize_der().into());
        let cert_der = rustls::pki_types::CertificateDer::from(cert.cert);
        Ok((cert_der, key))
    }

    /// Load certificate and key from files
    fn load_certs_from_file(cert_path: &str, key_path: &str) -> Result<(Vec<rustls::pki_types::CertificateDer<'static>>, rustls::pki_types::PrivateKeyDer<'static>)> {
        #[cfg(feature = "rustls-pemfile")]
        {
            // Load certificate
            let cert_file = std::fs::File::open(cert_path)?;
            let mut cert_reader = std::io::BufReader::new(cert_file);
            let certs = rustls_pemfile::certs(&mut cert_reader)
                .collect::<Result<Vec<_>, _>>()?;

            if certs.is_empty() {
                return Err(anyhow!("No certificates found in {}", cert_path));
            }

            // Load private key
            let key_file = std::fs::File::open(key_path)?;
            let mut key_reader = std::io::BufReader::new(key_file);
            
            let key = rustls_pemfile::private_key(&mut key_reader)?
                .ok_or_else(|| anyhow!("No private key found in {}", key_path))?;

            Ok((certs, key))
        }
        
        #[cfg(not(feature = "rustls-pemfile"))]
        {
            let _ = (cert_path, key_path);
            Err(anyhow!("rustls-pemfile feature not enabled"))
        }
    }

    /// Create server configuration with certificates
    fn create_server_config(&self) -> Result<ServerConfig> {
        let (certs, key) = if let (Some(cert_path), Some(key_path)) = (&self.config.cert_path, &self.config.key_path) {
            // Load from files
            tracing::info!("Loading certificates from {} and {}", cert_path, key_path);
            Self::load_certs_from_file(cert_path, key_path)?
        } else if self.config.auto_cert {
            // Generate self-signed certificate
            tracing::warn!("Generating self-signed certificate for development (DO NOT use in production!)");
            let (cert, key) = Self::generate_self_signed_cert()?;
            (vec![cert], key)
        } else {
            return Err(anyhow!("No certificate configuration provided"));
        };

        let mut crypto = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;

        // Enable ALPN for RethinkDB protocol
        crypto.alpn_protocols = vec![b"rethinkdb".to_vec()];

        let mut server_config = ServerConfig::with_crypto(Arc::new(
            quinn::crypto::rustls::QuicServerConfig::try_from(crypto)?
        ));

        // Configure transport parameters
        let mut transport = quinn::TransportConfig::default();
        transport.max_concurrent_bidi_streams(256u32.into());
        transport.max_concurrent_uni_streams(256u32.into());
        transport.max_idle_timeout(Some(std::time::Duration::from_secs(30).try_into()?));
        
        server_config.transport_config(Arc::new(transport));

        Ok(server_config)
    }

    /// Start the QUIC server
    pub async fn serve(&self) -> Result<()> {
        let server_config = self.create_server_config()?;
        let endpoint = Endpoint::server(server_config, self.config.bind_addr)?;

        tracing::info!(
            "RethinkDB QUIC protocol server listening on {}",
            self.config.bind_addr
        );

        loop {
            // Accept incoming connections
            let Some(connecting) = endpoint.accept().await else {
                break;
            };

            // Acquire connection permit
            let permit = match self.connection_semaphore.clone().try_acquire_owned() {
                Ok(p) => p,
                Err(_) => {
                    tracing::warn!("Max connections reached, rejecting new connection");
                    continue;
                }
            };

            let storage = self.storage.clone();
            
            tokio::spawn(async move {
                match connecting.await {
                    Ok(connection) => {
                        let remote = connection.remote_address();
                        tracing::info!("New QUIC connection from {}", remote);
                        
                        if let Err(e) = Self::handle_connection(connection, storage).await {
                            tracing::error!("QUIC connection error from {}: {}", remote, e);
                        }
                        
                        tracing::info!("QUIC connection closed from {}", remote);
                    }
                    Err(e) => {
                        tracing::error!("Failed to establish QUIC connection: {}", e);
                    }
                }
                
                // Permit automatically released when dropped
                drop(permit);
            });
        }

        Ok(())
    }

    /// Handle a single QUIC connection
    async fn handle_connection(connection: quinn::Connection, storage: Arc<Storage>) -> Result<()> {
        // Create handshake (simplified for QUIC - TLS already done)
        let handshake = Handshake {
            version: ProtocolVersion::V1_0,
            protocol: WireProtocol::Json,
            auth_key: None, // Auth via TLS client certs or first message
        };

        let conn = Connection::new(handshake, storage);

        // Accept bi-directional streams
        loop {
            match connection.accept_bi().await {
                Ok((mut send, mut recv)) => {
                    // Read query from stream
                    let query_result = recv.read_to_end(1024 * 1024).await;
                    
                    let query_buf = match query_result {
                        Ok(buf) => buf,
                        Err(e) => {
                            tracing::error!("Failed to read query: {}", e);
                            continue;
                        }
                    };

                    // Parse query
                    if query_buf.len() < 8 {
                        tracing::error!("Query too short");
                        continue;
                    }

                    let token = match query_buf[0..8].try_into() {
                        Ok(bytes) => i64::from_le_bytes(bytes),
                        Err(e) => {
                            tracing::error!("Failed to parse token: {}", e);
                            continue;
                        }
                    };

                    let query_json: serde_json::Value = match serde_json::from_slice(&query_buf[8..]) {
                        Ok(json) => json,
                        Err(e) => {
                            tracing::error!("Failed to parse query JSON: {}", e);
                            continue;
                        }
                    };

                    let query_msg = super::protocol::QueryMessage {
                        token,
                        query: query_json,
                    };

                    // Handle query
                    match conn.handle_query(query_msg).await {
                        Ok(response) => {
                            // Write response
                            let response_json = match serde_json::to_vec(&response.response) {
                                Ok(json) => json,
                                Err(e) => {
                                    tracing::error!("Failed to serialize response: {}", e);
                                    continue;
                                }
                            };
                            
                            let mut response_buf = Vec::with_capacity(8 + response_json.len());
                            response_buf.extend_from_slice(&response.token.to_le_bytes());
                            response_buf.extend_from_slice(&response_json);

                            if let Err(e) = send.write_all(&response_buf).await {
                                tracing::error!("Failed to write response: {}", e);
                                break;
                            }
                            
                            if let Err(e) = send.finish() {
                                tracing::error!("Failed to finish stream: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!("Query execution error: {}", e);
                            // Send error response
                            let error_response = serde_json::json!({
                                "t": 18, // RUNTIME_ERROR
                                "r": [],
                                "e": 1000000,
                                "m": e.to_string()
                            });
                            
                            if let Ok(response_json) = serde_json::to_vec(&error_response) {
                                let mut response_buf = Vec::with_capacity(8 + response_json.len());
                                response_buf.extend_from_slice(&token.to_le_bytes());
                                response_buf.extend_from_slice(&response_json);

                                let _ = send.write_all(&response_buf).await;
                                let _ = send.finish();
                            }
                        }
                    }
                }
                Err(quinn::ConnectionError::ApplicationClosed(_)) => {
                    tracing::debug!("Client closed connection gracefully");
                    break;
                }
                Err(e) => {
                    tracing::error!("Failed to accept stream: {}", e);
                    break;
                }
            }
        }

        Ok(())
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
#[cfg(feature = "quic")]
mod tests {
    use super::*;

    #[test]
    fn test_quic_config_default() {
        let config = QuicServerConfig::default();
        assert_eq!(config.bind_addr.port(), 28016);
        assert_eq!(config.max_connections, 1024);
        assert!(config.auto_cert);
    }

    #[test]
    fn test_self_signed_cert_generation() {
        let result = QuicProtocolServer::generate_self_signed_cert();
        assert!(result.is_ok());
    }
}
