# PhotonDB Storage Engine Overview

The PhotonDB storage engine is designed for high performance, durability,
modularity, and scientific/vector workloads.  
It consists of three primary layers:

1. **Slab Allocator** – low-level paging, allocation, compression, caching  
2. **B-Tree Engine** – indexing, lookups, iteration  
3. **WAL (Write-Ahead Log)** – durability and crash consistency  

This document provides a full technical overview for developers.

---

# 1. Storage Architecture Overview

```
        +-----------------------------+
        |       Query Executor        |
        +-----------------------------+
                       |
                       v
        +-----------------------------+
        |     Storage Engine API      |
        |     (engine.rs trait)       |
        +-----------------------------+
            /                             v                    v
   +--------------+     +----------------+
   |  Slab Layer  |     |  B-Tree Index  |
   +--------------+     +----------------+
         |                       |
         +--------+   +----------+
                  v   v
            +-------------------+
            |  Persistence I/O  |
            |  (pages, WAL)     |
            +-------------------+
```

PhotonDB decouples *logical storage* (tables, indexes) from *physical storage*
(slabs, pages, WAL). This enables flexibility and future engine replacement.

---

# 2. Storage Engine API (`engine.rs`)

The storage engine defines a core trait:

- `get`
- `set`
- `delete`
- `scan`
- `iterator`
- `begin_transaction`
- `commit` / `rollback`

This provides an abstract interface for upper layers (query engine, cluster)
without binding them to a specific implementation.

---

# 3. Slab Allocator (`src/storage/slab/`)

The **slab allocator** is a low-level persistence module responsible for:

- Page allocation
- Size classes
- Page metadata & versioning
- Optional compression
- Hot-page caching
- Free-list management

Key files:

- `allocator.rs`
- `cache.rs`
- `metadata.rs`
- `slot.rs`
- `size_class.rs`
- `storage.rs`
- `bench.rs` / `production_tests.rs`

### 3.1 Page Model

Slabs consist of fixed-size pages:

```
+------------------------------------------------+
| Page Header | Metadata | Payload (Slots/Data) |
+------------------------------------------------+
```

Metadata includes:

- page ID  
- checksum  
- version  
- slot offsets  
- size class info  

Pages may contain:

- B-Tree nodes  
- raw document blobs  
- index metadata  

### 3.2 Slot Allocation

Pages store multiple "slots" of variable-sized encoded data:

- Small objects → packed tightly  
- Larger objects → grouped into larger size classes  

Slots are tracked via metadata to enable:

- Fast lookups  
- Partial rewrites  
- Compaction  

### 3.3 Caching

The slab cache uses:

- LRU or ARC strategies  
- Pointer-stable references  
- Preallocated pools  

This reduces disk I/O and eliminates “hot page thrashing.”

---

# 4. B-Tree Engine (`src/storage/btree/`)

PhotonDB uses a durable, WAL-backed B-Tree for indexing.

Files include:

- `btree.rs` – high-level interface
- `node.rs` – node encoding/decoding
- `page.rs` – relationship to slab pages
- `pager.rs` – buffered read/write manager
- `wal.rs` – write-ahead logging & durability
- `types.rs` – keys, values, pointers

### 4.1 Node Structure

Each B-Tree node typically contains:

```
+----------------------------------------------------+
| Node Header | Keys | Values | Child Pointers       |
+----------------------------------------------------+
```

Nodes are stored in slab pages with offsets recorded.

### 4.2 Pager

The pager:

- Loads nodes from slab pages  
- Keeps an internal node cache  
- Issues write operations via WAL  
- Flushes commits safely  

The pager is the central hub for storage I/O.

### 4.3 B-Tree Operations

Supports:

- `search(key)`
- `insert(key, value)`
- `delete(key)`
- `scan(range)`
- `iterate(prefix)`

All write operations validate page integrity and WAL sequence numbers.

---

# 5. WAL (Write-Ahead Log)

The WAL ensures:

- Atomicity
- Durability
- Crash recovery
- Partial-write detection

Features:

- Checksummed segments
- Log sequence numbers (LSN)
- Replay on startup
- Redo-only model for simplicity
- Future: encryption & compression support

WAL replay reconstructs:

- B-Tree mutations  
- Page writes  
- Metadata updates  

---

# 6. Storage Engine Lifecycle

### 6.1 Startup

1. Initialize slab allocator  
2. Load metadata pages  
3. Open WAL and replay logs  
4. Boot B-Tree engine  
5. Validate cluster storage state (if applicable)

### 6.2 Runtime

- All writes append to WAL  
- Dirty pages flushed lazily  
- Cache evictions based on memory pressure  
- Background compaction (future)

### 6.3 Shutdown

- Flush dirty pages  
- Write closing WAL marker  
- Sync file handles  

---

# 7. Data Model & Encoding

PhotonDB uses:

- JSON-like Datum structures (flexible)  
- B-Tree–friendly key encoding  
- Efficient binary encoding for values  
- Optional compression in slab slots  

Encoding modules live under:

- `src/reql/datum.rs`
- `src/reql/types.rs`

---

# 8. Advanced Plans

### 8.1 Columnar extensions  
Add hybrid row/column model for analytical queries.

### 8.2 Vector storage plugin  
For ANN (HNSW/IVF) indexing.

### 8.3 Time-series storage  
With retention, compaction, & high-ingest pathways.

### 8.4 Multi-version concurrency control (MVCC)  
Isolation for concurrent read/write workloads.

### 8.5 Replicated storage  
With Raft or EPaxos for distributed consensus.

---

# 9. Summary

The PhotonDB storage engine is:

- Modular  
- Durable  
- Performance-optimized  
- Designed for scientific & real-time workloads  
- Ready for future vector and time-series extensions  

This overview equips developers to navigate, extend, and debug the storage
subsystem effectively.
