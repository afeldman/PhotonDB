//! Clustering and replication support for horizontal and vertical scaling
//!
//! Features:
//! - Raft consensus for distributed coordination
//! - Master-replica replication
//! - Automatic sharding with consistent hashing
//! - Read replicas for horizontal scaling
//! - Write scaling through sharding
//! - Kubernetes-native auto-scaling (HPA/VPA)
//! - Prometheus metrics for monitoring
//! - Health checks for liveness/readiness probes

pub mod discovery;
pub mod health;
pub mod k8s;
pub mod metrics;
pub mod scaling;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tracing::{error, info, instrument, warn};

/// Node role in the cluster
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeRole {
    /// Master node (accepts writes)
    Master,
    /// Replica node (read-only)
    Replica,
    /// Candidate (during election)
    Candidate,
}

/// Node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub addr: SocketAddr,
    pub role: NodeRole,
    pub shard_range: Option<ShardRange>,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
}

/// Shard range for consistent hashing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardRange {
    pub start: u64,
    pub end: u64,
}

/// Replication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationConfig {
    /// Number of replicas per shard
    pub replica_count: usize,
    /// Replication factor (1 = no replication)
    pub replication_factor: usize,
    /// Number of shards for horizontal scaling
    pub shard_count: usize,
    /// Enable read replicas
    pub enable_read_replicas: bool,
    /// Quorum size for writes (typically replica_count/2 + 1)
    pub write_quorum: usize,
}

impl Default for ReplicationConfig {
    fn default() -> Self {
        Self {
            replica_count: 3,
            replication_factor: 3,
            shard_count: 16,
            enable_read_replicas: true,
            write_quorum: 2,
        }
    }
}

/// Cluster state
pub struct ClusterState {
    config: ReplicationConfig,
    nodes: Arc<RwLock<HashMap<String, Node>>>,
    current_node_id: String,
    current_role: Arc<RwLock<NodeRole>>,
}

impl ClusterState {
    pub fn new(node_id: String, config: ReplicationConfig) -> Self {
        Self {
            config,
            nodes: Arc::new(RwLock::new(HashMap::new())),
            current_node_id: node_id,
            current_role: Arc::new(RwLock::new(NodeRole::Replica)),
        }
    }

    /// Initialize as master node
    #[instrument(skip(self))]
    pub async fn init_as_master(&self) {
        info!(node_id = %self.current_node_id, "Initializing as master node");
        let mut role = self.current_role.write().await;
        *role = NodeRole::Master;
    }

    /// Add a node to the cluster
    #[instrument(skip(self))]
    pub async fn add_node(&self, node: Node) {
        info!(
            node_id = %node.id,
            addr = %node.addr,
            role = ?node.role,
            "Adding node to cluster"
        );

        let mut nodes = self.nodes.write().await;
        nodes.insert(node.id.clone(), node);
    }

    /// Remove a node from the cluster
    #[instrument(skip(self))]
    pub async fn remove_node(&self, node_id: &str) {
        warn!(node_id = %node_id, "Removing node from cluster");

        let mut nodes = self.nodes.write().await;
        nodes.remove(node_id);
    }

    /// Get current node role
    pub async fn get_role(&self) -> NodeRole {
        *self.current_role.read().await
    }

    /// Check if current node is master
    pub async fn is_master(&self) -> bool {
        matches!(self.get_role().await, NodeRole::Master)
    }

    /// Get all nodes in cluster
    pub async fn get_nodes(&self) -> Vec<Node> {
        let nodes = self.nodes.read().await;
        nodes.values().cloned().collect()
    }

    /// Get master nodes
    pub async fn get_masters(&self) -> Vec<Node> {
        let nodes = self.nodes.read().await;
        nodes
            .values()
            .filter(|n| n.role == NodeRole::Master)
            .cloned()
            .collect()
    }

    /// Get replica nodes
    pub async fn get_replicas(&self) -> Vec<Node> {
        let nodes = self.nodes.read().await;
        nodes
            .values()
            .filter(|n| n.role == NodeRole::Replica)
            .cloned()
            .collect()
    }

    /// Calculate shard for a given key using consistent hashing
    pub fn calculate_shard(&self, key: &[u8]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        hash % self.config.shard_count as u64
    }

