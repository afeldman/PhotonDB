//! HTTP routes definition

use axum::{
    extract::Extension,
    routing::{delete, get, post},
    Router, Json,
};
use std::sync::Arc;

use super::{database_handlers, handlers, AppState};
use crate::cluster::health::HealthStatus;

/// API routes for query execution and legacy table operations
pub fn api_routes() -> Router {
    Router::new()
        .route("/api/query", post(handlers::execute_query))
        // Legacy table routes (will be deprecated)
        .route("/api/tables", get(handlers::list_tables))
        .route("/api/tables/:name", get(handlers::get_table_info))
}

/// Database hierarchy routes (NEW)
///
/// REST API for database and table management:
/// - GET    /api/dbs                    - List all databases
/// - POST   /api/dbs                    - Create a database
/// - GET    /api/dbs/:name              - Get database info
/// - DELETE /api/dbs/:name              - Drop a database
/// - GET    /api/dbs/:db/tables         - List tables in database
/// - POST   /api/dbs/:db/tables         - Create table in database
/// - DELETE /api/dbs/:db/tables/:table  - Drop table
pub fn database_routes() -> Router {
    Router::new()
        // Database operations
        .route("/api/dbs", get(database_handlers::list_databases))
        .route("/api/dbs", post(database_handlers::create_database))
        .route("/api/dbs/:name", get(database_handlers::get_database))
        .route("/api/dbs/:name", delete(database_handlers::drop_database))
        // Table operations (scoped to database)
        .route(
            "/api/dbs/:db_name/tables",
            get(database_handlers::list_tables),
        )
        .route(
            "/api/dbs/:db_name/tables",
            post(database_handlers::create_table),
        )
        .route(
            "/api/dbs/:db_name/tables/:table_name",
            delete(database_handlers::drop_table),
        )
}

/// Admin routes
pub fn admin_routes() -> Router {
    Router::new()
        .route("/_admin", get(admin_dashboard))
}

/// Health check routes
pub fn health_routes() -> Router {
    Router::new()
        .route("/_health", get(health_detailed))
        .route("/health", get(health_detailed))
        .route("/_ready", get(health_ready))
        .route("/health/ready", get(health_ready))
        .route("/health/live", get(health_live))
        .route("/health/startup", get(health_startup))
        .route("/_metrics", get(metrics_endpoint))
}

/// Admin dashboard (HTML)
async fn admin_dashboard() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../../static/admin.html"))
}

/// Detailed health check endpoint
async fn health_detailed(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<HealthStatus> {
    Json(state.health.get_status().await)
}

/// Readiness probe (K8s)
async fn health_ready(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<HealthStatus>, (axum::http::StatusCode, String)> {
    let status = state.health.get_status().await;
    if status.ready {
        Ok(Json(status))
    } else {
        Err((axum::http::StatusCode::SERVICE_UNAVAILABLE, "Not ready".to_string()))
    }
}

/// Liveness probe (K8s)
async fn health_live(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<HealthStatus>, (axum::http::StatusCode, String)> {
    let status = state.health.get_status().await;
    if status.alive {
        Ok(Json(status))
    } else {
        Err((axum::http::StatusCode::SERVICE_UNAVAILABLE, "Not alive".to_string()))
    }
}

/// Startup probe (K8s)
async fn health_startup(
    Extension(state): Extension<Arc<AppState>>,
) -> Result<Json<HealthStatus>, (axum::http::StatusCode, String)> {
    let status = state.health.get_status().await;
    if status.state != "starting" {
        Ok(Json(status))
    } else {
        Err((axum::http::StatusCode::SERVICE_UNAVAILABLE, "Still starting".to_string()))
    }
}

/// Prometheus metrics endpoint
async fn metrics_endpoint() -> String {
    crate::cluster::metrics::export_metrics()
}
