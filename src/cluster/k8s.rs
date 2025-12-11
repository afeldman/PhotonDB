//! Kubernetes client integration for managing cluster resources
//!
//! Features:
//! - StatefulSet management for database pods
//! - Horizontal Pod Autoscaler (HPA) configuration
//! - Pod Disruption Budget (PDB) for high availability
//! - Service discovery and health monitoring

use k8s_openapi::api::{
    apps::v1::{StatefulSet, StatefulSetSpec},
    autoscaling::v2::{HorizontalPodAutoscaler, HorizontalPodAutoscalerSpec, MetricSpec},
    core::v1::{
        Container, ContainerPort, EnvVar, PersistentVolumeClaim, Pod, PodSpec,
        PodTemplateSpec, Probe, ResourceRequirements, TCPSocketAction,
    },
    policy::v1::{PodDisruptionBudget, PodDisruptionBudgetSpec},
};
use kube::{
    api::{Api, ListParams, Patch, PatchParams, PostParams},
    Client,
};
use std::collections::BTreeMap;
use tracing::{info, instrument, warn};

/// Kubernetes cluster manager
pub struct K8sClusterManager {
    client: Client,
    namespace: String,
    app_name: String,
}

impl K8sClusterManager {
    /// Create a new Kubernetes cluster manager
    pub async fn new(namespace: String, app_name: String) -> Result<Self, kube::Error> {
        let client = Client::try_default().await?;
        Ok(Self {
            client,
            namespace,
            app_name,
        })
    }

    /// Deploy or update StatefulSet
    #[instrument(skip(self))]
    pub async fn deploy_statefulset(
        &self,
        replicas: i32,
        cpu: String,
        memory: String,
    ) -> Result<(), kube::Error> {
        info!(
            replicas = replicas,
            cpu = %cpu,
            memory = %memory,
            "Deploying StatefulSet"
        );

        let statefulset = self.build_statefulset(replicas, cpu, memory);
        let api: Api<StatefulSet> = Api::namespaced(self.client.clone(), &self.namespace);

        // Try to get existing StatefulSet
        match api.get(&self.app_name).await {
            Ok(_) => {
                // Update existing
                let patch = Patch::Apply(&statefulset);
                let pp = PatchParams::apply("rethinkdb-controller");
                api.patch(&self.app_name, &pp, &patch).await?;
                info!("Updated existing StatefulSet");
            }
            Err(_) => {
                // Create new
                let pp = PostParams::default();
                api.create(&pp, &statefulset).await?;
                info!("Created new StatefulSet");
            }
        }

        Ok(())
    }

    /// Build StatefulSet configuration
    fn build_statefulset(&self, replicas: i32, cpu: String, memory: String) -> StatefulSet {
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.app_name.clone());
        labels.insert("component".to_string(), "database".to_string());

        let mut resources = BTreeMap::new();
        let mut limits = BTreeMap::new();
        limits.insert("cpu".to_string(), k8s_openapi::apimachinery::pkg::api::resource::Quantity(cpu.clone()));
        limits.insert("memory".to_string(), k8s_openapi::apimachinery::pkg::api::resource::Quantity(memory.clone()));
        resources.insert("limits".to_string(), limits.clone());
        resources.insert("requests".to_string(), limits);

