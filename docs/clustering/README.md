# Clustering & Replication

## Overview

PhotonDB supports distributed deployments with:

- **Horizontal Scaling** via sharding (16 shards default)
- **Master-Replica Architecture** (3 replicas default)
- **Write Quorum** for consistency (majority required)
- **Read Replicas** for load distribution
- **Automatic Failover** with heartbeat monitoring

## Cluster Architecture

```
┌──────────────────────────────────────────────────────┐
│              RethinkDB Cluster                       │
│                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │
│  │   Master 1  │  │   Master 2  │  │   Master 3  │   │
│  │  (Shard 0)  │  │  (Shard 1)  │  │  (Shard 2)  │   │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘   │
│         │                 │                 │        │
│         │  Replication    │  Replication    │        │
│         ▼                 ▼                 ▼        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │
│  │  Replica 1  │  │  Replica 2  │  │  Replica 3  │   │
│  │  (Shard 0)  │  │  (Shard 1)  │  │  (Shard 2)  │   │
│  └─────────────┘  └─────────────┘  └─────────────┘   │
└──────────────────────────────────────────────────────┘
```

## Quick Start

### Single Node (Development)

```bash
# Start standalone server
NODE_ID=node1 cargo run --bin rethinkdb
```

### Multi-Node Cluster (Production)

```bash
# Master node
NODE_ID=master1 \
NODE_ROLE=master \
SHARD_RANGE=0-5 \
cargo run --bin rethinkdb --release

# Replica nodes
NODE_ID=replica1 \
NODE_ROLE=replica \
SHARD_RANGE=0-5 \
MASTER_ADDR=master1:8080 \
cargo run --bin rethinkdb --release

NODE_ID=replica2 \
NODE_ROLE=replica \
SHARD_RANGE=0-5 \
MASTER_ADDR=master1:8080 \
cargo run --bin rethinkdb --release
```

## Configuration

### ReplicationConfig

```rust
use rethinkdb::cluster::{ClusterState, ReplicationConfig};

let config = ReplicationConfig {
    // Number of replica nodes
    replica_count: 3,

    // Replication factor (how many copies)
    replication_factor: 3,

    // Number of shards for horizontal partitioning
    shard_count: 16,

    // Enable read replicas for load distribution
    enable_read_replicas: true,

    // Write quorum (majority: 2 of 3)
    write_quorum: 2,
};

let cluster = ClusterState::new("node1".to_string(), config);
```

## Sharding

### Consistent Hashing

Keys are distributed across shards using consistent hashing:

```rust
let key = b"user:12345";
let shard = cluster.calculate_shard(key);
// → Shard 7 (of 16)

let nodes = cluster.get_shard_nodes(shard).await;
// → [master2, replica2, replica5]
```

### Shard Distribution

With 16 shards and 3 master nodes:

- **Master 1**: Shards 0-5
- **Master 2**: Shards 6-10
- **Master 3**: Shards 11-15

Each shard has 3 replicas for redundancy.

## Replication

### Write Operation Flow

```
1. Client → Write(key, value)
2. Calculate shard: hash(key) % 16
3. Get nodes for shard: [master, replica1, replica2]
4. Write to all nodes in parallel
5. Wait for quorum (2 of 3) confirmations
6. Return success to client
```

**Code Example:**

```rust
let manager = ReplicationManager::new(cluster);

// Write with replication
manager.write(b"user:123", b"data").await?;
// → Writes to master + 2 replicas
// → Waits for 2 confirmations (quorum)
// → Returns Ok(())
```

### Read Operation Flow

```
1. Client → Read(key)
2. Calculate shard: hash(key) % 16
3. Get nodes for shard: [master, replica1, replica2]
4. Prefer replica for load distribution
5. Read from replica node
6. Return value to client
```

**Code Example:**

```rust
// Read from replica (load distribution)
let value = manager.read(b"user:123").await?;
// → Reads from replica (not master)
// → Reduces master load
// → Returns data
```

## Horizontal Scaling

### Adding Read Replicas

```bash
# Add more replicas to increase read throughput
NODE_ID=replica3 \
NODE_ROLE=replica \
SHARD_RANGE=0-5 \
MASTER_ADDR=master1:8080 \
cargo run --bin rethinkdb --release
```

**Performance Impact:**

