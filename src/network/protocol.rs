//! RethinkDB Wire Protocol Implementation
//!
//! Implements the RethinkDB client protocol with handshake and query/response cycles.
//! Based on the original C++ implementation and Cap'n Proto schemas.

use anyhow::{anyhow, Result};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Protocol version constants (from handshake.capnp)
pub const VERSION_V0_1: u32 = 0x3f61ba36;
pub const VERSION_V0_2: u32 = 0x723081e1;
pub const VERSION_V0_3: u32 = 0x5f75e83e;
pub const VERSION_V0_4: u32 = 0x400c2d20;
pub const VERSION_V1_0: u32 = 0x34c2bdc3;

/// Protocol type constants
pub const PROTOCOL_PROTOBUF: u32 = 0x271ffc41;
pub const PROTOCOL_JSON: u32 = 0x7e6970c7;

/// Size limits
pub const HARD_LIMIT_TOO_LARGE_QUERY_SIZE: u32 = 1024 * 1024 * 1024; // 1 GB
pub const TOO_LARGE_QUERY_SIZE: u32 = 128 * 1024 * 1024; // 128 MB
pub const TOO_LONG_QUERY_TIME: u32 = 5 * 60 * 1000; // 5 minutes in ms
pub const MAX_MESSAGE_SIZE: u32 = 256 * 1024 * 1024; // 256 MB

/// Protocol version information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolVersion {
    V0_1,
    V0_2,
    V0_3,
    V0_4,
    V1_0,
}

impl ProtocolVersion {
    pub fn from_magic(magic: u32) -> Result<Self> {
        match magic {
            VERSION_V0_1 => Ok(ProtocolVersion::V0_1),
            VERSION_V0_2 => Ok(ProtocolVersion::V0_2),
            VERSION_V0_3 => Ok(ProtocolVersion::V0_3),
            VERSION_V0_4 => Ok(ProtocolVersion::V0_4),
            VERSION_V1_0 => Ok(ProtocolVersion::V1_0),
            _ => Err(anyhow!("Unsupported protocol version: 0x{:x}", magic)),
        }
    }

    pub fn to_magic(self) -> u32 {
        match self {
            ProtocolVersion::V0_1 => VERSION_V0_1,
            ProtocolVersion::V0_2 => VERSION_V0_2,
            ProtocolVersion::V0_3 => VERSION_V0_3,
            ProtocolVersion::V0_4 => VERSION_V0_4,
            ProtocolVersion::V1_0 => VERSION_V1_0,
        }
    }

    pub fn supports_json(&self) -> bool {
        matches!(self, ProtocolVersion::V0_3 | ProtocolVersion::V0_4 | ProtocolVersion::V1_0)
    }

    pub fn supports_parallel_queries(&self) -> bool {
        matches!(self, ProtocolVersion::V0_4 | ProtocolVersion::V1_0)
    }

    pub fn supports_auth(&self) -> bool {
        matches!(self, ProtocolVersion::V1_0)
    }
}

/// Wire protocol type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireProtocol {
    Json,
    Protobuf,
}

impl WireProtocol {
    pub fn from_magic(magic: u32) -> Result<Self> {
        match magic {
            PROTOCOL_JSON => Ok(WireProtocol::Json),
            PROTOCOL_PROTOBUF => Ok(WireProtocol::Protobuf),
            _ => Err(anyhow!("Unknown protocol type: 0x{:x}", magic)),
        }
    }

    pub fn to_magic(self) -> u32 {
        match self {
            WireProtocol::Json => PROTOCOL_JSON,
            WireProtocol::Protobuf => PROTOCOL_PROTOBUF,
        }
    }
}

/// Handshake state
#[derive(Debug)]
pub struct Handshake {
    pub version: ProtocolVersion,
    pub protocol: WireProtocol,
    pub auth_key: Option<String>,
}

impl Handshake {
    /// Perform server-side handshake
    pub async fn accept<T>(stream: &mut T) -> Result<Self>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        // 1. Read version magic number (4 bytes, little-endian)
        let version_magic = stream.read_u32_le().await?;
        let version = ProtocolVersion::from_magic(version_magic)?;

        tracing::debug!(
            "Client protocol version: {:?} (0x{:x})",
            version,
            version_magic
        );

