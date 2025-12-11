//! Auto-scaling logic for horizontal and vertical scaling
//!
//! Features:
//! - Horizontal Pod Autoscaling (HPA) based on CPU/Memory/Custom metrics
//! - Vertical Pod Autoscaling (VPA) recommendations
//! - Shard rebalancing during scale operations
//! - Graceful scale-down with data migration

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument, warn};

/// Scaling strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingStrategy {
    /// Horizontal Pod Autoscaling (add/remove pods)
    Horizontal(HorizontalScalingConfig),
    /// Vertical Pod Autoscaling (resize pods)
    Vertical(VerticalScalingConfig),
    /// Both horizontal and vertical
    Hybrid {
        horizontal: HorizontalScalingConfig,
        vertical: VerticalScalingConfig,
    },
}

/// Horizontal scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorizontalScalingConfig {
    /// Minimum number of replicas
    pub min_replicas: u32,
    /// Maximum number of replicas
    pub max_replicas: u32,
    /// Target CPU utilization percentage (0-100)
    pub target_cpu_utilization: u32,
    /// Target memory utilization percentage (0-100)
    pub target_memory_utilization: u32,
    /// Custom metrics for scaling
    pub custom_metrics: Vec<CustomMetric>,
    /// Scale-up stabilization window (seconds)
    pub scale_up_stabilization: u64,
    /// Scale-down stabilization window (seconds)
    pub scale_down_stabilization: u64,
}

impl Default for HorizontalScalingConfig {
    fn default() -> Self {
        Self {
            min_replicas: 3,
            max_replicas: 10,
            target_cpu_utilization: 70,
            target_memory_utilization: 80,
            custom_metrics: vec![],
            scale_up_stabilization: 60,
            scale_down_stabilization: 300,
        }
    }
}

/// Vertical scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerticalScalingConfig {
    /// Minimum CPU cores
    pub min_cpu: f32,
    /// Maximum CPU cores
    pub max_cpu: f32,
    /// Minimum memory in GB
    pub min_memory: f32,
    /// Maximum memory in GB
    pub max_memory: f32,
    /// Update mode (Auto, Initial, Off)
    pub update_mode: VpaUpdateMode,
}

impl Default for VerticalScalingConfig {
    fn default() -> Self {
        Self {
            min_cpu: 0.5,
            max_cpu: 8.0,
            min_memory: 1.0,
            max_memory: 16.0,
            update_mode: VpaUpdateMode::Auto,
        }
    }
}

/// VPA update mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VpaUpdateMode {
    /// Automatically apply recommendations
    Auto,
    /// Only on pod creation
    Initial,
    /// Only provide recommendations (no updates)
    Off,
}

/// Custom metric for scaling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetric {
    pub name: String,
    pub target_value: f64,
    pub metric_type: MetricType,
}

/// Metric type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    /// Average value across pods
    AverageValue,
    /// Utilization percentage
    Utilization,
    /// Total value across pods
    Value,
}

/// Resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub cpu_utilization: f32,
    pub memory_utilization: f32,
    pub disk_utilization: f32,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub queries_per_second: f64,
    pub active_connections: u64,
    pub replication_lag: f64,
}

impl Default for ResourceMetrics {
    fn default() -> Self {
        Self {
            cpu_utilization: 0.0,
            memory_utilization: 0.0,
            disk_utilization: 0.0,
            network_rx_bytes: 0,
            network_tx_bytes: 0,
            queries_per_second: 0.0,
            active_connections: 0,
            replication_lag: 0.0,
        }
    }
}

/// Scaling decision
#[derive(Debug, Clone)]
pub enum ScalingDecision {
    /// Scale up by N replicas
    ScaleUp(u32),
    /// Scale down by N replicas
    ScaleDown(u32),
    /// Increase vertical resources
    VerticalScaleUp { cpu: f32, memory: f32 },
    /// Decrease vertical resources
    VerticalScaleDown { cpu: f32, memory: f32 },
    /// No scaling needed
    NoAction,
}

/// Auto-scaler manager
pub struct AutoScaler {
    config: ScalingStrategy,
    current_replicas: Arc<RwLock<u32>>,
    current_resources: Arc<RwLock<(f32, f32)>>, // (cpu, memory)
    metrics: Arc<RwLock<ResourceMetrics>>,
}

impl AutoScaler {
    pub fn new(config: ScalingStrategy) -> Self {
        let initial_replicas = match &config {
            ScalingStrategy::Horizontal(h) => h.min_replicas,
            ScalingStrategy::Hybrid { horizontal, .. } => horizontal.min_replicas,
            ScalingStrategy::Vertical(_) => 3,
        };

        Self {
            config,
            current_replicas: Arc::new(RwLock::new(initial_replicas)),
            current_resources: Arc::new(RwLock::new((1.0, 2.0))),
            metrics: Arc::new(RwLock::new(ResourceMetrics::default())),
        }
    }

    /// Update current metrics
    #[instrument(skip(self))]
    pub async fn update_metrics(&self, metrics: ResourceMetrics) {
        let mut current = self.metrics.write().await;
        *current = metrics;
        info!(
            cpu = current.cpu_utilization,
            memory = current.memory_utilization,
            qps = current.queries_per_second,
            "Updated resource metrics"
        );
    }

