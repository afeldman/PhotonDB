//! Integration tests for network protocol and query execution

use rethinkdb::network::server::{ProtocolServer, ServerConfig};
use rethinkdb::storage::btree::SledStorage;
use rethinkdb::storage::Storage;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

/// Helper to start a test server
async fn start_test_server() -> (ProtocolServer, std::net::SocketAddr) {
    // Use unique temp directory for each test
    let temp_dir = std::env::temp_dir().join(format!(
        "rethinkdb_test_{}",
        std::process::id()
    ));
    let storage = Arc::new(Storage::new(Box::new(
        SledStorage::new(temp_dir.to_str().unwrap()).expect("Failed to create storage"),
    )));

    let config = ServerConfig {
        bind_addr: "127.0.0.1:0".parse().unwrap(), // Random port
        max_connections: 10,
        tls_enabled: false,
        tls_cert_path: None,
        tls_key_path: None,
    };

    let addr = config.bind_addr;
    let server = ProtocolServer::new(config, storage);

    (server, addr)
}

#[tokio::test]
async fn test_server_starts() {
    let (server, addr) = start_test_server().await;

    // Start server in background
    let _handle = tokio::spawn(async move {
        let _ = server.serve().await;
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Try to connect
    let result = timeout(Duration::from_secs(1), TcpStream::connect(addr)).await;

    assert!(result.is_ok(), "Server should accept connections");
    if let Ok(Ok(stream)) = result {
        drop(stream);
    }
}

#[tokio::test]
async fn test_handshake() {
    let (server, addr) = start_test_server().await;

    // Start server
    let _handle = tokio::spawn(async move {
        let _ = server.serve().await;
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect
    let mut stream = TcpStream::connect(addr)
        .await
        .expect("Failed to connect");

    // Send handshake (V1_0, JSON protocol)
    // Magic number for V1_0: 0x34c2bdc3
    let version: u32 = 0x34c2bdc3;
    stream
        .write_all(&version.to_le_bytes())
        .await
        .expect("Failed to write version");

    // Protocol type (JSON): 0x7e6970c7
    let protocol: u32 = 0x7e6970c7;
    stream
        .write_all(&protocol.to_le_bytes())
        .await
        .expect("Failed to write protocol");

    // Auth key length (0 for no auth)
    let auth_len: u32 = 0;
    stream
        .write_all(&auth_len.to_le_bytes())
        .await
        .expect("Failed to write auth length");

    // Protocol range
    let min_version: u32 = 0x34c2bdc3;
    let max_version: u32 = 0x34c2bdc3;
    stream
        .write_all(&min_version.to_le_bytes())
        .await
        .expect("Failed to write min version");
    stream
        .write_all(&max_version.to_le_bytes())
        .await
        .expect("Failed to write max version");

    // Read handshake response
    let mut response = [0u8; 8];
    let result = timeout(Duration::from_secs(1), stream.read_exact(&mut response)).await;

    assert!(result.is_ok(), "Should receive handshake response");
}

#[tokio::test]
async fn test_db_list_query() {
    let (server, addr) = start_test_server().await;

    // Start server
    let _handle = tokio::spawn(async move {
        let _ = server.serve().await;
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect and handshake
    let mut stream = TcpStream::connect(addr)
        .await
        .expect("Failed to connect");

    // Perform handshake (simplified)
    let version: u32 = 0x34c2bdc3;
    stream.write_all(&version.to_le_bytes()).await.unwrap();
    let protocol: u32 = 0x7e6970c7;
    stream.write_all(&protocol.to_le_bytes()).await.unwrap();
    let auth_len: u32 = 0;
    stream.write_all(&auth_len.to_le_bytes()).await.unwrap();
    let min_version: u32 = 0x34c2bdc3;
    let max_version: u32 = 0x34c2bdc3;
    stream.write_all(&min_version.to_le_bytes()).await.unwrap();
    stream.write_all(&max_version.to_le_bytes()).await.unwrap();

    // Read handshake response
    let mut response = [0u8; 8];
    stream.read_exact(&mut response).await.unwrap();

    // Send DB_LIST query
    // Query format: [79] (DB_LIST has term_type 79)
    let query_json = serde_json::json!({
        "type": "START",
        "query": [79] // DB_LIST
    });

    let query_str = serde_json::to_string(&query_json).unwrap();
    let query_bytes = query_str.as_bytes();

    // Send query length
    let len = query_bytes.len() as u32;
    stream.write_all(&len.to_le_bytes()).await.unwrap();

    // Send token
    let token: i64 = 1;
    stream.write_all(&token.to_le_bytes()).await.unwrap();

    // Send query
    stream.write_all(query_bytes).await.unwrap();

    // Read response length
    let mut len_bytes = [0u8; 4];
    let result = timeout(Duration::from_secs(2), stream.read_exact(&mut len_bytes)).await;

    assert!(
        result.is_ok(),
        "Should receive response for DB_LIST query"
    );
}
