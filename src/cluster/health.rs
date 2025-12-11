//! Health check endpoints for Kubernetes probes
//!
//! Features:
//! - Liveness probe (is the process alive?)
//! - Readiness probe (ready to accept traffic?)
//! - Startup probe (has initialization completed?)

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument, warn};

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall status ("healthy", "degraded", "starting")
    pub status: String,
    /// State (alias for status)
    pub state: String,
    /// Ready flag (for K8s readiness probe)
    pub ready: bool,
    /// Alive flag (for K8s liveness probe)
    pub alive: bool,
    /// Application version
    pub version: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Database status
    pub database: DatabaseHealth,
    /// Cluster status
    pub cluster: ClusterHealth,
}

/// Database health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealth {
    pub status: String,
    pub tables_count: u64,
    pub active_queries: u64,
    pub connections: u64,
}

/// Cluster health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterHealth {
    pub status: String,
    pub nodes: u64,
    pub masters: u64,
    pub replicas: u64,
    pub replication_lag_ms: f64,
}

/// Health check manager
#[derive(Clone)]
pub struct HealthChecker {
    start_time: Arc<RwLock<std::time::Instant>>,
    is_ready: Arc<RwLock<bool>>,
    is_startup_complete: Arc<RwLock<bool>>,
    database_health: Arc<RwLock<DatabaseHealth>>,
    cluster_health: Arc<RwLock<ClusterHealth>>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            start_time: Arc::new(RwLock::new(std::time::Instant::now())),
            is_ready: Arc::new(RwLock::new(false)),
            is_startup_complete: Arc::new(RwLock::new(false)),
            database_health: Arc::new(RwLock::new(DatabaseHealth {
                status: "starting".to_string(),
                tables_count: 0,
                active_queries: 0,
                connections: 0,
            })),
            cluster_health: Arc::new(RwLock::new(ClusterHealth {
                status: "starting".to_string(),
                nodes: 0,
                masters: 0,
                replicas: 0,
                replication_lag_ms: 0.0,
            })),
        }
    }

    /// Mark as ready
    pub async fn set_ready(&self) {
        let mut is_ready = self.is_ready.write().await;
        *is_ready = true;
        info!("Health checker: Application is READY");
    }

    /// Mark as not ready
    pub async fn set_not_ready(&self) {
        let mut is_ready = self.is_ready.write().await;
        *is_ready = false;
        warn!("Health checker: Application is NOT READY");
    }

    /// Mark startup as complete
    pub async fn set_startup_complete(&self) {
        let mut is_complete = self.is_startup_complete.write().await;
        *is_complete = true;
        info!("Health checker: Startup COMPLETE");
    }

    /// Update database health
    pub async fn update_database_health(&self, health: DatabaseHealth) {
        let mut db_health = self.database_health.write().await;
        *db_health = health;
    }

    /// Update cluster health
    pub async fn update_cluster_health(&self, health: ClusterHealth) {
        let mut cl_health = self.cluster_health.write().await;
        *cl_health = health;
    }

    /// Get uptime in seconds
    async fn get_uptime(&self) -> u64 {
        let start = self.start_time.read().await;
        start.elapsed().as_secs()
    }

    /// Check liveness (basic process health)
    #[instrument(skip(self))]
    pub async fn check_liveness(&self) -> bool {
        // If we can execute this function, the process is alive
        true
    }

    /// Check readiness (can accept traffic)
    #[instrument(skip(self))]
    pub async fn check_readiness(&self) -> bool {
        let is_ready = *self.is_ready.read().await;
        let db_health = self.database_health.read().await;
        let cluster_health = self.cluster_health.read().await;

        // Ready if:
        // 1. Startup is complete
        // 2. Database is healthy
        // 3. Cluster has at least one node
        is_ready
            && db_health.status == "healthy"
            && cluster_health.nodes > 0
    }

    /// Check startup (has initialization completed)
    #[instrument(skip(self))]
    pub async fn check_startup(&self) -> bool {
        *self.is_startup_complete.read().await
    }

    /// Get full health status
    pub async fn get_status(&self) -> HealthStatus {
        let uptime = self.get_uptime().await;
        let db_health = self.database_health.read().await.clone();
        let cluster_health = self.cluster_health.read().await.clone();
        
        let ready = self.check_readiness().await;
        let alive = self.check_liveness().await;
        let startup = self.check_startup().await;

        let overall_status = if ready {
            "healthy"
        } else if startup {
            "degraded"
        } else {
            "starting"
        };

        HealthStatus {
            status: overall_status.to_string(),
            state: overall_status.to_string(),
            ready,
            alive,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: uptime,
            database: db_health,
            cluster: cluster_health,
        }
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Liveness probe handler
/// Returns 200 if process is alive
#[instrument(skip(health))]
pub async fn liveness_handler(
    State(health): State<Arc<HealthChecker>>,
) -> impl IntoResponse {
    if health.check_liveness().await {
        (StatusCode::OK, Json(serde_json::json!({ "status": "alive" })))
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "status": "dead" })),
        )
    }
}

