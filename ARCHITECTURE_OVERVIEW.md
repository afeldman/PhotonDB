# PhotonDB Architecture Overview

PhotonDB is a modular, Rust-native database engine designed for real‑time,
scientific, vector, and time‑series workloads.  
Its architecture is inspired by RethinkDB but re‑engineered for modern Rust,
async networking, pluggable storage engines, and distributed cluster operation.

This document provides a high‑level overview of the entire system.

---

# 1. Architectural Principles

PhotonDB is built on the following principles:

- **Separation of Concerns**  
  Clear, isolated subsystems (ReQL, query, storage, server, network, cluster).

- **Modularity & Replaceability**  
  Storage engines, plugins, indexes, and protocols are swappable.

- **Strong Typing & Safety**  
  Rust ensures memory safety and predictable concurrency.

- **Performance**  
  Cache-aware slab allocator, B‑Trees, async networking, WAL durability.

- **Observability & Deployability**  
  Metrics, tracing, health endpoints, Helm charts, Kubernetes support.

---

# 2. High‑Level Architecture

```
           +------------------------+
           |      Client Apps       |
           |  (HTTP, WebSocket,     |
           |   ReQL, RPC)           |
           +-----------+------------+
                       |
                       v
             +------------------+
             |   Server Layer   |
             | (HTTP/WS/Axum)   |
             +---------+--------+
                       |
                       v
       +-------------------------------+
       |       Network Layer           |
       | (TCP/QUIC, Framing, Auth,     |
       |   ReQL protocol adapter)      |
       +---------------+---------------+
                       |
                       v
        +--------------------------------+
        |       Query Engine             |
        | (Compiler, Planner, Executor)  |
        +---------------+----------------+
                        |
                        v
        +--------------------------------+
        |         Storage Engine         |
        | (Slab allocator, B‑Tree, WAL)  |
        +--------------------------------+
                        |
                        v
           +--------------------------+
           |        Persistence       |
           | (On-disk pages, slots,   |
           |  logs, metadata)         |
           +--------------------------+

```

---

# 3. Major Subsystems

PhotonDB is organized into several key subsystems under `src/`.

---

## 3.1 Server Layer (`src/server/`)

The **server layer** is built on *Axum* and provides:

- HTTP REST API
- WebSocket changefeed channels
- Routing, middleware, compression
- Admin UI (`static/admin.html`)
- Health & readiness checks
- Metrics endpoint (`/_metrics`)

It is the user‑facing interface for dashboards, tools, and debugging.

---

## 3.2 Network Layer (`src/network/`)

The **network layer** handles:

- TCP and QUIC connections
- Authentication (`auth.rs`)
- ReQL message framing / decoding
- Cap’n Proto handshake (future)
- Connection lifecycle management
- Backpressure and message buffering

The network layer converts *wire representation → internal queries*.

---

## 3.3 ReQL Layer (`src/reql/`)

PhotonDB supports a subset of the RethinkDB ReQL language.

Components:

- **AST (`ast.rs`)** – Expression tree
- **Terms (`terms.rs`)** – Query verbs and operators
- **Datum Types (`datum.rs`, `types.rs`)** – JSON‑like representation
- **Protocol (`protocol.rs`)** – ReQL framing for the network layer

The goal is to remain compatible with existing ReQL client libraries.

---

## 3.4 Query Engine (`src/query/`)

The **query engine** is responsible for:

- Validating & compiling ReQL AST
- Planning access against indexes
- Executing operations via storage engine
- Streaming results
- Handling errors and type conversions

Key files:

- `compiler.rs`
- `executor.rs`

Queries are executed against a transaction or consistent snapshot.

---

## 3.5 Storage Engine (`src/storage/`)

PhotonDB’s storage layer is highly modular and consists of:

### 3.5.1 Storage Abstraction (`engine.rs`)
Defines traits for reading/writing tables, indexes, and transactions.

### 3.5.2 Slab Allocator (`slab/`)
A contiguous allocator with:

- Size classes
- Free lists
- Page metadata
- Optional compression
- Caching and write barriers

This is used as the low‑level persistence primitive.

### 3.5.3 B‑Tree Storage (`btree/`)
A durable B‑Tree built on top of pages provided by the slab allocator.

Contains:

- Node format
- Page layout
- Pager/cache subsystem
- WAL integration

### 3.5.4 WAL (`btree/wal.rs`)
Ensures crash safety and atomicity.

---

# 4. Plugin System (`src/plugin/`)

PhotonDB supports runtime‑loaded plugins for:

- Vector search (HNSW, IVF…)
- Time‑series engines
- Custom aggregations
- ML inference
- Indexing extensions

Subsystem parts:

- `traits.rs` – Plugin interfaces
- `registry.rs` – Central registry
- `loader.rs` – Dynamic loading

Plugins connect into query execution or storage layers.

---

# 5. Clustering Architecture (`src/cluster/`)

PhotonDB supports distributed clustering with:

- Node discovery
- Health checks
- Load & state metrics
- Horizontal scaling logic
- Kubernetes integration (`k8s/` directory)

Future additions:

- Replication
- Consensus (Raft)
- Distributed transactions

---

# 6. Persistence Layout

The persistence layer is built on:

- Slab pages (fixed‑size)
- Page metadata
- WAL segments
- B‑Tree nodes
- Segment compaction

Diagrams are available in:

```
docs/architecture/storage_layout.png
docs/architecture/database_hierarchy_simple.png
```

---

# 7. Protocols

PhotonDB supports:

### ✔ ReQL JSON Protocol (current)
Partial compatibility with RethinkDB message format.

### ✔ HTTP / WebSocket
Used for metrics, admin UI, monitoring, and realtime subscriptions.

### ⏳ Cap’n Proto RPC (planned)
A fast, binary protocol for client SDKs & internal cluster communication.

---

# 8. Observability

PhotonDB includes:

- Prometheus metrics endpoint
- Structured logging (`tracing`)
- Tracing spans for query & storage
- Health & readiness checks
- Benchmark tooling (`benches/`)

---

# 9. Deployment Architecture

PhotonDB includes out‑of‑the‑box deployment assets:

- **Helm chart** (`helm/rethinkdb`)  
- **Kubernetes manifests** (`k8s/`)  
- **Packaging scripts** (`packaging/`)  
  - `.deb`, `.rpm`, `.dmg`, `.msi`

Supports:

- Horizontal scaling
- Stateful storage
- Automatic configuration through environment variables

---

# 10. Future Architectural Directions

- Distributed consensus (Raft/EPaxos)
- Columnar storage engine
- Tiered storage (SSD + memory)
- Native vector index across cluster
- Machine‑learning driven query optimization
- Cap’n Proto SDKs (Rust, Python, Node)

---

# 11. Summary

PhotonDB is a modern, modular database engine with:

- A well‑layered architecture
- A powerful storage subsystem
- A clean query layer
- Excellent deployment support
- Strong extensibility foundations

This document is the entry point for developers and contributors to understand
PhotonDB’s internal architecture.
