//! Prometheus metrics exporter for monitoring and auto-scaling
//!
//! Features:
//! - Resource metrics (CPU, Memory, Disk, Network)
//! - Database metrics (QPS, connections, latency)
//! - Cluster metrics (replication lag, shard distribution)
//! - Custom metrics for HPA

use prometheus::{
    core::{AtomicU64, GenericCounter, GenericGauge},
    Encoder, GaugeVec, HistogramVec, IntCounterVec, IntGaugeVec, Opts, Registry, TextEncoder,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, instrument};

lazy_static::lazy_static! {
    /// Global metrics registry
    pub static ref METRICS_REGISTRY: Registry = Registry::new();

    // Resource metrics
    pub static ref CPU_USAGE: GenericGauge<AtomicU64> = GenericGauge::new(
        "photondb_cpu_usage_percent",
        "CPU usage percentage"
    ).unwrap();

    pub static ref MEMORY_USAGE: GenericGauge<AtomicU64> = GenericGauge::new(
        "photondb_memory_usage_bytes",
        "Memory usage in bytes"
    ).unwrap();

    pub static ref MEMORY_USAGE_PERCENT: GenericGauge<AtomicU64> = GenericGauge::new(
        "photondb_memory_usage_percent",
        "Memory usage percentage"
    ).unwrap();

    pub static ref DISK_USAGE: GenericGauge<AtomicU64> = GenericGauge::new(
        "photondb_disk_usage_bytes",
        "Disk usage in bytes"
    ).unwrap();

    pub static ref DISK_USAGE_PERCENT: GenericGauge<AtomicU64> = GenericGauge::new(
        "photondb_disk_usage_percent",
        "Disk usage percentage"
    ).unwrap();

    pub static ref NETWORK_RX_BYTES: GenericCounter<AtomicU64> = GenericCounter::new(
        "photondb_network_rx_bytes_total",
        "Total bytes received"
    ).unwrap();

    pub static ref NETWORK_TX_BYTES: GenericCounter<AtomicU64> = GenericCounter::new(
        "photondb_network_tx_bytes_total",
        "Total bytes transmitted"
    ).unwrap();

    // Database metrics
    pub static ref QUERIES_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new("photondb_queries_total", "Total number of queries"),
        &["type", "status"]
    ).unwrap();

    pub static ref QUERIES_PER_SECOND: GenericGauge<AtomicU64> = GenericGauge::new(
        "photondb_queries_per_second",
        "Queries per second"
    ).unwrap();

    pub static ref QUERY_DURATION: HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "photondb_query_duration_seconds",
            "Query duration in seconds"
        ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0]),
        &["type"]
    ).unwrap();

    pub static ref ACTIVE_CONNECTIONS: GenericGauge<AtomicU64> = GenericGauge::new(
        "photondb_active_connections",
        "Number of active client connections"
    ).unwrap();

    pub static ref CONNECTION_ERRORS: IntCounterVec = IntCounterVec::new(
        Opts::new("photondb_connection_errors_total", "Total connection errors"),
        &["reason"]
    ).unwrap();

    // Cluster metrics
    pub static ref CLUSTER_NODES: IntGaugeVec = IntGaugeVec::new(
        Opts::new("photondb_cluster_nodes", "Number of cluster nodes"),
        &["role"]
    ).unwrap();

    pub static ref REPLICATION_LAG: GaugeVec = GaugeVec::new(
        Opts::new("photondb_replication_lag_seconds", "Replication lag in seconds"),
        &["node"]
    ).unwrap();

    pub static ref SHARD_DISTRIBUTION: IntGaugeVec = IntGaugeVec::new(
        Opts::new("photondb_shard_distribution", "Shard distribution across nodes"),
        &["node"]
    ).unwrap();

    pub static ref CLUSTER_HEALTH: GenericGauge<AtomicU64> = GenericGauge::new(
        "photondb_cluster_health",
        "Cluster health status (0=unhealthy, 1=healthy)"
    ).unwrap();

    // Storage metrics
    pub static ref TABLES_COUNT: GenericGauge<AtomicU64> = GenericGauge::new(
        "photondb_tables_count",
        "Number of tables"
    ).unwrap();

    pub static ref ROWS_COUNT: IntGaugeVec = IntGaugeVec::new(
        Opts::new("photondb_rows_count", "Number of rows per table"),
        &["database", "table"]
    ).unwrap();

    pub static ref WRITES_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new("photondb_writes_total", "Total write operations"),
        &["database", "table", "status"]
    ).unwrap();

    pub static ref READS_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new("photondb_reads_total", "Total read operations"),
        &["database", "table", "status"]
    ).unwrap();
}

