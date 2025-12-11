//! Query execution engine

pub mod compiler;
pub mod executor;

pub use compiler::QueryCompiler;
pub use executor::QueryExecutor;

use crate::error::Result;
use crate::storage::Storage;
use serde_json::Value;
use std::sync::Arc;
use tracing::{info, instrument};

/// Execute a ReQL query from JSON
#[instrument(skip(storage, query))]
pub async fn execute_json(storage: Arc<Storage>, query: &Value) -> Result<Value> {
    info!("Executing JSON query");
    
    // Parse query
    let term = QueryCompiler::compile(query)
        .map_err(|e| crate::error::Error::Query(e.to_string()))?;
    
    // Execute term
    let executor = QueryExecutor::new(storage);
    let result = executor.execute(&term).await
        .map_err(|e| crate::error::Error::Query(e.to_string()))?;
    
    // Convert result to JSON
    Ok(QueryCompiler::datum_to_json(&result))
}

/// Execute a ReQL query string (legacy interface)
#[instrument(skip(storage, query))]
pub async fn execute(storage: Arc<Storage>, query: &str) -> Result<Value> {
    info!(query = %query, "Executing query string");

    // For now, simple parsing - TODO: Implement full ReQL string parser
    if query.contains("db_list") {
        return Ok(serde_json::json!(["test"]));
    }

    if query.contains("table_list") {
        let tables = storage.list_tables().await?;
        return Ok(serde_json::json!(tables));
    }

    if query.contains("table(") {
        return Ok(serde_json::json!({
            "type": "table",
            "data": []
        }));
    }

    Ok(serde_json::json!({
        "type": "SUCCESS",
        "result": null,
        "message": "Query executed"
    }))
}
