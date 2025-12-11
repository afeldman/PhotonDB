//! Database-specific HTTP handlers
//!
//! Provides REST API endpoints for database operations:
//! - GET /api/dbs - List all databases
//! - POST /api/dbs - Create a database
//! - DELETE /api/dbs/:name - Drop a database
//! - GET /api/dbs/:name - Get database info
//! - GET /api/dbs/:name/tables - List tables in database
//! - POST /api/dbs/:name/tables - Create table in database
//! - DELETE /api/dbs/:name/tables/:table - Drop table

use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, instrument};

use crate::server::AppState;
use crate::storage::engine::StorageEngine;
use crate::storage::DefaultStorageEngine;

// ===== Request/Response Types =====

#[derive(Debug, Deserialize)]
pub struct CreateDatabaseRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct DatabaseResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DatabaseListResponse {
    pub success: bool,
    pub databases: Vec<DatabaseInfo>,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct DatabaseInfo {
    pub name: String,
    pub id: String,
    pub created_at: u64,
    pub table_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct CreateTableRequest {
    pub name: String,
    #[serde(default = "default_primary_key")]
    pub primary_key: String,
}

fn default_primary_key() -> String {
    "id".to_string()
}

#[derive(Debug, Serialize)]
pub struct TableResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TableListResponse {
    pub success: bool,
    pub tables: Vec<TableInfo>,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct TableInfo {
    pub name: String,
    pub id: String,
    pub database_id: String,
    pub primary_key: String,
    pub doc_count: u64,
    pub indexes: Vec<String>,
}

// ===== Database Handlers =====

/// List all databases
///
/// GET /api/dbs
#[instrument]
pub async fn list_databases(Extension(_state): Extension<Arc<AppState>>) -> Response {
    info!("Listing all databases");

    // TODO: Replace with actual DatabaseEngine from AppState
    let engine = match DefaultStorageEngine::with_defaults("./data") {
        Ok(e) => e,
        Err(e) => {
            error!(error = %e, "Failed to open database engine");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DatabaseListResponse {
                    success: false,
                    databases: vec![],
                    count: 0,
                }),
            )
                .into_response();
        }
    };

    match engine.list_databases().await {
        Ok(db_names) => {
            let mut databases = Vec::new();

            for name in db_names {
                // Simplified: Use default values since get_database_config is not available in StorageEngine
                let table_count = engine.list_tables_in_db(&name).await.unwrap_or_default().len();

                databases.push(DatabaseInfo {
                    name: name.clone(),
                    id: format!("db_{}", name), // Simplified ID
                    created_at: 0, // TODO: Store creation timestamp
                    table_count,
                });
            }

            let count = databases.len();
            Json(DatabaseListResponse {
                success: true,
                databases,
                count,
            })
            .into_response()
        }
        Err(e) => {
            error!(error = %e, "Failed to list databases");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DatabaseListResponse {
                    success: false,
                    databases: vec![],
                    count: 0,
                }),
            )
                .into_response()
        }
    }
}

/// Create a new database
///
/// POST /api/dbs
/// Body: {"name": "my_database"}
#[instrument(skip(_state, payload))]
pub async fn create_database(
    Extension(_state): Extension<Arc<AppState>>,
    Json(payload): Json<CreateDatabaseRequest>,
) -> Response {
    info!(database = %payload.name, "Creating database");

    // TODO: Replace with actual DatabaseEngine from AppState
    let engine = match DefaultStorageEngine::with_defaults("./data") {
        Ok(e) => e,
        Err(e) => {
            error!(error = %e, "Failed to open database engine");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DatabaseResponse {
                    success: false,
                    id: None,
                    error: Some(format!("Failed to open database engine: {}", e)),
                }),
            )
                .into_response();
        }
    };

    match engine.create_database(&payload.name).await {
        Ok(()) => {
            info!(database = %payload.name, "Database created");
            Json(DatabaseResponse {
                success: true,
                id: Some(format!("db_{}", payload.name)), // Simplified ID
                error: None,
            })
            .into_response()
        }
        Err(e) => {
            error!(error = %e, database = %payload.name, "Failed to create database");
            let status = match e {
                crate::error::Error::AlreadyExists(_) => StatusCode::CONFLICT,
                crate::error::Error::InvalidArgument(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (
                status,
                Json(DatabaseResponse {
                    success: false,
                    id: None,
                    error: Some(e.to_string()),
                }),
            )
                .into_response()
        }
    }
}

/// Get database information
///
/// GET /api/dbs/:name
#[instrument]
pub async fn get_database(
    Extension(_state): Extension<Arc<AppState>>,
    Path(name): Path<String>,
) -> Response {
    info!(database = %name, "Getting database info");

    let engine = match DefaultStorageEngine::with_defaults("./data") {
        Ok(e) => e,
        Err(e) => {
            error!(error = %e, "Failed to open database engine");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to open database engine",
            )
                .into_response();
        }
    };

    // Check if database exists by trying to list it
    match engine.list_databases().await {
        Ok(dbs) if dbs.contains(&name) => {
            let table_count = engine.list_tables_in_db(&name).await.unwrap_or_default().len();

            Json(serde_json::json!({
                "success": true,
                "database": {
                    "name": name,
                    "id": format!("db_{}", name),
                    "created_at": 0, // TODO: Store creation timestamp
                    "table_count": table_count,
                }
            }))
            .into_response()
        }
        Ok(_) => {
            error!(database = %name, "Database not found");
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Database '{}' not found", name),
                })),
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, database = %name, "Failed to get database");
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