/// Initialize metrics registry
pub fn init_metrics() {
    info!("Initializing Prometheus metrics");

    // Register all metrics
    METRICS_REGISTRY.register(Box::new(CPU_USAGE.clone())).ok();
    METRICS_REGISTRY.register(Box::new(MEMORY_USAGE.clone())).ok();
    METRICS_REGISTRY.register(Box::new(MEMORY_USAGE_PERCENT.clone())).ok();
    METRICS_REGISTRY.register(Box::new(DISK_USAGE.clone())).ok();
    METRICS_REGISTRY.register(Box::new(DISK_USAGE_PERCENT.clone())).ok();
    METRICS_REGISTRY.register(Box::new(NETWORK_RX_BYTES.clone())).ok();
    METRICS_REGISTRY.register(Box::new(NETWORK_TX_BYTES.clone())).ok();
    
    METRICS_REGISTRY.register(Box::new(QUERIES_TOTAL.clone())).ok();
    METRICS_REGISTRY.register(Box::new(QUERIES_PER_SECOND.clone())).ok();
    METRICS_REGISTRY.register(Box::new(QUERY_DURATION.clone())).ok();
    METRICS_REGISTRY.register(Box::new(ACTIVE_CONNECTIONS.clone())).ok();
    METRICS_REGISTRY.register(Box::new(CONNECTION_ERRORS.clone())).ok();
    
    METRICS_REGISTRY.register(Box::new(CLUSTER_NODES.clone())).ok();
    METRICS_REGISTRY.register(Box::new(REPLICATION_LAG.clone())).ok();
    METRICS_REGISTRY.register(Box::new(SHARD_DISTRIBUTION.clone())).ok();
    METRICS_REGISTRY.register(Box::new(CLUSTER_HEALTH.clone())).ok();
    
    METRICS_REGISTRY.register(Box::new(TABLES_COUNT.clone())).ok();
    METRICS_REGISTRY.register(Box::new(ROWS_COUNT.clone())).ok();
    METRICS_REGISTRY.register(Box::new(WRITES_TOTAL.clone())).ok();
    METRICS_REGISTRY.register(Box::new(READS_TOTAL.clone())).ok();

    info!("Metrics initialized successfully");
}

/// Metrics collector
pub struct MetricsCollector {
    last_query_count: Arc<RwLock<u64>>,
    last_update: Arc<RwLock<std::time::Instant>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            last_query_count: Arc::new(RwLock::new(0)),
            last_update: Arc::new(RwLock::new(std::time::Instant::now())),
        }
    }

    /// Update resource metrics
    #[instrument(skip(self))]
    pub async fn update_resource_metrics(
        &self,
        cpu: f32,
        memory_bytes: u64,
        memory_percent: f32,
        disk_bytes: u64,
        disk_percent: f32,
    ) {
        CPU_USAGE.set((cpu * 100.0) as u64);
        MEMORY_USAGE.set(memory_bytes);
        MEMORY_USAGE_PERCENT.set(memory_percent as u64);
        DISK_USAGE.set(disk_bytes);
        DISK_USAGE_PERCENT.set(disk_percent as u64);
    }

    /// Update network metrics
    pub fn update_network_metrics(&self, rx_bytes: u64, tx_bytes: u64) {
        NETWORK_RX_BYTES.inc_by(rx_bytes);
        NETWORK_TX_BYTES.inc_by(tx_bytes);
    }

    /// Record query
    #[instrument(skip(self))]
    pub async fn record_query(
        &self,
        query_type: &str,
        duration: f64,
        success: bool,
    ) {
        let status = if success { "success" } else { "error" };
        QUERIES_TOTAL.with_label_values(&[query_type, status]).inc();
        QUERY_DURATION.with_label_values(&[query_type]).observe(duration);

        // Update QPS
        let mut last_count = self.last_query_count.write().await;
        let mut last_time = self.last_update.write().await;
        
        *last_count += 1;
        let elapsed = last_time.elapsed().as_secs_f64();
        
        if elapsed >= 1.0 {
            let qps = *last_count as f64 / elapsed;
            QUERIES_PER_SECOND.set(qps as u64);
            *last_count = 0;
            *last_time = std::time::Instant::now();
        }
    }

    /// Update connection metrics
    pub fn update_connections(&self, active: u64) {
        ACTIVE_CONNECTIONS.set(active);
    }

    /// Record connection error
    pub fn record_connection_error(&self, reason: &str) {
        CONNECTION_ERRORS.with_label_values(&[reason]).inc();
    }

    /// Update cluster metrics
    pub fn update_cluster_metrics(
        &self,
        masters: u64,
        replicas: u64,
        candidates: u64,
        healthy: bool,
    ) {
        CLUSTER_NODES.with_label_values(&["master"]).set(masters as i64);
        CLUSTER_NODES.with_label_values(&["replica"]).set(replicas as i64);
        CLUSTER_NODES.with_label_values(&["candidate"]).set(candidates as i64);
        CLUSTER_HEALTH.set(if healthy { 1 } else { 0 });
    }

    /// Update replication lag
    pub fn update_replication_lag(&self, node: &str, lag_seconds: f64) {
        REPLICATION_LAG.with_label_values(&[node]).set(lag_seconds);
    }

    /// Update shard distribution
    pub fn update_shard_distribution(&self, node: &str, shard_count: i64) {
        SHARD_DISTRIBUTION.with_label_values(&[node]).set(shard_count);
    }

    /// Update storage metrics
    pub fn update_storage_metrics(
        &self,
        tables_count: u64,
        database: &str,
        table: &str,
        rows: i64,
    ) {
        TABLES_COUNT.set(tables_count);
        ROWS_COUNT.with_label_values(&[database, table]).set(rows);
    }

    /// Record write operation
    pub fn record_write(&self, database: &str, table: &str, success: bool) {
        let status = if success { "success" } else { "error" };
        WRITES_TOTAL.with_label_values(&[database, table, status]).inc();
    }

    /// Record read operation
    pub fn record_read(&self, database: &str, table: &str, success: bool) {
        let status = if success { "success" } else { "error" };
        READS_TOTAL.with_label_values(&[database, table, status]).inc();
    }

    /// Export metrics in Prometheus format
    pub fn export_metrics(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        let metric_families = METRICS_REGISTRY.gather();
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer)?;
        
        String::from_utf8(buffer).map_err(|e| {
            prometheus::Error::Msg(format!("UTF-8 conversion error: {}", e))
        })
    }
}

