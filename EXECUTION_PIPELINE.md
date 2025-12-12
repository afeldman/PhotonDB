# PhotonDB Execution Pipeline

This document describes the **end‑to‑end execution pipeline** of PhotonDB,
from the moment a client sends a request until the response is delivered.

PhotonDB is designed for modularity, performance, and clear separation of
responsibilities. Understanding this pipeline is essential for contributors
working on networking, query execution, storage, or indexing.

---

# 1. High-Level Overview

```
Client
  |
  v
Protocol Layer (JSON / Cap’n Proto / WebSocket)
  |
  v
Network Layer (TCP/QUIC, Auth, Framing)
  |
  v
ReQL AST Layer (term translation)
  |
  v
Query Compiler (validation, planning)
  |
  v
Query Executor (runtime evaluation)
  |
  v
Storage Engine (B-Tree, Slab, WAL)
  |
  v
Result Assembly
  |
  v
Protocol Serialization
  |
  v
Client
```

---

# 2. Step-by-Step Pipeline

Below is a fully expanded walk-through.

---

# 3. Client → Protocol Layer

PhotonDB supports multiple incoming protocols:

- **ReQL JSON over TCP**
- **HTTP/REST**
- **WebSocket**
- **Cap’n Proto RPC** (future high‑performance protocol)

Each protocol maps incoming requests into a **normalized internal message**:

```
InternalQueryRequest {
    token,
    database,
    table,
    term_ast,
    options,
}
```

---

# 4. Protocol → Network Layer

The network layer (`src/network/`) handles:

- Connection management  
- TLS / QUIC encryption  
- Authentication handshake  
- Message framing  
- Reading/writing request streams  

Network layer responsibilities:

### 4.1 Authentication  
Before any query is processed, the session must authenticate using:

- API key
- JWT token
- mTLS (future)
- Cluster token (internal)

### 4.2 Framing  
JSON messages are length‑prefixed; Cap’n Proto uses its binary framing.

---

# 5. Network → ReQL AST Layer

The ReQL AST layer (`src/reql/`) transforms protocol messages into a typed,
recursive tree:

Examples:

```
r.table("users")
```

→

```
Term::Table("users")
```

```
r.table("users").filter({active: true})
```

→

```
Filter(
  Table("users"),
  Predicate(...)
)
```

The AST ensures:

- Correct structure  
- Well-defined types  
- Compatibility with RethinkDB semantics  

---

# 6. AST → Query Compiler

The compiler (`src/query/compiler.rs`) converts AST → **QueryPlan**:

### Responsibilities:

- Validate AST
- Resolve database & table handles
- Infer types
- Optimize plan steps
- Build linear execution plan

### Example transformation:

AST:

```
Filter(
  Table("users"),
  Predicate(active == true)
)
```

Compiler output:

```
Plan {
  steps: [
    TableScan("users"),
    Filter(active == true),
  ]
}
```

Future enhancements:

- Predicate pushdown
- Index selection
- Vector index routing
- Query rewriting
- Cost-based optimization

---

# 7. QueryPlan → Executor

The executor (`src/query/executor.rs`) performs the actual work.

Execution responsibilities:

- Open iterators from the storage engine
- Allocate evaluation context
- Evaluate expressions
- Materialize or stream results
- Enforce limit/order/offset
- Apply backpressure for streaming channels

### Execution Model

PhotonDB uses a **pull-based iterator** model:

```
next_row() -> Option<Row>
```

Advantages:

- Works naturally with streaming
- Reduces memory use
- Supports early termination
- Integrates cleanly with WebSockets

---

# 8. Executor → Storage Engine

The storage engine (`src/storage/`) exposes:

### API
```
get(key)
set(key, value)
delete(key)
scan(range)
iterator(start, end)
```

### Components involved:
- **Slab allocator** → low-level paging  
- **B-Tree engine** → indexing & ordering  
- **WAL** → durability & crash recovery  

Storage ensures:

- ACID‑like semantics for single operations  
- Crash consistency  
- Fast reads using page+slot caching  
- High-ingest performance  

---

# 9. Storage → Result Assembly

After retrieving data, the executor assembles results into ReQL‑compatible
data structures (`reql/datum.rs`).

Example conversion:

```
internal row → Datum::Object
```

Results can be:

- Single values  
- Arrays / streams  
- Partial objects (for `pluck`)  
- Structural transformations (for `map`, `reduce`)  

---

# 10. Result → Protocol Serialization

PhotonDB serializes results depending on the outgoing protocol:

### JSON
```
{
  "token": 1,
  "type": "success",
  "data": [...]
}
```

### WebSocket
- Streaming frames
- Partial responses
- Heartbeat keepalives

### Cap’n Proto RPC
- Binary-encoded structs
- Optional zero-copy buffers
- Streaming RPC via capabilities

---

# 11. Response → Client

Finally, the protocol layer sends the serialized result back to the client.

The client may:

- Render rows
- Stream batches
- Push updates into UI
- Trigger next query based on cursor state

PhotonDB ensures:

- Order preservation  
- Flow control  
- Error propagation  
- Cancellation handling  

---

# 12. Error Pipeline

Errors can occur at any stage:

### Sources:
- Protocol validation  
- Authentication failure  
- Invalid AST term  
- Compiler rejection  
- Storage I/O errors  
- WAL corruption  
- Predicate evaluation failures  

### Standardized Error Format:
```
{
  "type": "error",
  "code": <int>,
  "message": "...",
  "details": "..."
}
```

Cap’n Proto uses `RpcError` structs.

---

# 13. Future Enhancements

Planned improvements:

### Query Engine
- Vectorized execution  
- Batch operators  
- Cost-based optimization  
- GPU-accelerated execution (future)

### Storage Layer
- Multi-version concurrency control  
- Columnar storage options  
- Distributed execution operators  

### Cluster Layer
- Distributed transactions  
- Multi-node query pushdown  
- Index-aware routing  

### Protocols
- gRPC gateway  
- Native WebAssembly client  

---

# 14. Summary

The PhotonDB execution pipeline is:

- Modular  
- Efficient  
- Extensible  
- Cleanly separated by responsibility  
- Prepared for advanced features (ANN, time-series, clustering)

Understanding this pipeline provides a foundation for contributing to the
query engine, storage systems, network stack, or distributed execution path.