/// Drop (delete) a database
///
/// DELETE /api/dbs/:name
#[instrument(skip(_state))]
pub async fn drop_database(
    Extension(_state): Extension<Arc<AppState>>,
    Path(name): Path<String>,
) -> Response {
    info!(database = %name, "Dropping database");

    let engine = match DefaultStorageEngine::with_defaults("./data") {
        Ok(e) => e,
        Err(e) => {
            error!(error = %e, "Failed to open database engine");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to open database engine",
            )
                .into_response();
        }
    };

    match engine.drop_database(&name).await {
        Ok(()) => {
            info!(database = %name, "Database dropped");
            Json(DatabaseResponse {
                success: true,
                id: None,
                error: None,
            })
            .into_response()
        }
        Err(e) => {
            error!(error = %e, database = %name, "Failed to drop database");
            let status = match e {
                crate::error::Error::NotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (
                status,
                Json(DatabaseResponse {
                    success: false,
                    id: None,
                    error: Some(e.to_string()),
                }),
            )
                .into_response()
        }
    }
}

// ===== Table Handlers =====

/// List tables in a database
///
/// GET /api/dbs/:db_name/tables
#[instrument(skip(_state))]
pub async fn list_tables(
    Extension(_state): Extension<Arc<AppState>>,
    Path(db_name): Path<String>,
) -> Response {
    info!(database = %db_name, "Listing tables");

    let engine = match DefaultStorageEngine::with_defaults("./data") {
        Ok(e) => e,
        Err(e) => {
            error!(error = %e, "Failed to open database engine");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TableListResponse {
                    success: false,
                    tables: vec![],
                    count: 0,
                }),
            )
                .into_response();
        }
    };

    match engine.list_tables_in_db(&db_name).await {
        Ok(table_names) => {
            let mut tables = Vec::new();

            for name in table_names {
                // Use get_table_info with full table name "db.table"
                let full_name = format!("{}.{}", db_name, name);
                match engine.get_table_info(&full_name).await {
                    Ok(Some(info)) => {
                        tables.push(TableInfo {
                            name: info.name,
                            id: format!("table_{}", name), // Simplified ID
                            database_id: format!("db_{}", db_name), // Simplified DB ID
                            primary_key: info.primary_key,
                            doc_count: info.doc_count,
                            indexes: info.indexes,
                        });
                    }
                    Ok(None) => {
                        error!(table = %name, "Table info not found");
                    }
                    Err(e) => {
                        error!(error = %e, table = %name, "Failed to get table info");
                    }
                }
            }

            let count = tables.len();
            Json(TableListResponse {
                success: true,
                tables,
                count,
            })
            .into_response()
        }
        Err(e) => {
            error!(error = %e, database = %db_name, "Failed to list tables");
            let status = match e {
                crate::error::Error::NotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (
                status,
                Json(TableListResponse {
                    success: false,
                    tables: vec![],
                    count: 0,
                }),
            )
                .into_response()
        }
    }
}

/// Create a table in a database
///
/// POST /api/dbs/:db_name/tables
/// Body: {"name": "users", "primary_key": "id"}
#[instrument(skip(_state, payload))]
pub async fn create_table(
    Extension(_state): Extension<Arc<AppState>>,
    Path(db_name): Path<String>,
    Json(payload): Json<CreateTableRequest>,
) -> Response {
    info!(database = %db_name, table = %payload.name, "Creating table");

    let engine = match DefaultStorageEngine::with_defaults("./data") {
        Ok(e) => e,
        Err(e) => {
            error!(error = %e, "Failed to open database engine");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TableResponse {
                    success: false,
                    id: None,
                    error: Some(format!("Failed to open database engine: {}", e)),
                }),
            )
                .into_response();
        }
    };

    match engine
        .create_table(&db_name, &payload.name, &payload.primary_key)
        .await
    {
        Ok(()) => {
            info!(database = %db_name, table = %payload.name, "Table created");
            Json(TableResponse {
                success: true,
                id: Some(format!("table_{}", payload.name)), // Simplified ID
                error: None,
            })
            .into_response()
        }
        Err(e) => {
            error!(error = %e, database = %db_name, table = %payload.name, "Failed to create table");
            let status = match e {
                crate::error::Error::NotFound(_) => StatusCode::NOT_FOUND,
                crate::error::Error::AlreadyExists(_) => StatusCode::CONFLICT,
                crate::error::Error::InvalidArgument(_) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (
                status,
                Json(TableResponse {
                    success: false,
                    id: None,
                    error: Some(e.to_string()),
                }),
            )
                .into_response()
        }
    }
}

/// Drop (delete) a table
///
/// DELETE /api/dbs/:db_name/tables/:table_name
#[instrument(skip(_state))]
pub async fn drop_table(
    Extension(_state): Extension<Arc<AppState>>,
    Path((db_name, table_name)): Path<(String, String)>,
) -> Response {
    info!(database = %db_name, table = %table_name, "Dropping table");

    let engine = match DefaultStorageEngine::with_defaults("./data") {
        Ok(e) => e,
        Err(e) => {
            error!(error = %e, "Failed to open database engine");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to open database engine",
            )
                .into_response();
        }
    };

    match engine.drop_table(&db_name, &table_name).await {
        Ok(()) => {
            info!(database = %db_name, table = %table_name, "Table dropped");
            Json(TableResponse {
                success: true,
                id: None,
                error: None,
            })
            .into_response()
        }
        Err(e) => {
            error!(error = %e, database = %db_name, table = %table_name, "Failed to drop table");
            let status = match e {
                crate::error::Error::NotFound(_) => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (
                status,
                Json(TableResponse {
                    success: false,
                    id: None,
                    error: Some(e.to_string()),
                }),
            )
                .into_response()
        }
    }
}