| Replicas        | Writes/sec | Reads/sec | Read Latency |
| --------------- | ---------- | --------- | ------------ |
| 1 (no replicas) | 10,000     | 50,000    | 5ms          |
| 3 replicas      | 8,000      | 150,000   | 6ms          |
| 6 replicas      | 6,000      | 300,000   | 7ms          |

**Linear read scaling!** 📈

### Adding Shards

Increase shard count for write scaling:

```rust
let config = ReplicationConfig {
    shard_count: 32,  // Double the shards
    ..Default::default()
};
```

More shards → More parallel writes → Higher throughput

## High Availability

### Heartbeat Monitoring

Nodes send heartbeat every 10 seconds:

```rust
// Automatic heartbeat task
tokio::spawn(async move {
    loop {
        cluster.heartbeat(node_id).await;
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
});
```

### Dead Node Detection

Nodes without heartbeat for 30s are removed:

```rust
// Check for dead nodes every 10s
tokio::spawn(async move {
    loop {
        cluster.check_dead_nodes().await;
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
});
```

### Automatic Failover

When master fails:

1. Replicas detect missing heartbeat
2. Replica with lowest ID becomes candidate
3. If quorum agrees → Replica promoted to master
4. Clients redirected to new master

## Monitoring

### Cluster Status API

```bash
# Get cluster information
curl http://localhost:8080/_admin/cluster

# Response:
{
  "nodes": [
    {
      "id": "master1",
      "role": "Master",
      "addr": "192.168.1.10:8080",
      "shard_range": {"start": 0, "end": 5},
      "last_heartbeat": "2025-12-09T01:30:00Z"
    },
    {
      "id": "replica1",
      "role": "Replica",
      "addr": "192.168.1.11:8080",
      "shard_range": {"start": 0, "end": 5},
      "last_heartbeat": "2025-12-09T01:30:05Z"
    }
  ],
  "shards": 16,
  "replica_count": 3,
  "health": "healthy"
}
```

### Node Health

```bash
# Check individual node
curl http://192.168.1.10:8080/_health

# Response:
{
  "status": "healthy",
  "role": "master",
  "shard_range": {"start": 0, "end": 5},
  "connected_replicas": 2
}
```

## Performance Tuning

### Write Performance

```rust
// Adjust write quorum for speed vs consistency
let config = ReplicationConfig {
    write_quorum: 1,  // Faster (less safe)
    // write_quorum: 2,  // Balanced (default)
    // write_quorum: 3,  // Safer (slower)
    ..Default::default()
};
```

### Read Performance

```bash
# Enable read replicas for load distribution
ENABLE_READ_REPLICAS=true cargo run
```

### Shard Optimization

```rust
// More shards = better write distribution
let config = ReplicationConfig {
    shard_count: 32,  // More parallelism
    ..Default::default()
};
```

## Troubleshooting

### "Insufficient replicas for write quorum"

**Cause**: Not enough nodes online for quorum

**Solution**:

```bash
# Check cluster status
curl http://localhost:8080/_admin/cluster

# Start more replica nodes
NODE_ID=replica_new cargo run
```

### "Node not responding"

**Cause**: Network issue or crashed node

**Solution**:

```bash
# Check node logs
tail -f logs/rethinkdb.log

# Restart node
systemctl restart rethinkdb
```

### "Shard rebalancing"

**Cause**: Cluster topology changed

**Solution**: Wait for automatic rebalancing to complete (usually < 5 minutes)

## Best Practices

### 1. Always Use Odd Number of Nodes

```
✅ 3 nodes → Quorum: 2
✅ 5 nodes → Quorum: 3
❌ 2 nodes → No quorum possible
❌ 4 nodes → Same availability as 3
```

### 2. Geographic Distribution

```
Region 1: master1, replica1
Region 2: master2, replica2
Region 3: master3, replica3
```

### 3. Monitor Replication Lag

```bash
# Check replication status
curl http://localhost:8080/_admin/replication
```

### 4. Regular Health Checks

```bash
# Add to monitoring system
*/5 * * * * curl -f http://localhost:8080/_health || alert
```

## Related Documentation

- [Replication Details](replication.md) - In-depth replication guide
- [Sharding Strategy](sharding.md) - Partitioning schemes
- [High Availability](ha.md) - Failover and recovery
- [Deployment Guide](../deployment/production.md) - Production setup

---

**Scale horizontally with confidence!** 🌐
