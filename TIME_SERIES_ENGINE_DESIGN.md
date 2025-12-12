# PhotonDB Time-Series Engine Design

PhotonDB is designed to support high-ingest, low-latency, and scalable
time-series workloads.  
This document defines the architecture, data model, indexing strategy,
retention logic, and cluster behavior of the PhotonDB time-series engine.

---

# 1. Goals of the Time-Series Engine

PhotonDB aims to provide:

- **High write throughput** (millions of points/sec per node)
- **Low-latency reads** for recent data
- **Native downsampling & rollups**
- **Efficient compression**
- **Partitioned storage model**
- **Cluster-aware distribution**
- **Strong integration with ReQL-like query language**

Use cases include:

- IoT telemetry  
- Sensor networks  
- AI streaming pipelines (audio, motion, biometrics)  
- Observability data  
- Financial tick data  
- Energy grid telemetry  

---

# 2. Architectural Overview

```
                     Query Engine
                          |
                          v
               Time-Series Query Planner
                          |
                          v
          +--------------------------------------+
          |        Time-Series Engine            |
          |--------------------------------------|
          |  Hot Partitions (Memory / WAL)       |
          |  Warm Partitions (Compressed Slabs)  |
          |  Cold Storage (Archive / Object)     |
          +--------------------------------------+
                          |
                          v
                   Storage Engine
```

Time-series data is stored in **partitions**, organized by time windows.

---

# 3. Data Model

Each time-series record has the structure:

```
{
  ts: Timestamp,
  value: Float or Object,
  tags: { key: value, ... },
  metadata: Datum
}
```

### Internal structure:

```
struct TsPoint {
    timestamp: i64,         // nanoseconds or microseconds
    payload: Vec<u8>,       // encoded float|object
    tags: Vec<(String, String)>,
}
```

Tags are indexed separately for fast filtering.

---

# 4. Partitioning Strategy

PhotonDB divides time-series data into **time partitions**:

```
/ts/{table}/{year}/{month}/{day}/partition-{id}
```

Recommended window sizes:

- 1h for high-ingest workloads  
- 24h for moderate workloads  
- Customizable per table  

Each partition transitions through three stages:

### 4.1 Hot Partition
- Stored in-memory (skiplist or small B-Tree)
- Writes appended to WAL
- Ideal for fast inserts
- Limited retention (e.g., last few hours)

### 4.2 Warm Partition
- Serialized into slab pages
- Compressed using:
  - Gorilla compression (timestamp deltas)
  - RLE
  - XOR-based float compression
- Indexed with lightweight B-Tree for queries

### 4.3 Cold Partition
- Fully immutable
- Optimized for scans
- Stored in compressed blocks
- May be offloaded to object storage (S3, MinIO, File)

---

# 5. Ingestion Pipeline

```
Incoming TS Data
      |
      v
Hot Partition (memory)
      |
 WAL Append
      |
Periodic Flush
      |
Warm Partition (compressed slabs)
      |
Retention rules → Cold Partition or Drop
```

PhotonDB ingestion is **append-only**, ensuring:

- High throughput  
- Simple WAL replay  
- Fast recovery  

---

# 6. Query Engine Extensions

### 6.1 Range Query

```
r.table("metrics").between(ts_start, ts_end)
```

### 6.2 Aggregations

```
r.table("metrics").between(0, now).avg("value")
```

Supported aggregations:

- count  
- min / max  
- sum  
- avg  
- percentile (p50, p90, p95, p99)  
- rate  
- derivative  
- integral  

### 6.3 Downsampling

```
r.table("metrics")
 .downsample("1m", { agg: "avg", field: "value" })
```

### 6.4 Tag Filtering

```
r.table("metrics")
 .filter({ host: "node-1", region: "eu" })
```

Tags are stored in a secondary B-Tree index.

---

# 7. Compression Strategy

PhotonDB uses **adaptive compression**:

### 7.1 Timestamp compression (Gorilla)
- delta-of-delta encoding  
- RLE for repeated intervals  

### 7.2 Value compression
Float compression:
- XOR-based float compression (Facebook’s Gorilla)  
- Snappy/LZ4 fallback  

Object compression:
- Brotli for nested JSON-like data  

### 7.3 Tag compression
Dictionary encoding for repeated tag values.

---

# 8. Time-Series Indexing

Each partition maintains:

- **Time index** (start_ts, end_ts)
- **Tag index** (secondary B-Tree)
- **Block index** (for compressed cold blocks)

Index files stored as:

```
partition.meta
partition.index
partition.data
```

---

# 9. Storage Layout

Example directory structure:

```
ts_sensor_data/
   hot/
      partition-2025-12-12T00
   warm/
      partition-2025-12-11
   cold/
      2025/
         12/
           partition-2025-12-01
           partition-2025-12-02
```

---

# 10. Retention & Rollups

PhotonDB supports:

### 10.1 Retention Policies

```
r.table("metrics").setRetention("7d")
```

Time-based deletion runs asynchronously.

### 10.2 Rollup Policies

Automatic periodic downsampling:

```
rollup: [
  { interval: "1m", agg: "avg" },
  { interval: "1h", agg: "avg" }
]
```

Roll-ups are stored in separate time-series tables.

---

# 11. Cluster Behavior

PhotonDB distributes time-series workloads via **temporal sharding**:

### 11.1 Sharding Model

```
shard = floor(timestamp / partition_size) % num_nodes
```

Benefits:

- Even distribution of ingest load  
- Locality for time-range scans  
- Predictable storage growth  

### 11.2 Replication

Replication levels:

- Async replication for hot data  
- Sync replication for warm/cold  
- Metadata replication via Raft (future)

### 11.3 Distributed Range Queries

Coordinator Node:

1. Identify partitions based on time range  
2. Route requests to responsible nodes  
3. Merge results  
4. Apply aggregations  

---

# 12. WAL & Recovery

All hot writes are appended to WAL:

```
timestamp, payload, tags, partition-id
```

During recovery:

1. Recreate hot partitions  
2. Rebuild recent warm partitions if needed  

Cold partitions are immutable and require no reconstruction.

---

# 13. Background Jobs

PhotonDB time-series engine uses a dedicated task system for:

- Partition flushing  
- Retention cleanup  
- Rollup creation  
- Tag index rebuilding  
- Compression optimization  

---

# 14. Future Enhancements

### AI-driven compression optimization  
Automatically choose compression strategy based on data patterns.

### Learned indexes  
ML-based models for timestamp prediction & indexing.

### Hybrid vector + time-series search  
Combine temporal context + embedding similarity.

### GPU-accelerated aggregation  
CUDA kernels for high-speed scans and stats.

---

# 15. Summary

PhotonDB’s time-series engine is:

- High-ingest  
- Low-latency  
- Durable  
- Efficiently compressed  
- Cluster-native  
- Extensible for AI workloads  

It's built to handle everything from IoT sensors to large-scale analytics,
integrating seamlessly with PhotonDB’s query engine and storage engine.