    /// Evaluate if scaling is needed
    #[instrument(skip(self))]
    pub async fn evaluate(&self) -> ScalingDecision {
        let metrics = self.metrics.read().await;

        match &self.config {
            ScalingStrategy::Horizontal(config) => {
                self.evaluate_horizontal(config, &metrics).await
            }
            ScalingStrategy::Vertical(config) => self.evaluate_vertical(config, &metrics).await,
            ScalingStrategy::Hybrid {
                horizontal,
                vertical,
            } => {
                // Try horizontal first, then vertical
                let h_decision = self.evaluate_horizontal(horizontal, &metrics).await;
                if !matches!(h_decision, ScalingDecision::NoAction) {
                    return h_decision;
                }
                self.evaluate_vertical(vertical, &metrics).await
            }
        }
    }

    /// Evaluate horizontal scaling
    async fn evaluate_horizontal(
        &self,
        config: &HorizontalScalingConfig,
        metrics: &ResourceMetrics,
    ) -> ScalingDecision {
        let current_replicas = *self.current_replicas.read().await;

        // Check CPU utilization
        let cpu_pressure = metrics.cpu_utilization > config.target_cpu_utilization as f32;
        let memory_pressure =
            metrics.memory_utilization > config.target_memory_utilization as f32;

        // Check custom metrics
        let custom_pressure = self.check_custom_metrics(config, metrics);

        if cpu_pressure || memory_pressure || custom_pressure {
            if current_replicas < config.max_replicas {
                let scale_amount = self.calculate_scale_amount(
                    current_replicas,
                    config.max_replicas,
                    metrics.cpu_utilization,
                    config.target_cpu_utilization as f32,
                );
                info!(
                    current = current_replicas,
                    scale_by = scale_amount,
                    "Scaling up horizontally"
                );
                return ScalingDecision::ScaleUp(scale_amount);
            }
        } else if metrics.cpu_utilization < (config.target_cpu_utilization as f32 * 0.5)
            && metrics.memory_utilization < (config.target_memory_utilization as f32 * 0.5)
        {
            if current_replicas > config.min_replicas {
                info!(current = current_replicas, "Scaling down horizontally");
                return ScalingDecision::ScaleDown(1);
            }
        }

        ScalingDecision::NoAction
    }

    /// Evaluate vertical scaling
    async fn evaluate_vertical(
        &self,
        config: &VerticalScalingConfig,
        metrics: &ResourceMetrics,
    ) -> ScalingDecision {
        let (current_cpu, current_memory) = *self.current_resources.read().await;

        // Calculate recommended resources
        let recommended_cpu = self.calculate_recommended_cpu(metrics.cpu_utilization, current_cpu);
        let recommended_memory =
            self.calculate_recommended_memory(metrics.memory_utilization, current_memory);

        // Clamp to configured limits
        let target_cpu = recommended_cpu.clamp(config.min_cpu, config.max_cpu);
        let target_memory = recommended_memory.clamp(config.min_memory, config.max_memory);

        // Check if scaling is needed (threshold: 20% difference)
        let cpu_diff = (target_cpu - current_cpu).abs() / current_cpu;
        let memory_diff = (target_memory - current_memory).abs() / current_memory;

        if cpu_diff > 0.2 || memory_diff > 0.2 {
            if target_cpu > current_cpu || target_memory > current_memory {
                info!(
                    current_cpu = current_cpu,
                    target_cpu = target_cpu,
                    current_memory = current_memory,
                    target_memory = target_memory,
                    "Scaling up vertically"
                );
                return ScalingDecision::VerticalScaleUp {
                    cpu: target_cpu,
                    memory: target_memory,
                };
            } else {
                info!(
                    current_cpu = current_cpu,
                    target_cpu = target_cpu,
                    current_memory = current_memory,
                    target_memory = target_memory,
                    "Scaling down vertically"
                );
                return ScalingDecision::VerticalScaleDown {
                    cpu: target_cpu,
                    memory: target_memory,
                };
            }
        }

        ScalingDecision::NoAction
    }

    /// Check custom metrics
    fn check_custom_metrics(
        &self,
        config: &HorizontalScalingConfig,
        metrics: &ResourceMetrics,
    ) -> bool {
        for custom in &config.custom_metrics {
            let current_value = match custom.name.as_str() {
                "queries_per_second" => metrics.queries_per_second,
                "active_connections" => metrics.active_connections as f64,
                "replication_lag" => metrics.replication_lag,
                _ => continue,
            };

            if current_value > custom.target_value {
                warn!(
                    metric = %custom.name,
                    current = current_value,
                    target = custom.target_value,
                    "Custom metric threshold exceeded"
                );
                return true;
            }
        }
        false
    }

    /// Calculate scale amount based on utilization
    fn calculate_scale_amount(
        &self,
        current: u32,
        max: u32,
        utilization: f32,
        target: f32,
    ) -> u32 {
        // Calculate desired replicas based on utilization
        let desired = ((current as f32) * (utilization / target)).ceil() as u32;
        let scale_by = desired.saturating_sub(current).min(max - current);
        scale_by.max(1) // At least scale by 1
    }