/// Export all metrics in Prometheus text format (module-level function)
pub fn export_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = METRICS_REGISTRY.gather();
    
    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        error!(error = %e, "Failed to encode metrics");
        return String::from("# Error encoding metrics\n");
    }
    
    String::from_utf8(buffer).unwrap_or_else(|_| String::from("# Error converting metrics\n"))
}

impl MetricsCollector {
    /// Start metrics collection background task
    #[instrument(skip(self))]
    pub async fn start_collection(&self, interval_seconds: u64) {
        info!(interval = interval_seconds, "Starting metrics collection");

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(interval_seconds)).await;

            // Collect system metrics
            if let Err(e) = self.collect_system_metrics().await {
                error!(error = %e, "Failed to collect system metrics");
            }
        }
    }

    /// Collect system metrics
    async fn collect_system_metrics(&self) -> Result<(), String> {
        // Get CPU usage
        #[cfg(target_os = "linux")]
        {
            use sysinfo::{System, SystemExt};
            let mut sys = System::new_all();
            sys.refresh_all();

            let cpu_usage = sys.global_cpu_info().cpu_usage();
            let memory_usage = sys.used_memory();
            let total_memory = sys.total_memory();
            let memory_percent = (memory_usage as f32 / total_memory as f32) * 100.0;

            self.update_resource_metrics(
                cpu_usage / 100.0,
                memory_usage,
                memory_percent,
                0, // disk_bytes - would need additional collection
                0.0, // disk_percent
            ).await;
        }

        Ok(())
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics HTTP handler (for /metrics endpoint)
pub async fn metrics_handler() -> Result<String, String> {
    let collector = MetricsCollector::new();
    collector.export_metrics().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_metrics() {
        init_metrics();
        // Should not panic
    }

    #[tokio::test]
    async fn test_update_resource_metrics() {
        let collector = MetricsCollector::new();
        collector.update_resource_metrics(0.5, 1024, 50.0, 2048, 75.0).await;

        assert_eq!(CPU_USAGE.get(), 50);
        assert_eq!(MEMORY_USAGE.get(), 1024);
    }

    #[tokio::test]
    async fn test_record_query() {
        let collector = MetricsCollector::new();
        collector.record_query("SELECT", 0.1, true).await;

        // Query should be recorded
        let metrics = collector.export_metrics().unwrap();
        assert!(metrics.contains("photondb_queries_total"));
    }

    #[test]
    fn test_update_connections() {
        let collector = MetricsCollector::new();
        collector.update_connections(42);

        assert_eq!(ACTIVE_CONNECTIONS.get(), 42);
    }

    #[test]
    fn test_cluster_metrics() {
        let collector = MetricsCollector::new();
        collector.update_cluster_metrics(1, 2, 0, true);

        assert_eq!(CLUSTER_HEALTH.get(), 1);
    }

    #[test]
    fn test_export_metrics() {
        init_metrics();
        let collector = MetricsCollector::new();
        
        let output = collector.export_metrics().unwrap();
        assert!(output.contains("photondb_"));
    }
}
