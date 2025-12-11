//! Internal cluster communication handlers
//!
//! Endpoints for node-to-node communication:
//! - POST /internal/replicate - Receive replicated data
//! - POST /internal/read - Read data from this node

use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    routing::post,
    Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, instrument};

use super::AppState;
use crate::reql::Datum;

/// Replication request payload
#[derive(Debug, Deserialize)]
pub struct ReplicateRequest {
    /// Base64-encoded key
    pub key: String,
    /// Base64-encoded data
    pub data: String,
}

/// Read request payload
#[derive(Debug, Deserialize)]
pub struct ReadRequest {
    /// Base64-encoded key
    pub key: String,
}

/// Read response payload
#[derive(Debug, Serialize)]
pub struct ReadResponse {
    /// Base64-encoded data
    pub data: String,
}

/// Internal cluster routes
pub fn internal_routes() -> Router {
    Router::new()
        .route("/internal/replicate", post(handle_replicate))
        .route("/internal/read", post(handle_read))
}

/// Handle replication from another node
#[instrument(skip(state, req))]
async fn handle_replicate(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<ReplicateRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Decode key and data
    let key = BASE64
        .decode(&req.key)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid key encoding: {}", e)))?;

    let data = BASE64
        .decode(&req.data)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid data encoding: {}", e)))?;

    info!(
        key_size = key.len(),
        data_size = data.len(),
        "Receiving replicated data"
    );

    // Convert data to Datum (use String for now, could be enhanced)
    let datum = Datum::String(String::from_utf8_lossy(&data).to_string());

    // Store data in local storage
    match state.storage.set(&key, datum).await {
        Ok(_) => {
            info!(key_size = key.len(), "Replication successful");
            Ok(StatusCode::OK)
        }
        Err(e) => {
            error!(error = %e, "Failed to store replicated data");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Storage error: {}", e),
            ))
        }
    }
}

/// Handle read request from another node
#[instrument(skip(state, req))]
async fn handle_read(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<ReadRequest>,
) -> Result<Json<ReadResponse>, (StatusCode, String)> {
    // Decode key
    let key = BASE64
        .decode(&req.key)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid key encoding: {}", e)))?;

    info!(key_size = key.len(), "Reading data for remote node");

    // Read from local storage
    match state.storage.get(&key).await {
        Ok(Some(datum)) => {
            // Convert Datum to bytes
            let data = match &datum {
                Datum::String(s) => s.as_bytes().to_vec(),
                Datum::Number(n) => n.to_string().into_bytes(),
                Datum::Boolean(b) => b.to_string().into_bytes(),
                Datum::Null => vec![],
                Datum::Array(arr) => {
                    // Serialize array as JSON
                    match serde_json::to_vec(arr) {
                        Ok(bytes) => bytes,
                        Err(e) => {
                            error!(error = %e, "Failed to serialize array");
                            return Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Serialization error: {}", e),
                            ));
                        }
                    }
                }
                Datum::Object(obj) => {
                    // Serialize object as JSON
                    match serde_json::to_vec(obj) {
                        Ok(bytes) => bytes,
                        Err(e) => {
                            error!(error = %e, "Failed to serialize object");
                            return Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("Serialization error: {}", e),
                            ));
                        }
                    }
                }
            };
            
            info!(key_size = key.len(), data_size = data.len(), "Read successful");
            
            let response = ReadResponse {
                data: BASE64.encode(&data),
            };
            
            Ok(Json(response))
        }
        Ok(None) => {
            info!(key_size = key.len(), "Key not found");
            Err((StatusCode::NOT_FOUND, "Key not found".to_string()))
        }
        Err(e) => {
            error!(error = %e, "Failed to read data");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Storage error: {}", e),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replicate_request_deserialization() {
        let json = r#"{"key": "dGVzdA==", "data": "dmFsdWU="}"#;
        let req: ReplicateRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(req.key, "dGVzdA==");
        assert_eq!(req.data, "dmFsdWU=");
    }

    #[test]
    fn test_read_request_deserialization() {
        let json = r#"{"key": "dGVzdA=="}"#;
        let req: ReadRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(req.key, "dGVzdA==");
    }

    #[test]
    fn test_base64_encoding() {
        let key = b"test_key";
        let encoded = BASE64.encode(key);
        let decoded = BASE64.decode(&encoded).unwrap();
        
        assert_eq!(key.as_slice(), decoded.as_slice());
    }
}