    /// Get nodes responsible for a shard
    pub async fn get_shard_nodes(&self, shard: u64) -> Vec<Node> {
        let nodes = self.nodes.read().await;
        nodes
            .values()
            .filter(|n| {
                if let Some(range) = &n.shard_range {
                    shard >= range.start && shard < range.end
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    /// Replicate data to replica nodes
    #[instrument(skip(self))]
    pub async fn replicate(&self, key: &[u8], _data: &[u8]) -> Result<(), String> {
        let shard = self.calculate_shard(key);
        let nodes = self.get_shard_nodes(shard).await;

        info!(
            shard = shard,
            node_count = nodes.len(),
            "Replicating data to nodes"
        );

        // Check if we have enough nodes for write quorum
        if nodes.len() < self.config.write_quorum {
            error!(
                available = nodes.len(),
                required = self.config.write_quorum,
                "Not enough nodes for write quorum"
            );
            return Err("Insufficient replicas for write quorum".to_string());
        }

        // Replicate to all nodes in parallel
        let mut replication_tasks = Vec::new();
        
        for node in nodes.iter() {
            let node_addr = node.addr;
            let node_id = node.id.clone();
            let key = key.to_vec();
            let data = _data.to_vec();
            
            // Spawn replication task for each node
            let task = tokio::spawn(async move {
                Self::replicate_to_node(node_addr, &node_id, &key, &data).await
            });
            
            replication_tasks.push(task);
        }

        // Wait for write quorum confirmations
        let mut successful_replications = 0;
        for task in replication_tasks {
            if let Ok(Ok(())) = task.await {
                successful_replications += 1;
            }
        }

        info!(
            successful = successful_replications,
            required = self.config.write_quorum,
            "Replication completed"
        );

        if successful_replications >= self.config.write_quorum {
            Ok(())
        } else {
            error!(
                successful = successful_replications,
                required = self.config.write_quorum,
                "Failed to achieve write quorum"
            );
            Err(format!(
                "Write quorum not achieved: {}/{}",
                successful_replications, self.config.write_quorum
            ))
        }
    }

    /// Handle node heartbeat
    #[instrument(skip(self))]
    pub async fn heartbeat(&self, node_id: &str) {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.get_mut(node_id) {
            node.last_heartbeat = chrono::Utc::now();
        }
    }

    /// Replicate data to a single node via HTTP
    async fn replicate_to_node(
        node_addr: SocketAddr,
        node_id: &str,
        key: &[u8],
        data: &[u8],
    ) -> Result<(), String> {
        // Build HTTP request to node's replication endpoint
        let url = format!("http://{}/internal/replicate", node_addr);
        
        // Create payload with key and data
        let payload = serde_json::json!({
            "key": BASE64.encode(key),
            "data": BASE64.encode(data),
        });

        // Send replication request with timeout
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            async {
                reqwest::Client::new()
                    .post(&url)
                    .json(&payload)
                    .send()
                    .await
            },
        )
        .await
        {
            Ok(Ok(response)) if response.status().is_success() => {
                info!(node_id = %node_id, "Replication successful");
                Ok(())
            }
            Ok(Ok(response)) => {
                warn!(
                    node_id = %node_id,
                    status = %response.status(),
                    "Replication failed with status"
                );
                Err(format!("Replication failed: {}", response.status()))
            }
            Ok(Err(e)) => {
                warn!(node_id = %node_id, error = %e, "Replication request failed");
                Err(format!("Request error: {}", e))
            }
            Err(_) => {
                warn!(node_id = %node_id, "Replication timeout");
                Err("Replication timeout".to_string())
            }
        }
    }

    /// Check for dead nodes and remove them
    #[instrument(skip(self))]
    pub async fn check_dead_nodes(&self) {
        let timeout = chrono::Duration::seconds(30);
        let now = chrono::Utc::now();

        let mut nodes = self.nodes.write().await;
        let dead_nodes: Vec<String> = nodes
            .iter()
            .filter(|(_, node)| now.signed_duration_since(node.last_heartbeat) > timeout)
            .map(|(id, _)| id.clone())
            .collect();

        for node_id in dead_nodes {
            warn!(node_id = %node_id, "Removing dead node");
            nodes.remove(&node_id);
        }
    }
}

/// Sharding strategy
pub enum ShardingStrategy {
    /// Hash-based sharding (default)
    Hash,
    /// Range-based sharding
    Range,
    /// Custom sharding function
    Custom(Box<ShardingFn>),
}

/// Type alias for custom sharding functions
type ShardingFn = dyn Fn(&[u8]) -> u64 + Send + Sync;

/// Replication manager
pub struct ReplicationManager {
    cluster: Arc<ClusterState>,
}

impl ReplicationManager {
    pub fn new(cluster: Arc<ClusterState>) -> Self {
        Self { cluster }
    }

    /// Start replication background task
    #[instrument(skip(self))]
    pub async fn start(&self) {
        info!("Starting replication manager");

        // Spawn heartbeat task
        let cluster = self.cluster.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                cluster.check_dead_nodes().await;
            }
        });

        info!("Replication manager started");
    }

    /// Perform write with replication
    #[instrument(skip(self, value))]
    pub async fn write(&self, key: &[u8], value: &[u8]) -> Result<(), String> {
        // Check if we're master
        if !self.cluster.is_master().await {
            return Err("Not master node".to_string());
        }

        // Replicate to other nodes
        self.cluster.replicate(key, value).await?;

        Ok(())
    }