        StatefulSet {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some(self.app_name.clone()),
                namespace: Some(self.namespace.clone()),
                labels: Some(labels.clone()),
                ..Default::default()
            },
            spec: Some(StatefulSetSpec {
                replicas: Some(replicas),
                selector: k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector {
                    match_labels: Some(labels.clone()),
                    ..Default::default()
                },
                service_name: self.app_name.clone(),
                template: PodTemplateSpec {
                    metadata: Some(k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                        labels: Some(labels),
                        ..Default::default()
                    }),
                    spec: Some(PodSpec {
                        containers: vec![Container {
                            name: "rethinkdb".to_string(),
                            image: Some(format!("{}:latest", self.app_name)),
                            ports: Some(vec![
                                ContainerPort {
                                    container_port: 28015,
                                    name: Some("client".to_string()),
                                    ..Default::default()
                                },
                                ContainerPort {
                                    container_port: 29015,
                                    name: Some("cluster".to_string()),
                                    ..Default::default()
                                },
                                ContainerPort {
                                    container_port: 8080,
                                    name: Some("http".to_string()),
                                    ..Default::default()
                                },
                            ]),
                            env: Some(vec![
                                EnvVar {
                                    name: "PHOTONDB_CLUSTER_MODE".to_string(),
                                    value: Some("kubernetes".to_string()),
                                    ..Default::default()
                                },
                                EnvVar {
                                    name: "POD_NAMESPACE".to_string(),
                                    value_from: Some(k8s_openapi::api::core::v1::EnvVarSource {
                                        field_ref: Some(k8s_openapi::api::core::v1::ObjectFieldSelector {
                                            field_path: "metadata.namespace".to_string(),
                                            ..Default::default()
                                        }),
                                        ..Default::default()
                                    }),
                                    ..Default::default()
                                },
                                EnvVar {
                                    name: "POD_NAME".to_string(),
                                    value_from: Some(k8s_openapi::api::core::v1::EnvVarSource {
                                        field_ref: Some(k8s_openapi::api::core::v1::ObjectFieldSelector {
                                            field_path: "metadata.name".to_string(),
                                            ..Default::default()
                                        }),
                                        ..Default::default()
                                    }),
                                    ..Default::default()
                                },
                            ]),
                            resources: Some(ResourceRequirements {
                                limits: Some(resources.get("limits").cloned().unwrap_or_default()),
                                requests: Some(resources.get("requests").cloned().unwrap_or_default()),
                                ..Default::default()
                            }),
                            liveness_probe: Some(Probe {
                                tcp_socket: Some(TCPSocketAction {
                                    port: k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(28015),
                                    ..Default::default()
                                }),
                                initial_delay_seconds: Some(30),
                                period_seconds: Some(10),
                                timeout_seconds: Some(5),
                                failure_threshold: Some(3),
                                ..Default::default()
                            }),
                            readiness_probe: Some(Probe {
                                tcp_socket: Some(TCPSocketAction {
                                    port: k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(28015),
                                    ..Default::default()
                                }),
                                initial_delay_seconds: Some(10),
                                period_seconds: Some(5),
                                timeout_seconds: Some(3),
                                failure_threshold: Some(3),
                                ..Default::default()
                            }),
                            volume_mounts: Some(vec![k8s_openapi::api::core::v1::VolumeMount {
                                name: "data".to_string(),
                                mount_path: "/data".to_string(),
                                ..Default::default()
                            }]),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                },
                volume_claim_templates: Some(vec![PersistentVolumeClaim {
                    metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                        name: Some("data".to_string()),
                        ..Default::default()
                    },
                    spec: Some(k8s_openapi::api::core::v1::PersistentVolumeClaimSpec {
                        access_modes: Some(vec!["ReadWriteOnce".to_string()]),
                        resources: Some(k8s_openapi::api::core::v1::VolumeResourceRequirements {
                            requests: Some({
                                let mut reqs = BTreeMap::new();
                                reqs.insert(
                                    "storage".to_string(),
                                    k8s_openapi::apimachinery::pkg::api::resource::Quantity("10Gi".to_string()),
                                );
                                reqs
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    /// Deploy Horizontal Pod Autoscaler
    #[instrument(skip(self))]
    pub async fn deploy_hpa(
        &self,
        min_replicas: i32,
        max_replicas: i32,
        target_cpu: i32,
        target_memory: i32,
    ) -> Result<(), kube::Error> {
        info!(
            min = min_replicas,
            max = max_replicas,
            target_cpu = target_cpu,
            target_memory = target_memory,
            "Deploying HPA"
        );

        let hpa = self.build_hpa(min_replicas, max_replicas, target_cpu, target_memory);
        let api: Api<HorizontalPodAutoscaler> =
            Api::namespaced(self.client.clone(), &self.namespace);

        let hpa_name = format!("{}-hpa", self.app_name);
        match api.get(&hpa_name).await {
            Ok(_) => {
                let patch = Patch::Apply(&hpa);
                let pp = PatchParams::apply("rethinkdb-controller");
                api.patch(&hpa_name, &pp, &patch).await?;
                info!("Updated existing HPA");
            }
            Err(_) => {
                let pp = PostParams::default();
                api.create(&pp, &hpa).await?;
                info!("Created new HPA");
            }
        }

        Ok(())
    }

    /// Build HPA configuration
    fn build_hpa(
        &self,
        min_replicas: i32,
        max_replicas: i32,
        target_cpu: i32,
        target_memory: i32,
    ) -> HorizontalPodAutoscaler {
        let hpa_name = format!("{}-hpa", self.app_name);
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.app_name.clone());

        HorizontalPodAutoscaler {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some(hpa_name),
                namespace: Some(self.namespace.clone()),
                labels: Some(labels),
                ..Default::default()
            },
            spec: Some(HorizontalPodAutoscalerSpec {
                min_replicas: Some(min_replicas),
                max_replicas,
                scale_target_ref: k8s_openapi::api::autoscaling::v2::CrossVersionObjectReference {
                    api_version: Some("apps/v1".to_string()),
                    kind: "StatefulSet".to_string(),
                    name: self.app_name.clone(),
                },
                metrics: Some(vec![
                    MetricSpec {
                        type_: "Resource".to_string(),
                        resource: Some(k8s_openapi::api::autoscaling::v2::ResourceMetricSource {
                            name: "cpu".to_string(),
                            target: k8s_openapi::api::autoscaling::v2::MetricTarget {
                                type_: "Utilization".to_string(),
                                average_utilization: Some(target_cpu),
                                ..Default::default()
                            },
                        }),
                        ..Default::default()
                    },
                    MetricSpec {
                        type_: "Resource".to_string(),
                        resource: Some(k8s_openapi::api::autoscaling::v2::ResourceMetricSource {
                            name: "memory".to_string(),
                            target: k8s_openapi::api::autoscaling::v2::MetricTarget {
                                type_: "Utilization".to_string(),
                                average_utilization: Some(target_memory),
                                ..Default::default()
                            },
                        }),
                        ..Default::default()
                    },
                ]),
                behavior: Some(k8s_openapi::api::autoscaling::v2::HorizontalPodAutoscalerBehavior {
                    scale_up: Some(k8s_openapi::api::autoscaling::v2::HPAScalingRules {
                        stabilization_window_seconds: Some(60),
                        policies: Some(vec![
                            k8s_openapi::api::autoscaling::v2::HPAScalingPolicy {
                                type_: "Pods".to_string(),
                                value: 2,
                                period_seconds: 60,
                            },
                            k8s_openapi::api::autoscaling::v2::HPAScalingPolicy {
                                type_: "Percent".to_string(),
                                value: 50,
                                period_seconds: 60,
                            },
                        ]),
                        select_policy: Some("Max".to_string()),
                    }),
                    scale_down: Some(k8s_openapi::api::autoscaling::v2::HPAScalingRules {
                        stabilization_window_seconds: Some(300),
                        policies: Some(vec![k8s_openapi::api::autoscaling::v2::HPAScalingPolicy {
                            type_: "Pods".to_string(),
                            value: 1,
                            period_seconds: 300,
                        }]),
                        select_policy: Some("Min".to_string()),
                    }),
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    /// Deploy Pod Disruption Budget
    #[instrument(skip(self))]
    pub async fn deploy_pdb(&self, min_available: i32) -> Result<(), kube::Error> {
        info!(min_available = min_available, "Deploying PDB");

        let pdb = self.build_pdb(min_available);
        let api: Api<PodDisruptionBudget> = Api::namespaced(self.client.clone(), &self.namespace);

        let pdb_name = format!("{}-pdb", self.app_name);
        match api.get(&pdb_name).await {
            Ok(_) => {
                let patch = Patch::Apply(&pdb);
                let pp = PatchParams::apply("rethinkdb-controller");
                api.patch(&pdb_name, &pp, &patch).await?;
                info!("Updated existing PDB");
            }
            Err(_) => {
                let pp = PostParams::default();
                api.create(&pp, &pdb).await?;
                info!("Created new PDB");
            }
        }

        Ok(())
    }

    /// Build PDB configuration
    fn build_pdb(&self, min_available: i32) -> PodDisruptionBudget {
        let pdb_name = format!("{}-pdb", self.app_name);
        let mut labels = BTreeMap::new();
        labels.insert("app".to_string(), self.app_name.clone());

        PodDisruptionBudget {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some(pdb_name),
                namespace: Some(self.namespace.clone()),
                labels: Some(labels.clone()),
                ..Default::default()
            },
            spec: Some(PodDisruptionBudgetSpec {
                min_available: Some(k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(
                    min_available,
                )),
                selector: Some(k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector {
                    match_labels: Some(labels),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    /// Scale StatefulSet
    #[instrument(skip(self))]
    pub async fn scale(&self, replicas: i32) -> Result<(), kube::Error> {
        info!(replicas = replicas, "Scaling StatefulSet");

        let api: Api<StatefulSet> = Api::namespaced(self.client.clone(), &self.namespace);

        // Patch scale subresource
        let scale_patch = serde_json::json!({
            "spec": {
                "replicas": replicas
            }
        });

        let pp = PatchParams::apply("rethinkdb-controller");
        api.patch(
            &self.app_name,
            &pp,
            &Patch::Merge(&scale_patch),
        )
        .await?;

        info!("Scaled StatefulSet to {} replicas", replicas);
        Ok(())
    }

    /// Get current replica count
    pub async fn get_replica_count(&self) -> Result<i32, kube::Error> {
        let api: Api<StatefulSet> = Api::namespaced(self.client.clone(), &self.namespace);
        let sts = api.get(&self.app_name).await?;

        Ok(sts
            .spec
            .and_then(|s| s.replicas)
            .unwrap_or(0))
    }

    /// List all pods
    pub async fn list_pods(&self) -> Result<Vec<Pod>, kube::Error> {
        let api: Api<Pod> = Api::namespaced(self.client.clone(), &self.namespace);
        let lp = ListParams::default().labels(&format!("app={}", self.app_name));
        let pods = api.list(&lp).await?;

        Ok(pods.items)
    }

    /// Check if pod is ready
    pub fn is_pod_ready(pod: &Pod) -> bool {
        pod.status
            .as_ref()
            .and_then(|s| s.conditions.as_ref())
            .map(|conditions| {
                conditions.iter().any(|c| {
                    c.type_ == "Ready" && c.status == "True"
                })
            })
            .unwrap_or(false)
    }

    /// Wait for all pods to be ready
    #[instrument(skip(self))]
    pub async fn wait_for_ready(&self, timeout_seconds: u64) -> Result<(), kube::Error> {
        info!(timeout = timeout_seconds, "Waiting for pods to be ready");

        let start = std::time::Instant::now();
        loop {
            let pods = self.list_pods().await?;
            let ready_count = pods.iter().filter(|p| Self::is_pod_ready(p)).count();

            info!(
                ready = ready_count,
                total = pods.len(),
                "Pod readiness status"
            );

            if ready_count == pods.len() && !pods.is_empty() {
                info!("All pods are ready");
                return Ok(());
            }

            if start.elapsed().as_secs() > timeout_seconds {
                warn!("Timeout waiting for pods to be ready");
                return Err(kube::Error::Service(Box::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Timeout waiting for pods to be ready",
                ))));
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    // Note: Integration tests for K8s client require actual cluster
    // See tests/k8s_scaling_test.rs for full integration tests
}
