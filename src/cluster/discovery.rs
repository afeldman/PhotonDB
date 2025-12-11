//! Kubernetes Service Discovery for automatic cluster formation
//!
//! Features:
//! - Automatic node discovery using K8s DNS
//! - Peer list management
//! - Health monitoring and failover
//! - StatefulSet-aware (stable network identities)

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info, instrument, warn};

use super::{ClusterState, Node, NodeRole};

/// Service discovery configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Kubernetes service name for cluster members
    pub service_name: String,
    /// Kubernetes namespace
    pub namespace: String,
    /// Port for cluster communication
    pub cluster_port: u16,
    /// Discovery interval in seconds
    pub discovery_interval_secs: u64,
    /// Enable Kubernetes-based discovery
    pub enabled: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            service_name: "rethinkdb".to_string(),
            namespace: "default".to_string(),
            cluster_port: 29015,
            discovery_interval_secs: 30,
            enabled: false,
        }
    }
}

impl DiscoveryConfig {
    /// Load from environment variables
    pub fn from_env() -> Self {
        let enabled = std::env::var("PHOTONDB_K8S_DISCOVERY")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let service_name = std::env::var("PHOTONDB_SERVICE_NAME")
            .unwrap_or_else(|_| "rethinkdb".to_string());

        let namespace = std::env::var("PHOTONDB_NAMESPACE")
            .or_else(|_| {
                // Try to read namespace from service account
                std::fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/namespace")
            })
            .unwrap_or_else(|_| "default".to_string());

        let cluster_port = std::env::var("PHOTONDB_CLUSTER_PORT")
            .unwrap_or_else(|_| "29015".to_string())
            .parse()
            .unwrap_or(29015);

        let discovery_interval_secs = std::env::var("PHOTONDB_DISCOVERY_INTERVAL")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        Self {
            service_name,
            namespace,
            cluster_port,
            discovery_interval_secs,
            enabled,
        }
    }

    /// Get DNS name for StatefulSet headless service
    pub fn get_dns_name(&self) -> String {
        format!(
            "{}.{}.svc.cluster.local",
            self.service_name, self.namespace
        )
    }

    /// Get DNS name for a specific pod
    pub fn get_pod_dns_name(&self, pod_index: usize) -> String {
        format!(
            "{}-{}.{}.{}.svc.cluster.local",
            self.service_name, pod_index, self.service_name, self.namespace
        )
    }
}

/// Service discovery manager
pub struct DiscoveryManager {
    config: DiscoveryConfig,
    cluster: Arc<ClusterState>,
}

impl DiscoveryManager {
    pub fn new(config: DiscoveryConfig, cluster: Arc<ClusterState>) -> Self {
        Self { config, cluster }
    }