        // 2. Read auth key (for V0_2 and later)
        let auth_key = if version != ProtocolVersion::V0_1 {
            let key_len = stream.read_u32_le().await?;
            if key_len > 4096 {
                return Err(anyhow!("Auth key too long: {} bytes", key_len));
            }

            let mut key_bytes = vec![0u8; key_len as usize];
            stream.read_exact(&mut key_bytes).await?;
            
            // Remove null terminator if present
            if key_bytes.last() == Some(&0) {
                key_bytes.pop();
            }
            
            Some(String::from_utf8(key_bytes)?)
        } else {
            None
        };

        // 3. Read protocol type (for V0_3 and later)
        let protocol = if version.supports_json() {
            let protocol_magic = stream.read_u32_le().await?;
            WireProtocol::from_magic(protocol_magic)?
        } else {
            // V0_1 and V0_2 only supported Protobuf (now deprecated)
            return Err(anyhow!("PROTOBUF protocol is no longer supported"));
        };

        tracing::debug!("Client wire protocol: {:?}", protocol);

        // 4. Send success response
        let success_msg = if version == ProtocolVersion::V1_0 {
            // V1_0 sends JSON response with version info
            serde_json::json!({
                "success": true,
                "min_protocol_version": 0,
                "max_protocol_version": 0,
                "server_version": env!("CARGO_PKG_VERSION")
            })
            .to_string()
        } else {
            "SUCCESS".to_string()
        };

        stream.write_all(success_msg.as_bytes()).await?;
        stream.write_all(b"\0").await?; // Null terminator
        stream.flush().await?;

        tracing::info!("Handshake complete: {:?} / {:?}", version, protocol);

        Ok(Handshake {
            version,
            protocol,
            auth_key,
        })
    }

    /// Perform client-side handshake
    pub async fn connect<T>(
        stream: &mut T,
        auth_key: Option<String>,
        version: ProtocolVersion,
        protocol: WireProtocol,
    ) -> Result<()>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        // 1. Send version magic number
        stream.write_u32_le(version.to_magic()).await?;

        // 2. Send auth key (for V0_2 and later)
        if version != ProtocolVersion::V0_1 {
            let key = auth_key.as_deref().unwrap_or("");
            stream.write_u32_le(key.len() as u32).await?;
            stream.write_all(key.as_bytes()).await?;
        }

        // 3. Send protocol type (for V0_3 and later)
        if version.supports_json() {
            stream.write_u32_le(protocol.to_magic()).await?;
        }

        stream.flush().await?;

        // 4. Read server response
        let mut response = Vec::new();
        loop {
            let byte = stream.read_u8().await?;
            if byte == 0 {
                break;
            }
            response.push(byte);
            if response.len() > 1024 {
                return Err(anyhow!("Handshake response too long"));
            }
        }

        let response_str = String::from_utf8(response)?;

        if version == ProtocolVersion::V1_0 {
            // Parse JSON response
            let response_json: serde_json::Value = serde_json::from_str(&response_str)?;
            if response_json.get("success") != Some(&serde_json::Value::Bool(true)) {
                return Err(anyhow!("Handshake failed: {}", response_str));
            }
        } else if response_str != "SUCCESS" {
            return Err(anyhow!("Handshake failed: {}", response_str));
        }

        tracing::info!("Client handshake complete");
        Ok(())
    }
}

/// Query message with token
#[derive(Debug, Clone)]
pub struct QueryMessage {
    pub token: i64,
    pub query: serde_json::Value,
}

/// Response message
#[derive(Debug, Clone)]
pub struct ResponseMessage {
    pub token: i64,
    pub response: serde_json::Value,
}

/// Read a query message from the stream (server side)
pub async fn read_query<T>(stream: &mut T) -> Result<QueryMessage>
where
    T: AsyncRead + Unpin,
{
    // Read message size (4 bytes, little-endian)
    let size = stream.read_u32_le().await?;

    if size == 0 {
        return Err(anyhow!("Empty query message"));
    }

    if size > MAX_MESSAGE_SIZE {
        return Err(anyhow!(
            "Query too large: {} bytes (max: {})",
            size,
            MAX_MESSAGE_SIZE
        ));
    }

    // Read query token (8 bytes, little-endian)
    let token = stream.read_i64_le().await?;

    // Read query data
    let mut buffer = vec![0u8; size as usize];
    stream.read_exact(&mut buffer).await?;

    // Parse JSON query
    let query: serde_json::Value = serde_json::from_slice(&buffer)?;

    Ok(QueryMessage { token, query })
}

