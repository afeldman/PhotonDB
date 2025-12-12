# PhotonDB Cluster Design

PhotonDB supports a modular, extensible clustering architecture aimed at
high availability, horizontal scalability, and low-latency distributed
operations. This document provides a complete overview of how clustering
works internally.

---

# 1. Cluster Architecture Overview

PhotonDB uses a **loosely coupled, gossip-based cluster model** with
optional future support for strong consensus mechanisms (Raft/EPaxos).

```
+---------------------+      +---------------------+
|      Node A         |      |      Node B         |
|  - Query Engine     |      |  - Storage Engine   |
|  - Storage Engine   | <--> |  - Cluster Gossip   |
|  - Gossip Protocol  |      |  - Metrics Export   |
+---------------------+      +---------------------+
            ^                         |
            |                         v
                     +---------------------+
                     |      Node C         |
                     |  - Query Router     |
                     |  - Metrics          |
                     +---------------------+
```

Each node:
- Maintains its own local storage
- Participates in a gossip mesh
- Publishes health, load, roles, and topology info
- Can route or rebalance requests as needed

---

# 2. Node Roles

PhotonDB supports multiple roles (future dynamic role assignment):

### 2.1 Primary Roles
- **Data Node**  
  Stores data; handles queries hitting local storage.

- **Coordinator Node**  
  Handles high-level routing, orchestration, and metadata.

- **Observer Node**  
  Read-only node; useful for analytics or testing.

### 2.2 System Roles (future)
- **Consensus Node** (Raft/EPaxos)  
  Manages metadata and global state safely.

- **Vector Node**  
  Hosts vector search indexes (HNSW, IVF).

- **TS Node**  
  Time-series optimized ingestion nodes.

---

# 3. Gossip Protocol (`src/cluster/`)

PhotonDB uses a gossip-based discovery mechanism.

Key features:

- Periodic heartbeat exchange  
- Anti-entropy state synchronization  
- Failure detection  
- Membership updates  
- Lightweight, low-latency communication  

### Gossip messages include:

- Node ID  
- Host/IP  
- Roles  
- Version  
- Storage health  
- Metrics snapshot  

Future: encrypted & signed gossip frames.

---

# 4. Health System (`src/cluster/health.rs`)

Every node reports:

- CPU load  
- Memory usage  
- Storage engine health  
- WAL integrity  
- Replication lag (future)  
- Query throughput  
- Latency metrics  

Two health models:
- **Liveness** – “Is the node alive at all?”
- **Readiness** – “Can it serve queries?”
- **Startup** – “Is it still booting or replaying WAL?”

---

# 5. Metrics System (`src/cluster/metrics.rs`)

PhotonDB integrates with Prometheus and provides:

- Query metrics
- Storage I/O metrics
- Iterator performance
- WAL throughput
- Network latency
- Cluster state metrics

Metrics are exposed at:

```
/_metrics
```

---

# 6. Scaling Logic (`src/cluster/scaling.rs`)

PhotonDB supports auto-scaling via:

- CPU thresholds  
- Memory pressure  
- Query QPS thresholds  
- Storage utilization  
- Latency-based triggers  

Scaling rules can trigger:

- Add new data nodes  
- Promote coordinators  
- Throttle ingestion  
- Rebalance partitions (future)

---

# 7. Partitioning (Future)

PhotonDB will support **range sharding** and **hash sharding**.

### Planned data distribution strategies:

1. **Hash partitioning**
   ```
   shard = hash(key) % N
   ```

2. **Range partitioning**
   Useful for time-series or sequential IDs.

3. **Hybrid model**
   Range-based primary shards + hash-based replicas.

Partition metadata will be stored in:

- a replicated metadata log
- accessed via cluster coordinators

---

# 8. Replication Model (Future)

Replication modes:

### 8.1 Asynchronous replication
- Lightweight  
- Low latency  
- Good for geographically distributed clusters  

### 8.2 Semi-synchronous replication
- Acks from a quorum of replicas  
- Stronger consistency  

### 8.3 Fully synchronous replication (consensus)
- Powered by Raft/EPaxos  
- Enables strict serializability  
- Requires consensus group management

Replication covers:

- WAL records  
- Page deltas  
- B-Tree mutations  
- Metadata (indexes, table definitions)

---

# 9. Consensus Layer (Planned)

PhotonDB’s long-term vision includes a **unified consensus layer**:

- Raft: simple, strong consistency, leader-based  
- EPaxos: leaderless, low-latency multi-region  
- HotStuff: future consideration  

Consensus will govern:

- Metadata  
- Shard placement  
- Configuration changes  
- Security settings  
- Cluster membership  

---

# 10. Cluster Security

See `SECURITY_MODEL.md` for details.  
In short:

- Nodes authenticate via mTLS  
- Gossip frames include signatures  
- Cluster secrets rotate automatically  
- Unauthorized nodes are rejected  
- Strict role enforcement  

---

# 11. Kubernetes Integration (`k8s/`, `helm/`)

PhotonDB provides:

- StatefulSet configuration  
- Headless services for discovery  
- Pod readiness/liveness checks  
- Horizontal auto-scaling hooks  
- Prometheus annotations  

Cluster nodes automatically discover each other inside a Kubernetes network.

---

# 12. Operational Workflows

### 12.1 Adding a Node
1. Node starts with bootstrap token  
2. Performs mTLS handshake  
3. Joins gossip mesh  
4. Fetches cluster metadata  
5. Starts serving queries  

### 12.2 Removing a Node
1. Coordinator marks node as draining  
2. Rebalance (future)  
3. Gossip updates membership  
4. Graceful shutdown  

### 12.3 Cluster Upgrade
- Rolling upgrade with version-aware gossip  
- Safely skip nodes that are out-of-date  
- WAL compatibility checks  

---

# 13. Fault Tolerance

PhotonDB will provide:

- Automatic failover  
- Read replicas  
- WAL replay on crash  
- Metadata recovery  
- Node quarantine for suspicious behavior  

With consensus enabled:

- Survives N/2 failures (quorum-based)

---

# 14. Future Cluster Enhancements

- Distributed vector search  
- Time-series ingestion clusters  
- Geosharded multi-region clusters  
- Automatic rebalancing  
- Consistent hash rings  
- Cross-node query pushdown  
- Distributed transactions  

---

# 15. Summary

PhotonDB’s cluster architecture is:

- Modular  
- Secure  
- Scalable  
- Future-proof  
- Designed for real-time and analytical workloads  

It supports both simple deployments and complex distributed systems
with strong consistency.

