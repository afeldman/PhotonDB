# Kubernetes Deployment Quick Reference

## Deploy Commands

```bash
# Basic deployment
./k8s-deploy.sh

# Production deployment
NAMESPACE=rethinkdb-prod \
REPLICAS=5 \
STORAGE_SIZE=500Gi \
CPU_REQUEST=2 \
MEMORY_REQUEST=4Gi \
CPU_LIMIT=8 \
MEMORY_LIMIT=16Gi \
./k8s-deploy.sh

# Development deployment
NAMESPACE=rethinkdb-dev \
REPLICAS=1 \
STORAGE_SIZE=10Gi \
./k8s-deploy.sh
```

## Management Commands

```bash
# Scale cluster
kubectl scale statefulset rethinkdb -n rethinkdb --replicas=5

# Update resources
kubectl set resources statefulset/rethinkdb -n rethinkdb \
  --requests=cpu=1,memory=2Gi \
  --limits=cpu=4,memory=8Gi

# Restart pods (rolling)
kubectl rollout restart statefulset/rethinkdb -n rethinkdb

# View logs
kubectl logs -f -n rethinkdb rethinkdb-0

# Execute command
kubectl exec -it -n rethinkdb rethinkdb-0 -- bash
```

## Monitoring

```bash
# Watch HPA
kubectl get hpa -n rethinkdb -w

# Pod metrics
kubectl top pods -n rethinkdb

# View Prometheus metrics
kubectl port-forward -n rethinkdb svc/rethinkdb-metrics 9090:9090
curl http://localhost:9090/metrics
```

## Troubleshooting

```bash
# Check pod status
kubectl describe pod rethinkdb-0 -n rethinkdb

# Check events
kubectl get events -n rethinkdb --sort-by='.lastTimestamp'

# Check storage
kubectl get pvc -n rethinkdb
kubectl describe pvc data-rethinkdb-0 -n rethinkdb

# Debug from pod
kubectl run -it --rm debug --image=busybox --restart=Never -n rethinkdb -- sh
```
