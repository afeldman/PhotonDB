//! Integration tests for Kubernetes cluster scaling

#[cfg(test)]
mod k8s_scaling_tests {
    use rethinkdb::cluster::{
        health::{ClusterHealth, DatabaseHealth, HealthChecker},
        metrics::{init_metrics, MetricsCollector, ResourceMetrics},
        scaling::{
            AutoScaler, HorizontalScalingConfig, ScalingDecision, ScalingStrategy,
            VerticalScalingConfig,
        },
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn test_horizontal_scaling_workflow() {
        // Initialize auto-scaler
        let config = ScalingStrategy::Horizontal(HorizontalScalingConfig {
            min_replicas: 3,
            max_replicas: 10,
            target_cpu_utilization: 70,
            target_memory_utilization: 80,
            ..Default::default()
        });

        let scaler = AutoScaler::new(config);

        // Simulate high load
        let high_load_metrics = ResourceMetrics {
            cpu_utilization: 85.0,
            memory_utilization: 90.0,
            queries_per_second: 1500.0,
            active_connections: 600,
            ..Default::default()
        };

        scaler.update_metrics(high_load_metrics).await;
        let decision = scaler.evaluate().await;

        // Should scale up
        assert!(matches!(decision, ScalingDecision::ScaleUp(_)));

        // Apply scaling
        scaler.apply_scaling(decision).await.unwrap();
        let replicas = scaler.get_replica_count().await;
        assert!(replicas > 3);

        // Simulate low load
        let low_load_metrics = ResourceMetrics {
            cpu_utilization: 20.0,
            memory_utilization: 25.0,
            queries_per_second: 100.0,
            active_connections: 50,
            ..Default::default()
        };

        scaler.update_metrics(low_load_metrics).await;
        let decision = scaler.evaluate().await;

        // Should scale down or stay stable
        assert!(matches!(
            decision,
            ScalingDecision::ScaleDown(_) | ScalingDecision::NoAction
        ));
    }

    #[tokio::test]
    async fn test_vertical_scaling_workflow() {
        let config = ScalingStrategy::Vertical(VerticalScalingConfig {
            min_cpu: 0.5,
            max_cpu: 8.0,
            min_memory: 1.0,
            max_memory: 16.0,
            ..Default::default()
        });

        let scaler = AutoScaler::new(config);

        // High resource usage
        let metrics = ResourceMetrics {
            cpu_utilization: 95.0,
            memory_utilization: 92.0,
            ..Default::default()
        };

        scaler.update_metrics(metrics).await;
        let decision = scaler.evaluate().await;

        // Should recommend vertical scale up
        assert!(matches!(decision, ScalingDecision::VerticalScaleUp { .. }));

        if let ScalingDecision::VerticalScaleUp { cpu, memory } = decision {
            assert!(cpu > 1.0);
            assert!(memory > 2.0);
        }
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        init_metrics();
        let collector = MetricsCollector::new();

        // Update metrics
        collector
            .update_resource_metrics(0.75, 2_000_000_000, 80.0, 50_000_000_000, 60.0)
            .await;

        collector.update_connections(42);
        collector.record_query("SELECT", 0.05, true).await;
        collector.record_write("testdb", "users", true);
        collector.record_read("testdb", "users", true);

        collector.update_cluster_metrics(1, 2, 0, true);
        collector.update_replication_lag("node-1", 0.5);

        // Export metrics
        let output = collector.export_metrics().unwrap();
        assert!(output.contains("rethinkdb_cpu_usage_percent"));
        assert!(output.contains("rethinkdb_memory_usage_bytes"));
        assert!(output.contains("rethinkdb_active_connections"));
    }

    #[tokio::test]
    async fn test_health_checks() {
        let health = HealthChecker::new();

        // Initially not ready
        assert!(!health.check_readiness().await);
        assert!(!health.check_startup().await);
        assert!(health.check_liveness().await); // Always alive

        // Mark startup complete
        health.set_startup_complete().await;
        assert!(health.check_startup().await);

        // Update health status
        health
            .update_database_health(DatabaseHealth {
                status: "healthy".to_string(),
                tables_count: 5,
                active_queries: 10,
                connections: 20,
            })
            .await;

        health
            .update_cluster_health(ClusterHealth {
                status: "healthy".to_string(),
                nodes: 3,
                masters: 1,
                replicas: 2,
                replication_lag_ms: 5.0,
            })
            .await;

        health.set_ready(true).await;

        // Now should be ready
        assert!(health.check_readiness().await);

        // Get full status
        let status = health.get_status().await;
        assert_eq!(status.status, "healthy");
        assert_eq!(status.cluster.nodes, 3);
        assert_eq!(status.database.tables_count, 5);
    }

    #[tokio::test]
    async fn test_hybrid_scaling() {
        let config = ScalingStrategy::Hybrid {
            horizontal: HorizontalScalingConfig {
                min_replicas: 3,
                max_replicas: 10,
                target_cpu_utilization: 70,
                ..Default::default()
            },
            vertical: VerticalScalingConfig {
                min_cpu: 0.5,
                max_cpu: 8.0,
                min_memory: 1.0,
                max_memory: 16.0,
                ..Default::default()
            },
        };

        let scaler = AutoScaler::new(config);

        // High load - should trigger horizontal scaling first
        let metrics = ResourceMetrics {
            cpu_utilization: 85.0,
            memory_utilization: 85.0,
            ..Default::default()
        };

        scaler.update_metrics(metrics).await;
        let decision = scaler.evaluate().await;

        // Hybrid mode prefers horizontal scaling
        assert!(matches!(decision, ScalingDecision::ScaleUp(_)));
    }

    #[tokio::test]
    async fn test_custom_metrics_scaling() {
        use rethinkdb::cluster::scaling::{CustomMetric, MetricType};

        let config = ScalingStrategy::Horizontal(HorizontalScalingConfig {
            min_replicas: 3,
            max_replicas: 10,
            target_cpu_utilization: 70,
            custom_metrics: vec![
                CustomMetric {
                    name: "queries_per_second".to_string(),
                    target_value: 1000.0,
                    metric_type: MetricType::AverageValue,
                },
                CustomMetric {
                    name: "active_connections".to_string(),
                    target_value: 500.0,
                    metric_type: MetricType::AverageValue,
                },
            ],
            ..Default::default()
        });

        let scaler = AutoScaler::new(config);

        // Exceed custom metric threshold
        let metrics = ResourceMetrics {
            cpu_utilization: 50.0, // Normal CPU
            memory_utilization: 50.0, // Normal memory
            queries_per_second: 1500.0, // HIGH QPS
            active_connections: 600, // HIGH connections
            ..Default::default()
        };

        scaler.update_metrics(metrics).await;
        let decision = scaler.evaluate().await;

        // Should scale up due to custom metrics
        assert!(matches!(decision, ScalingDecision::ScaleUp(_)));
    }

    #[tokio::test]
    async fn test_scaling_stabilization() {
        let config = ScalingStrategy::Horizontal(HorizontalScalingConfig {
            min_replicas: 3,
            max_replicas: 10,
            target_cpu_utilization: 70,
            scale_up_stabilization: 60,
            scale_down_stabilization: 300,
            ..Default::default()
        });

        let scaler = Arc::new(AutoScaler::new(config));

        // Simulate fluctuating load
        for i in 0..5 {
            let load = if i % 2 == 0 { 85.0 } else { 30.0 };

            let metrics = ResourceMetrics {
                cpu_utilization: load,
                memory_utilization: load,
                ..Default::default()
            };

            scaler.update_metrics(metrics).await;
            let decision = scaler.evaluate().await;
            scaler.apply_scaling(decision).await.unwrap();

            // Small delay to simulate time passing
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Final replica count should be stable
        let replicas = scaler.get_replica_count().await;
        assert!(replicas >= 3 && replicas <= 10);
    }

    #[tokio::test]
    async fn test_resource_limits() {
        let config = ScalingStrategy::Horizontal(HorizontalScalingConfig {
            min_replicas: 3,
            max_replicas: 5, // Low max for testing
            target_cpu_utilization: 70,
            ..Default::default()
        });

        let scaler = AutoScaler::new(config);

        // Very high load
        let metrics = ResourceMetrics {
            cpu_utilization: 99.0,
            memory_utilization: 99.0,
            ..Default::default()
        };

        scaler.update_metrics(metrics).await;

        // Scale up multiple times
        for _ in 0..10 {
            let decision = scaler.evaluate().await;
            scaler.apply_scaling(decision).await.unwrap();
        }

        // Should not exceed max_replicas
        let replicas = scaler.get_replica_count().await;
        assert!(replicas <= 5);
    }
}