/// Write a response message to the stream (server side)
pub async fn write_response<T>(stream: &mut T, msg: &ResponseMessage) -> Result<()>
where
    T: AsyncWrite + Unpin,
{
    // Serialize response to JSON
    let response_json = serde_json::to_vec(&msg.response)?;

    // Write token (8 bytes, little-endian)
    stream.write_i64_le(msg.token).await?;

    // Write response size (4 bytes, little-endian)
    stream.write_u32_le(response_json.len() as u32).await?;

    // Write response data
    stream.write_all(&response_json).await?;
    stream.flush().await?;

    Ok(())
}

/// Read a response message from the stream (client side)
pub async fn read_response<T>(stream: &mut T) -> Result<ResponseMessage>
where
    T: AsyncRead + Unpin,
{
    // Read token (8 bytes, little-endian)
    let token = stream.read_i64_le().await?;

    // Read response size (4 bytes, little-endian)
    let size = stream.read_u32_le().await?;

    if size > MAX_MESSAGE_SIZE {
        return Err(anyhow!(
            "Response too large: {} bytes (max: {})",
            size,
            MAX_MESSAGE_SIZE
        ));
    }

    // Read response data
    let mut buffer = vec![0u8; size as usize];
    stream.read_exact(&mut buffer).await?;

    // Parse JSON response
    let response: serde_json::Value = serde_json::from_slice(&buffer)?;

    Ok(ResponseMessage { token, response })
}

/// Write a query message to the stream (client side)
pub async fn write_query<T>(stream: &mut T, msg: &QueryMessage) -> Result<()>
where
    T: AsyncWrite + Unpin,
{
    // Serialize query to JSON
    let query_json = serde_json::to_vec(&msg.query)?;

    // Write message size (4 bytes, little-endian)
    stream.write_u32_le(query_json.len() as u32).await?;

    // Write token (8 bytes, little-endian)
    stream.write_i64_le(msg.token).await?;

    // Write query data
    stream.write_all(&query_json).await?;
    stream.flush().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_protocol_version_magic() {
        assert_eq!(ProtocolVersion::V1_0.to_magic(), VERSION_V1_0);
        assert_eq!(
            ProtocolVersion::from_magic(VERSION_V1_0).unwrap(),
            ProtocolVersion::V1_0
        );
    }

    #[test]
    fn test_wire_protocol_magic() {
        assert_eq!(WireProtocol::Json.to_magic(), PROTOCOL_JSON);
        assert_eq!(
            WireProtocol::from_magic(PROTOCOL_JSON).unwrap(),
            WireProtocol::Json
        );
    }

    #[test]
    fn test_version_features() {
        assert!(ProtocolVersion::V1_0.supports_json());
        assert!(ProtocolVersion::V1_0.supports_parallel_queries());
        assert!(ProtocolVersion::V1_0.supports_auth());

        assert!(!ProtocolVersion::V0_1.supports_json());
        assert!(!ProtocolVersion::V0_2.supports_parallel_queries());
    }

    #[tokio::test]
    async fn test_query_message_roundtrip() {
        let msg = QueryMessage {
            token: 42,
            query: serde_json::json!({
                "type": "START",
                "query": [1, ["table", ["db", "test"]]]
            }),
        };

        let mut buffer = Vec::new();
        write_query(&mut buffer, &msg).await.unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = read_query(&mut cursor).await.unwrap();

        assert_eq!(decoded.token, msg.token);
        assert_eq!(decoded.query, msg.query);
    }

    #[tokio::test]
    async fn test_response_message_roundtrip() {
        let msg = ResponseMessage {
            token: 123,
            response: serde_json::json!({
                "t": 1,
                "r": [{"id": 1, "name": "test"}]
            }),
        };

        let mut buffer = Vec::new();
        write_response(&mut buffer, &msg).await.unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = read_response(&mut cursor).await.unwrap();

        assert_eq!(decoded.token, msg.token);
        assert_eq!(decoded.response, msg.response);
    }
}
