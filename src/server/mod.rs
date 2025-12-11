//! PhotonDB Server Implementation
//!
//! Rust-based web server using axum framework (replaces JavaScript/Node.js)

pub mod database_handlers;
pub mod handlers;
pub mod internal;
pub mod middleware;
pub mod routes;
pub mod security;
pub mod websocket;

use axum::{extract::Extension, Router};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info, warn};

use crate::cluster::{ClusterState, ReplicationConfig, ReplicationManager};
use crate::cluster::discovery::{DiscoveryConfig, DiscoveryManager};
use crate::cluster::health::{HealthChecker, DatabaseHealth, ClusterHealth};
use crate::cluster::metrics::MetricsCollector;
use crate::cluster::scaling::{AutoScaler, ScalingStrategy};
use crate::query::QueryExecutor;
use crate::storage::Storage;

pub use security::{SecurityConfig, SecurityState};

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// HTTP server bind address
    pub http_addr: String,
    /// HTTP port
    pub http_port: u16,
    /// Enable CORS
    pub enable_cors: bool,
    /// Maximum request body size (bytes)
    pub max_body_size: usize,
    /// Request timeout (seconds)
    pub timeout_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            http_addr: "0.0.0.0".to_string(),
            http_port: 8080,
            enable_cors: true,
            max_body_size: 10 * 1024 * 1024, // 10MB
            timeout_secs: 30,
        }
    }
}

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Storage>,
    pub executor: Arc<QueryExecutor>,
    pub config: ServerConfig,
    pub security: Option<Arc<SecurityState>>,
    pub cluster: Arc<ClusterState>,
    pub health: Arc<HealthChecker>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("config", &self.config)
            .finish()
    }
}

/// Cluster configuration from environment
#[derive(Debug, Clone)]
pub struct ClusterConfig {
    pub enabled: bool,
    pub node_id: String,
    pub mode: String,
    pub peers: Vec<String>,
    pub replication: ReplicationConfig,
}

