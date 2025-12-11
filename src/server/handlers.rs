//! HTTP route handlers

use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, instrument};

use crate::reql::Datum;
use crate::server::AppState;

/// Query request
#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub query: String,
    #[serde(default)]
    pub options: QueryOptions,
}

#[derive(Debug, Deserialize, Default)]
pub struct QueryOptions {
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub batch_size: Option<usize>,
}

/// Query response
#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<Datum>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// Execute ReQL query
#[instrument(skip(state, payload))]
pub async fn execute_query(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<QueryRequest>,
) -> Response {
    info!(query = %payload.query, "Executing query");

    let start = std::time::Instant::now();

    // Parse query string to JSON Value
    let query_value: serde_json::Value = match serde_json::from_str(&payload.query) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(QueryResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Invalid query JSON: {}", e)),
                    execution_time_ms: 0,
                }),
            ).into_response();
        }
    };

    // Compile query to AST
    let term = match crate::query::QueryCompiler::compile(&query_value) {
        Ok(t) => t,
        Err(e) => {
            let duration = start.elapsed();
            return (
                StatusCode::BAD_REQUEST,
                Json(QueryResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Query compilation error: {}", e)),
                    execution_time_ms: duration.as_millis() as u64,
                }),
            ).into_response();
        }
    };

    // Execute query
    match state.executor.execute(&term).await {
        Ok(results) => {
            let duration = start.elapsed();
            info!(duration_ms = duration.as_millis(), "Query completed");

            Json(QueryResponse {
                success: true,
                data: Some(vec![Datum::String(results.to_string())]),
                error: None,
                execution_time_ms: duration.as_millis() as u64,
            })
            .into_response()
        }
        Err(e) => {
            let duration = start.elapsed();
            error!(
                error = %e,
                duration_ms = duration.as_millis(),
                "Query failed"
            );

            (
                StatusCode::BAD_REQUEST,
                Json(QueryResponse {
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                    execution_time_ms: duration.as_millis() as u64,
                }),
            )
                .into_response()
        }
    }
}

/// List all tables
#[instrument(skip(state))]
pub async fn list_tables(Extension(state): Extension<Arc<AppState>>) -> Response {
    info!("Listing all tables");

    match state.storage.list_tables().await {
        Ok(tables) => Json(serde_json::json!({
            "success": true,
            "tables": tables,
        }))
        .into_response(),
        Err(e) => {
            error!(error = %e, "Failed to list tables");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "error": e.to_string(),
                })),
            )
                .into_response()
        }
    }
}

/// Get table info
#[instrument(skip(state))]
pub async fn get_table_info(
    Extension(state): Extension<Arc<AppState>>,
    Path(table_name): Path<String>,
) -> Response {
    info!(table = %table_name, "Getting table info");

    match state.storage.get_table_info(&table_name).await {
        Ok(info) => Json(serde_json::json!({
            "success": true,
            "table": info,
        }))
        .into_response(),
        Err(e) => {
            error!(error = %e, table = %table_name, "Failed to get table info");
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "success": false,
                    "error": e.to_string(),
                })),
            )
                .into_response()
        }
    }
}

/// Health check
pub async fn health_check() -> Response {
    Json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
    .into_response()
}

/// Metrics endpoint (Prometheus format)
pub async fn metrics() -> Response {
    // TODO: Implement Prometheus metrics
    "# PhotonDB Metrics\n".into_response()
}