    /// Read data from a single node via HTTP
    async fn read_from_node(
        node_addr: SocketAddr,
        node_id: &str,
        key: &[u8],
    ) -> Result<Vec<u8>, String> {
        // Build HTTP request to node's read endpoint
        let url = format!("http://{}/internal/read", node_addr);
        
        // Create payload with key
        let payload = serde_json::json!({
            "key": BASE64.encode(key),
        });

        // Send read request with timeout
        match tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            async {
                reqwest::Client::new()
                    .post(&url)
                    .json(&payload)
                    .send()
                    .await
            },
        )
        .await
        {
            Ok(Ok(response)) if response.status().is_success() => {
                // Parse response and decode data
                match response.json::<serde_json::Value>().await {
                    Ok(json) => {
                        if let Some(data_b64) = json.get("data").and_then(|v| v.as_str()) {
                            match BASE64.decode(data_b64) {
                                Ok(data) => {
                                    info!(node_id = %node_id, size = data.len(), "Read successful");
                                    Ok(data)
                                }
                                Err(e) => {
                                    error!(node_id = %node_id, error = %e, "Failed to decode data");
                                    Err(format!("Decode error: {}", e))
                                }
                            }
                        } else {
                            Err("No data field in response".to_string())
                        }
                    }
                    Err(e) => {
                        error!(node_id = %node_id, error = %e, "Failed to parse response");
                        Err(format!("Parse error: {}", e))
                    }
                }
            }
            Ok(Ok(response)) => {
                warn!(
                    node_id = %node_id,
                    status = %response.status(),
                    "Read failed with status"
                );
                Err(format!("Read failed: {}", response.status()))
            }
            Ok(Err(e)) => {
                warn!(node_id = %node_id, error = %e, "Read request failed");
                Err(format!("Request error: {}", e))
            }
            Err(_) => {
                warn!(node_id = %node_id, "Read timeout");
                Err("Read timeout".to_string())
            }
        }
    }

    /// Perform read (can use replica)
    #[instrument(skip(self))]
    pub async fn read(&self, key: &[u8]) -> Result<Vec<u8>, String> {
        let shard = self.cluster.calculate_shard(key);
        let nodes = self.cluster.get_shard_nodes(shard).await;

        if nodes.is_empty() {
            return Err("No nodes available for shard".to_string());
        }

        // If read replicas enabled, prefer replica nodes
        let target_node = if self.cluster.config.enable_read_replicas {
            nodes
                .iter()
                .find(|n| n.role == NodeRole::Replica)
                .or_else(|| nodes.first())
        } else {
            nodes.first()
        };

        if let Some(node) = target_node {
            info!(
                shard = shard,
                node_id = %node.id,
                node_addr = %node.addr,
                "Reading from node"
            );
            
            // Read from remote node via HTTP
            Self::read_from_node(node.addr, &node.id, key).await
        } else {
            Err("No target node found".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cluster_initialization() {
        let config = ReplicationConfig::default();
        let cluster = ClusterState::new("node1".to_string(), config);

        assert_eq!(cluster.get_role().await, NodeRole::Replica);

        cluster.init_as_master().await;
        assert!(cluster.is_master().await);
    }

    #[tokio::test]
    async fn test_node_management() {
        let config = ReplicationConfig::default();
        let cluster = ClusterState::new("node1".to_string(), config);

        let node = Node {
            id: "node2".to_string(),
            addr: "127.0.0.1:8081".parse().unwrap(),
            role: NodeRole::Replica,
            shard_range: None,
            last_heartbeat: chrono::Utc::now(),
        };

        cluster.add_node(node.clone()).await;

        let nodes = cluster.get_nodes().await;
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, "node2");
    }

    #[test]
    fn test_shard_calculation() {
        let config = ReplicationConfig {
            shard_count: 16,
            ..Default::default()
        };
        let cluster = ClusterState::new("node1".to_string(), config);

        let key1 = b"test_key_1";
        let key2 = b"test_key_2";

        let shard1 = cluster.calculate_shard(key1);
        let shard2 = cluster.calculate_shard(key2);

        assert!(shard1 < 16);
        assert!(shard2 < 16);

        // Same key should always map to same shard
        assert_eq!(shard1, cluster.calculate_shard(key1));
    }

    #[tokio::test]
    async fn test_replication_quorum() {
        let config = ReplicationConfig {
            replica_count: 3,
            write_quorum: 2,
            ..Default::default()
        };
        let cluster = Arc::new(ClusterState::new("node1".to_string(), config));
        cluster.init_as_master().await;

        let manager = ReplicationManager::new(cluster.clone());

        // Should fail without enough nodes
        let result = manager.write(b"key", b"value").await;
        assert!(result.is_err());
    }
}
