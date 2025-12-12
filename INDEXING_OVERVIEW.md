# PhotonDB Indexing Overview

PhotonDB provides a modular, extensible indexing subsystem designed to support:
- Primary B‑Tree indexes (current)
- Secondary indexes (planned)
- Vector indexes for ANN search (planned)
- Time-series optimized indexes (planned)

This document gives a full developer-level overview of the indexing architecture.

---

# 1. Indexing Architecture

PhotonDB separates indexing into three conceptual layers:

```
          +----------------------------+
          |     Query Engine (AST)     |
          +----------------------------+
                       |
                       v
          +----------------------------+
          |    Index Planner (future)  |
          +----------------------------+
                       |
                       v
          +----------------------------+
          |  Index Subsystem (Traits)  |
          +----------------------------+
            /             |            \
           v              v             v
   +-------------+  +-------------+  +-------------+
   | Primary     |  | Secondary   |  | Vector ANN  |
   | B-Tree      |  | B-Tree      |  | (HNSW/IVF)  |
   +-------------+  +-------------+  +-------------+
```

This design allows PhotonDB to grow from classic document-store indexing
into vector and hybrid workloads without changing the query engine APIs.

---

# 2. Primary Index (Current Implementation)

Every table automatically has a **primary index**, implemented as a durable
B‑Tree stored over slab pages with WAL-backed writes.

### Features:
- Key → record pointer mapping
- Ordered key traversal (range scans)
- Support for:
  - `get`
  - `insert`
  - `delete`
  - prefix and range scans
- Crash‑safe via WAL replay
- Efficient page-level caching

### File locations:
- `src/storage/btree/`
- `src/reql/types.rs` (key encoding)
- `src/reql/datum.rs` (value encoding)

The primary key is always unique.

---

# 3. Secondary Indexes (Future)

Secondary indexes allow queries like:

```
r.table("users").getAll("active", { index: "status" })
```

PhotonDB plans to support:

### 3.1 B-Tree Secondary Indexes
Stored similarly to primary indexes but mapping:

```
secondary_key → [primary_keys]
```

Features:
- Multi-value entries
- Compound indexes
- Index constraints and type validation
- Pushdown of filters into indexed scans

### 3.2 Index Metadata
Stored in cluster metadata (future):

- index name
- index type
- key extractor function
- uniqueness flag
- sort order
- storage backend

Indexes will be defined via ReQL commands or admin API.

---

# 4. Vector Indexes (Planned)

PhotonDB is being designed for **scientific and AI workloads**, so vector search
is a first-class feature roadmap item.

Target: support ANN vector indexes through plugins.

Candidate algorithms:
- **HNSW** – fast, high recall, great for dynamic updates
- **IVF-Flat / IVF-PQ** – scalable clustering-based ANN
- **DiskANN / PQ** – high-compression vector storage
- **Product Quantization (PQ)** for memory efficiency

Architecture:

```
Query Engine
      |
Vector Query Planner (future)
      |
Vector Index Plugin (HNSW/IVF)
      |
Embedding Storage
```

### Planned API (conceptual):

```rust
vector_index.insert(id, embedding);
vector_index.search(query_embedding, k);
vector_index.delete(id);
```

Vector indexes will integrate with:
- storage engine (for persistent embeddings)
- plugin system (`src/plugin/`)
- cluster sharding (for multi-node ANN search)

---

# 5. Time-Series Indexes (Planned)

Time-series indexing requires:

- Timestamp-based partitioning
- High-ingest write path
- Compression & retention policies

Proposed internal structure:

1. **Hot partitions** → in-memory B‑Trees or skip-lists  
2. **Warm partitions** → compressed slab pages  
3. **Cold partitions** → archival storage  

Time-series indexes integrate with:
- sliding window scans
- downsampling operations
- timestamp-aware deletion

---

# 6. Index Planner (Future Component)

PhotonDB will eventually include a standalone index planner module that:

- Analyzes query AST
- Selects optimal index strategy
- Rewrites plan steps (filter → index_scan)
- Chooses between:
  - primary
  - secondary
  - vector
  - time-series indexes
- Pushes projections and limits into index scans

Planner will enable advanced optimizations such as:

- predicate pushdown  
- covering indexes  
- intersection of multiple indexes  
- clause reordering for ANN + metadata filtering  

---

# 7. Index API (Internal Trait Design)

A unified internal trait may look like:

```rust
pub trait Index {
    fn insert(&self, key: &[u8], value: &[u8]);
    fn delete(&self, key: &[u8]);
    fn scan(&self, range: Range<Key>) -> IndexIterator;
    fn index_type(&self) -> IndexType;
}
```

Specialized traits:

- `SecondaryIndex`
- `VectorIndex`
- `TimeSeriesIndex`

These will integrate into the storage engine through adapters.

---

# 8. Encoding and Key Design

PhotonDB uses:

- Bytewise-sortable primary keys
- Comparable value encoding for compound keys
- Optional compression for index entries
- Embedding codecs for vector indexes

Encoding definitions live in:

- `src/reql/types.rs`
- `src/reql/datum.rs`

---

# 9. Cluster Integration for Indexing

In cluster mode:

- Index metadata must be replicated
- Partitions must be redistributed on scaling events
- Vector shards may require k‑NN routing (multi-node ANN)
- Time-series partitions may use temporal locality for placement

Future functionality:

- Index versioning
- Index rebuild scheduling
- Distributed ANN search
- Automatic partition migration

---

# 10. Summary

PhotonDB’s indexing subsystem is built on strong foundations and designed for:

- **Performance** (B‑Tree primary + WAL)
- **Flexibility** (secondary indexes)
- **Modern workloads** (vector indexing)
- **Scalable operations** (time-series indexing + clustering)

The architecture ensures PhotonDB can scale from traditional document workloads
to AI-driven, vector-intensive, and time-series-heavy applications.