    /// Start background discovery task
    #[instrument(skip(self))]
    pub async fn start(&self) {
        if !self.config.enabled {
            info!("Service discovery is DISABLED");
            return;
        }

        info!(
            service = %self.config.service_name,
            namespace = %self.config.namespace,
            "Starting Kubernetes service discovery"
        );

        let config = self.config.clone();
        let cluster = self.cluster.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.discovery_interval_secs));

            loop {
                interval.tick().await;

                if let Err(e) = Self::discover_peers(&config, &cluster).await {
                    let error_msg = e.to_string();
                    error!(error = %error_msg, "Failed to discover peers");
                }
            }
        });

        info!("Service discovery background task started");
    }

    /// Discover peers using DNS
    #[instrument(skip(config, cluster))]
    async fn discover_peers(
        config: &DiscoveryConfig,
        cluster: &Arc<ClusterState>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let dns_name = config.get_dns_name();
        debug!(dns = %dns_name, "Resolving DNS for peer discovery");

        // Resolve DNS to get all pod IPs
        let resolver = tokio::net::lookup_host(format!("{}:{}", dns_name, config.cluster_port))
            .await
            .map_err(|e| {
                warn!(error = %e, dns = %dns_name, "DNS resolution failed");
                e
            })?;

        let mut discovered_count = 0;

        for addr in resolver {
            discovered_count += 1;

            // Check if node already exists
            let node_id = format!("node-{}", addr.ip());
            let existing_nodes = cluster.get_nodes().await;

            if existing_nodes.iter().any(|n| n.id == node_id) {
                debug!(node_id = %node_id, addr = %addr, "Node already registered");
                continue;
            }

            // Add new node
            let node = Node {
                id: node_id.clone(),
                addr,
                role: NodeRole::Replica, // Initially as replica
                shard_range: None,
                last_heartbeat: chrono::Utc::now(),
            };

            cluster.add_node(node).await;
            info!(node_id = %node_id, addr = %addr, "Discovered new peer");
        }

        let node_count = cluster.get_nodes().await.len();
        info!(
            discovered = discovered_count,
            total = node_count,
            "Peer discovery completed"
        );

        Ok(())
    }

    /// Discover peers using Kubernetes API (alternative to DNS)
    #[allow(dead_code)]
    #[cfg(feature = "k8s-api")]
    #[instrument(skip(config, cluster))]
    async fn discover_peers_k8s_api(
        config: &DiscoveryConfig,
        cluster: &Arc<ClusterState>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use k8s_openapi::api::core::v1::Pod;
        use kube::{Api, Client};

        let client = Client::try_default().await?;
        let pods: Api<Pod> = Api::namespaced(client, &config.namespace);

        // List pods with label selector
        let label_selector = format!("app.kubernetes.io/name={}", config.service_name);
        let pod_list = pods.list(&kube::api::ListParams::default().labels(&label_selector)).await?;

        info!(
            pod_count = pod_list.items.len(),
            "Discovered pods via Kubernetes API"
        );

        for pod in pod_list.items {
            if let Some(pod_status) = pod.status {
                if let Some(pod_ip) = pod_status.pod_ip {
                    let addr: SocketAddr = format!("{}:{}", pod_ip, config.cluster_port)
                        .parse()
                        .unwrap();

                    let node_id = pod
                        .metadata
                        .name
                        .unwrap_or_else(|| format!("node-{}", pod_ip));

                    let node = Node {
                        id: node_id.clone(),
                        addr,
                        role: NodeRole::Replica,
                        shard_range: None,
                        last_heartbeat: chrono::Utc::now(),
                    };

                    cluster.add_node(node).await;
                    info!(node_id = %node_id, addr = %addr, "Registered peer from K8s API");
                }
            }
        }

        Ok(())
    }

    /// Manually add peer
    pub async fn add_peer(&self, addr: SocketAddr) {
        let node_id = format!("node-{}", addr.ip());

        let node = Node {
            id: node_id.clone(),
            addr,
            role: NodeRole::Replica,
            shard_range: None,
            last_heartbeat: chrono::Utc::now(),
        };

        self.cluster.add_node(node).await;
        info!(node_id = %node_id, addr = %addr, "Manually added peer");
    }

    /// Remove peer
    pub async fn remove_peer(&self, node_id: &str) {
        self.cluster.remove_node(node_id).await;
        info!(node_id = %node_id, "Removed peer");
    }

    /// Get all discovered peers
    pub async fn get_peers(&self) -> Vec<Node> {
        self.cluster.get_nodes().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cluster::ReplicationConfig;

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert_eq!(config.service_name, "rethinkdb");
        assert_eq!(config.namespace, "default");
        assert_eq!(config.cluster_port, 29015);
        assert!(!config.enabled);
    }

    #[test]
    fn test_dns_name_generation() {
        let config = DiscoveryConfig {
            service_name: "rethinkdb".to_string(),
            namespace: "production".to_string(),
            cluster_port: 29015,
            discovery_interval_secs: 30,
            enabled: true,
        };

        assert_eq!(
            config.get_dns_name(),
            "rethinkdb.production.svc.cluster.local"
        );
    }

    #[test]
    fn test_pod_dns_name_generation() {
        let config = DiscoveryConfig {
            service_name: "rethinkdb".to_string(),
            namespace: "production".to_string(),
            cluster_port: 29015,
            discovery_interval_secs: 30,
            enabled: true,
        };

        assert_eq!(
            config.get_pod_dns_name(0),
            "rethinkdb-0.rethinkdb.production.svc.cluster.local"
        );

        assert_eq!(
            config.get_pod_dns_name(2),
            "rethinkdb-2.rethinkdb.production.svc.cluster.local"
        );
    }

    #[tokio::test]
    async fn test_discovery_manager_creation() {
        let config = DiscoveryConfig::default();
        let cluster = Arc::new(ClusterState::new(
            "node1".to_string(),
            ReplicationConfig::default(),
        ));

        let manager = DiscoveryManager::new(config, cluster);
        assert!(!manager.config.enabled);
    }

    #[tokio::test]
    async fn test_manual_peer_management() {
        let config = DiscoveryConfig::default();
        let cluster = Arc::new(ClusterState::new(
            "node1".to_string(),
            ReplicationConfig::default(),
        ));

        let manager = DiscoveryManager::new(config, cluster.clone());

        // Add peer
        let addr: SocketAddr = "127.0.0.1:29015".parse().unwrap();
        manager.add_peer(addr).await;

        let peers = manager.get_peers().await;
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].addr, addr);

        // Remove peer
        let node_id = peers[0].id.clone();
        manager.remove_peer(&node_id).await;

        let peers = manager.get_peers().await;
        assert_eq!(peers.len(), 0);
    }
}
