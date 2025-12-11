# RethinkDB Helm Chart

This chart bootstraps a [RethinkDB](https://rethinkdb.com) deployment on a [Kubernetes](https://kubernetes.io) cluster using the [Helm](https://helm.sh) package manager.

## Prerequisites

- Kubernetes 1.24+
- Helm 3.0+
- PV provisioner support in the underlying infrastructure
- Metrics Server (for HPA)

## Installing the Chart

### Add Helm Repository (if published)

```bash
helm repo add rethinkdb https://your-charts-repo.com
helm repo update
```

### Install from local chart

```bash
# Install with default values
helm install my-rethinkdb ./helm/rethinkdb

# Install in specific namespace
helm install my-rethinkdb ./helm/rethinkdb --namespace rethinkdb --create-namespace

# Install with custom values
helm install my-rethinkdb ./helm/rethinkdb -f my-values.yaml
```

## Uninstalling the Chart

```bash
helm uninstall my-rethinkdb --namespace rethinkdb
```

## Configuration

The following table lists the configurable parameters of the RethinkDB chart and their default values.

### Global Parameters

| Parameter                 | Description                         | Default |
| ------------------------- | ----------------------------------- | ------- |
| `global.imageRegistry`    | Global Docker image registry        | `""`    |
| `global.imagePullSecrets` | Global Docker registry secret names | `[]`    |
| `global.storageClass`     | Global StorageClass                 | `""`    |

### Common Parameters

| Parameter          | Description                                     | Default         |
| ------------------ | ----------------------------------------------- | --------------- |
| `nameOverride`     | String to partially override rethinkdb.fullname | `""`            |
| `fullnameOverride` | String to fully override rethinkdb.fullname     | `""`            |
| `clusterDomain`    | Kubernetes cluster domain                       | `cluster.local` |

### Image Parameters

| Parameter          | Description                | Default               |
| ------------------ | -------------------------- | --------------------- |
| `image.registry`   | RethinkDB image registry   | `docker.io`           |
| `image.repository` | RethinkDB image repository | `rethinkdb/rethinkdb` |
| `image.tag`        | RethinkDB image tag        | `3.0.0`               |
| `image.pullPolicy` | Image pull policy          | `IfNotPresent`        |

### Cluster Parameters

| Parameter           | Description                  | Default      |
| ------------------- | ---------------------------- | ------------ |
| `replicaCount`      | Number of RethinkDB replicas | `3`          |
| `clusterMode`       | Cluster mode                 | `kubernetes` |
| `shardCount`        | Number of shards             | `16`         |
| `replicationFactor` | Replication factor           | `3`          |

### Resource Parameters

| Parameter                   | Description    | Default |
| --------------------------- | -------------- | ------- |
| `resources.requests.cpu`    | CPU request    | `500m`  |
| `resources.requests.memory` | Memory request | `1Gi`   |
| `resources.limits.cpu`      | CPU limit      | `2`     |
| `resources.limits.memory`   | Memory limit   | `4Gi`   |

### Persistence Parameters

| Parameter                  | Description        | Default    |
| -------------------------- | ------------------ | ---------- |
| `persistence.enabled`      | Enable persistence | `true`     |
| `persistence.storageClass` | Storage class      | `fast-ssd` |
| `persistence.size`         | PVC size           | `100Gi`    |

### Auto-scaling Parameters

| Parameter                                       | Description      | Default |
| ----------------------------------------------- | ---------------- | ------- |
| `autoscaling.enabled`                           | Enable HPA       | `true`  |
| `autoscaling.minReplicas`                       | Minimum replicas | `3`     |
| `autoscaling.maxReplicas`                       | Maximum replicas | `10`    |
| `autoscaling.targetCPUUtilizationPercentage`    | Target CPU %     | `70`    |
| `autoscaling.targetMemoryUtilizationPercentage` | Target Memory %  | `80`    |

### Service Parameters

| Parameter             | Description             | Default        |
| --------------------- | ----------------------- | -------------- |
| `service.type`        | Kubernetes Service type | `LoadBalancer` |
| `service.clientPort`  | Client port             | `28015`        |
| `service.clusterPort` | Cluster port            | `29015`        |
| `service.httpPort`    | HTTP admin port         | `8080`         |
| `service.metricsPort` | Metrics port            | `9090`         |

## Examples

### Development Environment

```yaml
# values-dev.yaml
replicaCount: 1
persistence:
  size: 10Gi
resources:
  requests:
    cpu: 250m
    memory: 512Mi
  limits:
    cpu: 1
    memory: 2Gi
autoscaling:
  enabled: false
service:
  type: ClusterIP
```

```bash
helm install rethinkdb-dev ./helm/rethinkdb -f values-dev.yaml
```

### Production Environment

```yaml
# values-prod.yaml
replicaCount: 5
persistence:
  storageClass: fast-ssd
  size: 500Gi
resources:
  requests:
    cpu: 2
    memory: 4Gi
  limits:
    cpu: 8
    memory: 16Gi
autoscaling:
  enabled: true
  minReplicas: 5
  maxReplicas: 20
service:
  type: LoadBalancer
  loadBalancerIP: "1.2.3.4"
metrics:
  serviceMonitor:
    enabled: true
podDisruptionBudget:
  minAvailable: 3
```

```bash
helm install rethinkdb-prod ./helm/rethinkdb -f values-prod.yaml --namespace rethinkdb-prod --create-namespace
```

### With Custom Metrics

```yaml
# values-metrics.yaml
autoscaling:
  enabled: true
  customMetrics:
    - name: rethinkdb_queries_per_second
      targetValue: 2000
    - name: rethinkdb_active_connections
      targetValue: 1000
metrics:
  enabled: true
  serviceMonitor:
    enabled: true
    namespace: monitoring
```

## Upgrading

### Upgrade to new version

```bash
helm upgrade my-rethinkdb ./helm/rethinkdb --namespace rethinkdb
```

### Upgrade with new values

```bash
helm upgrade my-rethinkdb ./helm/rethinkdb -f new-values.yaml --namespace rethinkdb
```

### Rollback

```bash
helm rollback my-rethinkdb --namespace rethinkdb
```

## Testing

Run Helm tests:

```bash
helm test my-rethinkdb --namespace rethinkdb
```

## Monitoring

### Prometheus Integration

If you have Prometheus Operator installed:

```yaml
metrics:
  enabled: true
  serviceMonitor:
    enabled: true
    interval: 30s
    namespace: monitoring
```

### Grafana Dashboard

Import the Grafana dashboard from `k8s/grafana-dashboard.json` after installation.

## Troubleshooting

### Check pod status

```bash
kubectl get pods -n rethinkdb -l app.kubernetes.io/name=rethinkdb
```

### View logs

```bash
kubectl logs -n rethinkdb -l app.kubernetes.io/name=rethinkdb -f
```

### Check HPA status

```bash
kubectl get hpa -n rethinkdb
kubectl describe hpa rethinkdb -n rethinkdb
```

### Access Web UI

```bash
kubectl port-forward -n rethinkdb svc/rethinkdb-client 8080:8080
```

Open http://localhost:8080

## License

Apache-2.0
