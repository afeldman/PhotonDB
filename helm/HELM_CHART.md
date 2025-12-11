# RethinkDB Kubernetes Helm Chart

**Version:** 3.0.0  
**Chart Version:** 3.0.0

## Overview

This directory contains the official Helm chart for deploying RethinkDB 3.0 on Kubernetes with full auto-scaling, monitoring, and high-availability support.

## Chart Structure

```
helm/rethinkdb/
├── Chart.yaml                   # Chart metadata
├── values.yaml                  # Default configuration
├── values-dev.yaml             # Development environment
├── values-production.yaml      # Production environment
├── README.md                   # Chart documentation
├── .helmignore                 # Packaging exclusions
├── templates/
│   ├── _helpers.tpl            # Template helpers
│   ├── NOTES.txt               # Post-install instructions
│   ├── statefulset.yaml        # Main StatefulSet
│   ├── service.yaml            # 3 services (headless, client, metrics)
│   ├── serviceaccount.yaml     # ServiceAccount
│   ├── configmap.yaml          # Configuration
│   ├── hpa.yaml                # HorizontalPodAutoscaler
│   ├── pdb.yaml                # PodDisruptionBudget
│   ├── rbac.yaml               # RBAC resources
│   ├── servicemonitor.yaml     # Prometheus Operator
│   ├── networkpolicy.yaml      # Network policies
│   └── tests/
│       └── test-connection.yaml # Helm test
```

## Quick Start

### Install with default values

```bash
helm install my-rethinkdb ./helm/rethinkdb --namespace rethinkdb --create-namespace
```

### Install for development

```bash
helm install rethinkdb-dev ./helm/rethinkdb -f ./helm/rethinkdb/values-dev.yaml --namespace dev
```

### Install for production

```bash
helm install rethinkdb-prod ./helm/rethinkdb -f ./helm/rethinkdb/values-production.yaml --namespace production
```

## Features

### Auto-scaling

- **HPA (Horizontal Pod Autoscaling)**: Scale based on CPU, Memory, and custom metrics
- **Custom Metrics**: QPS, connections, replication lag
- **Behavior Policies**: Configurable scale-up/down stabilization

### High Availability

- **StatefulSet**: Stable network identities and persistent storage
- **PodDisruptionBudget**: Ensure minimum availability during updates
- **Anti-affinity**: Spread pods across nodes/zones

### Monitoring

- **Prometheus Metrics**: 20+ metrics exported on port 9090
- **ServiceMonitor**: Prometheus Operator integration
- **Health Checks**: Liveness, readiness, startup probes

### Storage

- **Persistent Volumes**: 100Gi default, configurable storage class
- **Volume Claim Templates**: Automatic PVC creation per pod

### Security

- **RBAC**: ClusterRole and Role with minimal permissions
- **NetworkPolicy**: Ingress/egress rules for cluster isolation
- **Security Contexts**: Run as non-root with fsGroup

## Configuration Examples

### Development (values-dev.yaml)

- 1 replica
- 10Gi storage
- Minimal resources (250m CPU, 512Mi memory)
- No auto-scaling
- ClusterIP service

### Production (values-production.yaml)

- 5 replicas (scales 5-20)
- 500Gi fast-ssd storage
- High resources (2-8 CPU, 4-16Gi memory)
- Aggressive auto-scaling
- LoadBalancer service
- Full monitoring with ServiceMonitor
- PDB with minAvailable: 3

## Testing

Run Helm tests to verify connectivity:

```bash
helm test my-rethinkdb --namespace rethinkdb
```

## Upgrading

```bash
# Upgrade to new version
helm upgrade my-rethinkdb ./helm/rethinkdb --namespace rethinkdb

# Upgrade with new values
helm upgrade my-rethinkdb ./helm/rethinkdb -f new-values.yaml --namespace rethinkdb

# Rollback
helm rollback my-rethinkdb --namespace rethinkdb
```

## Uninstalling

```bash
helm uninstall my-rethinkdb --namespace rethinkdb
```

## Values Documentation

See [README.md](./README.md) for complete values documentation.

## Requirements

- **Kubernetes**: 1.24+
- **Helm**: 3.0+
- **PV Provisioner**: For persistent storage
- **Metrics Server**: For HPA (if enabled)
- **Prometheus Operator** (optional): For ServiceMonitor

## Packaging

```bash
# Lint chart
helm lint ./helm/rethinkdb

# Package chart
helm package ./helm/rethinkdb

# Install from package
helm install my-rethinkdb rethinkdb-3.0.0.tgz
```

## License

Apache-2.0

## Maintainer

Anton Feldmann <anton.feldmann@gmail.com>