/// Readiness probe handler
/// Returns 200 if ready to accept traffic
#[instrument(skip(health))]
pub async fn readiness_handler(
    State(health): State<Arc<HealthChecker>>,
) -> impl IntoResponse {
    if health.check_readiness().await {
        (StatusCode::OK, Json(serde_json::json!({ "status": "ready" })))
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "status": "not_ready" })),
        )
    }
}

/// Startup probe handler
/// Returns 200 if startup is complete
#[instrument(skip(health))]
pub async fn startup_handler(
    State(health): State<Arc<HealthChecker>>,
) -> impl IntoResponse {
    if health.check_startup().await {
        (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "started" })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "status": "starting" })),
        )
    }
}

/// Detailed health status handler
#[instrument(skip(health))]
pub async fn health_status_handler(
    State(health): State<Arc<HealthChecker>>,
) -> impl IntoResponse {
    let status = health.get_status().await;
    let status_code = match status.status.as_str() {
        "healthy" => StatusCode::OK,
        "degraded" => StatusCode::OK,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(status))
}

/// Create health check router
pub fn health_router(health: Arc<HealthChecker>) -> Router {
    Router::new()
        .route("/health/live", get(liveness_handler))
        .route("/health/ready", get(readiness_handler))
        .route("/health/startup", get(startup_handler))
        .route("/health", get(health_status_handler))
        .with_state(health)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_liveness_always_true() {
        let checker = HealthChecker::new();
        assert!(checker.check_liveness().await);
    }

    #[tokio::test]
    async fn test_readiness_requires_ready_state() {
        let checker = HealthChecker::new();
        
        // Initially not ready
        assert!(!checker.check_readiness().await);
        
        // Mark as ready
        checker.set_ready().await;
        checker.update_database_health(DatabaseHealth {
            status: "healthy".to_string(),
            tables_count: 1,
            active_queries: 0,
            connections: 0,
        }).await;
        checker.update_cluster_health(ClusterHealth {
            status: "healthy".to_string(),
            nodes: 1,
            masters: 1,
            replicas: 0,
            replication_lag_ms: 0.0,
        }).await;
        
        // Now should be ready
        assert!(checker.check_readiness().await);
    }

    #[tokio::test]
    async fn test_startup_initially_false() {
        let checker = HealthChecker::new();
        
        assert!(!checker.check_startup().await);
        
        checker.set_startup_complete().await;
        assert!(checker.check_startup().await);
    }

    #[tokio::test]
    async fn test_uptime_increases() {
        let checker = HealthChecker::new();
        
        let uptime1 = checker.get_uptime().await;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let uptime2 = checker.get_uptime().await;
        
        assert!(uptime2 >= uptime1);
    }

    #[tokio::test]
    async fn test_health_status() {
        let checker = HealthChecker::new();
        
        let status = checker.get_status().await;
        assert_eq!(status.status, "starting");
        assert_eq!(status.version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_health_transitions() {
        let checker = HealthChecker::new();
        
        // Starting
        let status = checker.get_status().await;
        assert_eq!(status.status, "starting");
        
        // Startup complete but not ready
        checker.set_startup_complete().await;
        let status = checker.get_status().await;
        assert_eq!(status.status, "degraded");
        
        // Fully ready
        checker.set_ready().await;
        checker.update_database_health(DatabaseHealth {
            status: "healthy".to_string(),
            tables_count: 1,
            active_queries: 0,
            connections: 5,
        }).await;
        checker.update_cluster_health(ClusterHealth {
            status: "healthy".to_string(),
            nodes: 3,
            masters: 1,
            replicas: 2,
            replication_lag_ms: 5.0,
        }).await;
        
        let status = checker.get_status().await;
        assert_eq!(status.status, "healthy");
        assert_eq!(status.cluster.nodes, 3);
    }
}