impl ClusterConfig {
    pub fn from_env() -> Self {
        let enabled = std::env::var("PHOTONDB_CLUSTER_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let node_id = std::env::var("PHOTONDB_NODE_ID")
            .unwrap_or_else(|_| format!("node-{}", uuid::Uuid::new_v4()));

        let mode = std::env::var("PHOTONDB_CLUSTER_MODE")
            .unwrap_or_else(|_| "standalone".to_string());

        let peers = std::env::var("PHOTONDB_PEERS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        let replica_count = std::env::var("PHOTONDB_REPLICA_COUNT")
            .unwrap_or_else(|_| "3".to_string())
            .parse()
            .unwrap_or(3);

        let shard_count = std::env::var("PHOTONDB_SHARD_COUNT")
            .unwrap_or_else(|_| "16".to_string())
            .parse()
            .unwrap_or(16);

        Self {
            enabled,
            node_id,
            mode,
            peers,
            replication: ReplicationConfig {
                replica_count,
                replication_factor: replica_count,
                shard_count,
                enable_read_replicas: true,
                write_quorum: (replica_count / 2) + 1,
            },
        }
    }
}

/// Start the RethinkDB server
pub async fn start_server(
    config: ServerConfig,
    storage: Arc<Storage>,
    security_config: Option<SecurityConfig>,
) -> anyhow::Result<()> {
    info!(
        addr = %config.http_addr,
        port = config.http_port,
        "Starting PhotonDB HTTP server"
    );

    // Load cluster configuration
    let cluster_config = ClusterConfig::from_env();
    info!(
        enabled = cluster_config.enabled,
        node_id = %cluster_config.node_id,
        mode = %cluster_config.mode,
        peers = cluster_config.peers.len(),
        "Cluster configuration loaded"
    );

    // Create security state if enabled
    let security_state = security_config.map(|cfg| Arc::new(SecurityState::new(cfg)));

    if security_state.is_some() {
        info!("🔒 Security middleware enabled");
    } else {
        info!("⚠️  Security middleware disabled (DEV mode)");
    }

    // Create query executor
    let executor = Arc::new(QueryExecutor::new(storage.clone()));

    // Initialize cluster state
    let cluster = Arc::new(ClusterState::new(
        cluster_config.node_id.clone(),
        cluster_config.replication.clone(),
    ));

    // Initialize as master if in standalone mode
    if cluster_config.mode == "standalone" || cluster_config.mode == "master" {
        cluster.init_as_master().await;
        info!("🔵 Node initialized as MASTER");
    } else {
        info!("🟢 Node initialized as REPLICA");
    }

    // Start replication manager
    if cluster_config.enabled {
        let replication_manager = ReplicationManager::new(cluster.clone());
        replication_manager.start().await;
        info!("🔄 Replication manager started");
    }

    // Start service discovery
    let discovery_config = DiscoveryConfig::from_env();
    if discovery_config.enabled {
        let discovery_manager = DiscoveryManager::new(discovery_config.clone(), cluster.clone());
        discovery_manager.start().await;
        info!(
            "🔍 Service discovery started (DNS: {})",
            discovery_config.get_dns_name()
        );
    } else {
        // Manual peer configuration from env
        if !cluster_config.peers.is_empty() {
            info!(
                peers = cluster_config.peers.len(),
                "Adding {} manual peer(s)",
                cluster_config.peers.len()
            );
            for peer_addr in &cluster_config.peers {
                if let Ok(addr) = peer_addr.parse() {
                    let node_id = format!("peer-{}", peer_addr);
                    let node = crate::cluster::Node {
                        id: node_id.clone(),
                        addr,
                        role: crate::cluster::NodeRole::Replica,
                        shard_range: None,
                        last_heartbeat: chrono::Utc::now(),
                    };
                    cluster.add_node(node).await;
                    info!(node_id = %node_id, addr = %peer_addr, "Added manual peer");
                }
            }
        }
    }

    // Initialize health checker
    let health = Arc::new(HealthChecker::new());
    health.set_ready().await;
    info!("❤️  Health checker initialized");

    // Initialize and start metrics collector
    crate::cluster::metrics::init_metrics();
    let metrics_collector = MetricsCollector::new();
    let _metrics_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(15));
        loop {
            interval.tick().await;
            // Collect system metrics using sysinfo
            let mut sys = sysinfo::System::new_all();
            sys.refresh_all();
            
            let cpu = sys.cpus().first().map(|c| c.cpu_usage()).unwrap_or(0.0);
            let memory_bytes = sys.used_memory();
            let memory_percent = (sys.used_memory() as f64 / sys.total_memory() as f64 * 100.0) as f32;
            let disk_bytes = 0; // TODO: Implement disk monitoring
            let disk_percent = 0.0;
            
            metrics_collector.update_resource_metrics(
                cpu,
                memory_bytes,
                memory_percent,
                disk_bytes,
                disk_percent,
            ).await;
        }
    });
    info!("📊 Metrics collector started");

    // Start auto-scaler if enabled
    let autoscaling_enabled = std::env::var("PHOTONDB_AUTOSCALING_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);

    if autoscaling_enabled && cluster_config.enabled {
        let strategy_name = std::env::var("PHOTONDB_SCALING_STRATEGY")
            .unwrap_or_else(|_| "hybrid".to_string());

        let strategy = match strategy_name.as_str() {
            "horizontal" => ScalingStrategy::Horizontal(
                crate::cluster::scaling::HorizontalScalingConfig::default()
            ),
            "vertical" => ScalingStrategy::Vertical(
                crate::cluster::scaling::VerticalScalingConfig::default()
            ),
            "hybrid" => ScalingStrategy::Hybrid {
                horizontal: crate::cluster::scaling::HorizontalScalingConfig::default(),
                vertical: crate::cluster::scaling::VerticalScalingConfig::default(),
            },
            _ => {
                warn!("Unknown scaling strategy '{}', using Hybrid", strategy_name);
                ScalingStrategy::Hybrid {
                    horizontal: crate::cluster::scaling::HorizontalScalingConfig::default(),
                    vertical: crate::cluster::scaling::VerticalScalingConfig::default(),
                }
            }
        };

        let auto_scaler = AutoScaler::new(strategy);
        
        // Start auto-scaler in background
        tokio::spawn(async move {
            auto_scaler.start(60).await;
        });

        info!("🔧 Auto-scaler started with {} strategy", strategy_name);
    } else if autoscaling_enabled {
        warn!("⚠️  Auto-scaling enabled but cluster is disabled - auto-scaler will not start");
    }

    // Update health status with initial cluster info
    health.update_database_health(DatabaseHealth {
        status: "healthy".to_string(),
        tables_count: 0,
        active_queries: 0,
        connections: 0,
    }).await;

    health.update_cluster_health(ClusterHealth {
        status: "healthy".to_string(),
        nodes: cluster.get_nodes().await.len() as u64,
        masters: cluster.get_masters().await.len() as u64,
        replicas: cluster.get_replicas().await.len() as u64,
        replication_lag_ms: 0.0,
    }).await;

    health.set_startup_complete().await;
    info!("✅ Startup complete - application is healthy");

    // Build application state
    let state = AppState {
        storage,
        executor,
        config: config.clone(),
        security: security_state.clone(),
        cluster,
        health,
    };

    // Build router with all routes
    let app = Router::new()
        .merge(routes::api_routes())
        .merge(routes::database_routes()) // NEW: Database hierarchy routes
        .merge(routes::admin_routes())
        .merge(routes::health_routes())
        .merge(internal::internal_routes()) // Internal cluster communication
        .layer(Extension(Arc::new(state)))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new());

    // Add security middleware if enabled
    if let Some(_sec_state) = security_state {
        info!("Security state initialized (middleware integration pending)");
        // Note: Security middleware would be added here
        // For now, we just store it in AppState
        // TODO: Integrate security::security_middleware
    }

    // Add CORS if enabled
    let app = if config.enable_cors {
        app.layer(CorsLayer::permissive())
    } else {
        app
    };

    // Bind and serve
    let addr = format!("{}:{}", config.http_addr, config.http_port);
    let listener = TcpListener::bind(&addr).await?;

    info!("Server listening on http://{}", addr);
    info!("📊 Dashboard: http://{}/_admin", addr);
    info!("🔍 Metrics: http://{}/_metrics", addr);
    info!("❤️  Health: http://{}/_health", addr);

    axum::serve(listener, app).await.map_err(|e| {
        error!(error = %e, "Server error");
        anyhow::anyhow!("Server failed: {}", e)
    })
}