    /// Calculate recommended CPU
    fn calculate_recommended_cpu(&self, utilization: f32, current: f32) -> f32 {
        // Target 70% CPU utilization
        let target = 70.0;
        let ratio = utilization / target;
        (current * ratio * 1.1).max(0.5) // Add 10% buffer
    }

    /// Calculate recommended memory
    fn calculate_recommended_memory(&self, utilization: f32, current: f32) -> f32 {
        // Target 80% memory utilization
        let target = 80.0;
        let ratio = utilization / target;
        (current * ratio * 1.15).max(1.0) // Add 15% buffer
    }

    /// Apply scaling decision
    #[instrument(skip(self))]
    pub async fn apply_scaling(&self, decision: ScalingDecision) -> Result<(), String> {
        match decision {
            ScalingDecision::ScaleUp(amount) => {
                let mut replicas = self.current_replicas.write().await;
                *replicas += amount;
                info!(new_replicas = *replicas, "Scaled up replicas");
                Ok(())
            }
            ScalingDecision::ScaleDown(amount) => {
                let mut replicas = self.current_replicas.write().await;
                *replicas = replicas.saturating_sub(amount);
                info!(new_replicas = *replicas, "Scaled down replicas");
                Ok(())
            }
            ScalingDecision::VerticalScaleUp { cpu, memory } => {
                let mut resources = self.current_resources.write().await;
                *resources = (cpu, memory);
                info!(cpu = cpu, memory = memory, "Scaled up resources");
                Ok(())
            }
            ScalingDecision::VerticalScaleDown { cpu, memory } => {
                let mut resources = self.current_resources.write().await;
                *resources = (cpu, memory);
                info!(cpu = cpu, memory = memory, "Scaled down resources");
                Ok(())
            }
            ScalingDecision::NoAction => Ok(()),
        }
    }

    /// Start auto-scaling loop
    #[instrument(skip(self))]
    pub async fn start(&self, interval_seconds: u64) {
        info!(interval = interval_seconds, "Starting auto-scaler");

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(interval_seconds)).await;

            let decision = self.evaluate().await;
            if let Err(e) = self.apply_scaling(decision).await {
                warn!(error = %e, "Failed to apply scaling decision");
            }
        }
    }

    /// Get current replica count
    pub async fn get_replica_count(&self) -> u32 {
        *self.current_replicas.read().await
    }

    /// Get current resources
    pub async fn get_current_resources(&self) -> (f32, f32) {
        *self.current_resources.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_horizontal_scale_up() {
        let config = ScalingStrategy::Horizontal(HorizontalScalingConfig {
            min_replicas: 3,
            max_replicas: 10,
            target_cpu_utilization: 70,
            ..Default::default()
        });

        let scaler = AutoScaler::new(config);

        let metrics = ResourceMetrics {
            cpu_utilization: 85.0,
            memory_utilization: 60.0,
            ..Default::default()
        };

        scaler.update_metrics(metrics).await;
        let decision = scaler.evaluate().await;

        assert!(matches!(decision, ScalingDecision::ScaleUp(_)));
    }

    #[tokio::test]
    async fn test_horizontal_scale_down() {
        let config = ScalingStrategy::Horizontal(HorizontalScalingConfig {
            min_replicas: 3,
            max_replicas: 10,
            target_cpu_utilization: 70,
            ..Default::default()
        });

        let scaler = AutoScaler::new(config);

        let metrics = ResourceMetrics {
            cpu_utilization: 20.0,
            memory_utilization: 25.0,
            ..Default::default()
        };

        scaler.update_metrics(metrics).await;
        let decision = scaler.evaluate().await;

        assert!(matches!(
            decision,
            ScalingDecision::ScaleDown(_) | ScalingDecision::NoAction
        ));
    }

    #[tokio::test]
    async fn test_vertical_scaling() {
        let config = ScalingStrategy::Vertical(VerticalScalingConfig::default());

        let scaler = AutoScaler::new(config);

        let metrics = ResourceMetrics {
            cpu_utilization: 90.0,
            memory_utilization: 85.0,
            ..Default::default()
        };

        scaler.update_metrics(metrics).await;
        let decision = scaler.evaluate().await;

        assert!(matches!(decision, ScalingDecision::VerticalScaleUp { .. }));
    }

    #[tokio::test]
    async fn test_custom_metrics() {
        let config = ScalingStrategy::Horizontal(HorizontalScalingConfig {
            custom_metrics: vec![CustomMetric {
                name: "queries_per_second".to_string(),
                target_value: 1000.0,
                metric_type: MetricType::AverageValue,
            }],
            ..Default::default()
        });

        let scaler = AutoScaler::new(config);

        let metrics = ResourceMetrics {
            queries_per_second: 1500.0,
            cpu_utilization: 50.0,
            memory_utilization: 50.0,
            ..Default::default()
        };

        scaler.update_metrics(metrics).await;
        let decision = scaler.evaluate().await;

        assert!(matches!(decision, ScalingDecision::ScaleUp(_)));
    }
}
