# PhotonDB Plugin System Overview

PhotonDB includes a modular, extensible **plugin architecture** that enables developers
to extend core database functionality without modifying the main codebase.
The plugin system is designed for safety, isolation, performance, and predictable
integration with the query engine, storage engine, and network layers.

---

# 1. Goals of the Plugin System

The plugin system enables PhotonDB to support:

- Custom data types  
- New storage engines  
- Vector indexes (HNSW, IVF, PQ)  
- Time-series engines  
- Observability and metrics extensions  
- Authentication providers  
- Query-language extensions  
- Cap’n Proto protocol extensions  

Key goals:

- ⚡ Zero-cost abstractions where possible  
- 🔒 Memory-safe sandbox (Rust boundaries)  
- 🧩 Dynamic runtime loading of plugins  
- 🔗 Static build-time plugins for performance-critical modules  
- 🌐 Cluster-aware plugin behavior  

---

# 2. Architecture Overview

```
                 +----------------------------+
                 | PhotonDB Core              |
                 |  - Query Engine            |
                 |  - Storage Engine          |
                 |  - Network Layer           |
                 +--------------+-------------+
                                |
                     Plugin Registry (Global)
                                |
                +---------------+----------------+
                |                                |
     +---------------------+        +----------------------------+
     |  Runtime Plugins    |        |  Built-In Plugins          |
     |  (.so / .dll)       |        |  (compiled into binary)    |
     +---------------------+        +----------------------------+
                |                                |
        Exposed Traits                      Internal Traits
```

Plugins interact with the core via **stable trait interfaces**.

---

# 3. Plugin Lifecycle

Every plugin goes through a structured lifecycle:

### 3.1 Discovery
Plugins are loaded from:

- `/plugins/*.so`
- `/opt/photondb/plugins/`
- or statically compiled into the binary

The loader (`plugin/loader.rs`) scans directories, validates signatures,
and loads shared libraries safely via `libloading`.

### 3.2 Initialization
Plugins must implement:

```rust
pub trait Plugin {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn register(&self, registry: &mut PluginRegistry);
}
```

### 3.3 Registration
Plugins register:

- new query functions  
- new storage backends  
- new vector index types  
- new RPC endpoints  
- admin commands  
- custom background workers  

### 3.4 Runtime Execution
Plugins are invoked by:

- Query Engine  
- Storage Layer  
- Network Protocol  
- Scheduler (Task Engine)  

### 3.5 Unloading (optional)
Plugins may be unloaded if:

- the node runs out of memory  
- a hot reload is triggered  
- cluster management requires rotation  

---

# 4. Plugin Registry (`plugin/registry.rs`)

A global registry tracks:

- plugin metadata  
- loaded modules  
- exposed capabilities  
- hooks (query, storage, cluster, network)

Registry operations:

```rust
registry.register_storage_engine("myengine", Arc::new(MyEngine::new()));
registry.register_vector_index("hnsw", Arc::new(HnswIndex::new()));
registry.register_query_function("approx", approx_fn);
```

---

# 5. Plugin Types

PhotonDB supports several kinds of plugins:

---

## 5.1 Query Function Plugins

Add new operators:

```
r.table("data").map(row => plugin.approx(row.field))
```

### Example definition:

```rust
pub trait QueryFunctionPlugin {
    fn invoke(&self, args: &[Datum]) -> Datum;
}
```

Use cases:

- ML inference  
- Statistical analysis  
- Geospatial functions  
- Crypto functions  

---

## 5.2 Storage Engine Plugins

Allow swapping in new backends:

- columnar engine  
- time-series engine  
- in-memory engine  
- encrypted engine  

Trait:

```rust
pub trait StorageEnginePlugin: Send + Sync {
    fn open(&self, config: &StorageConfig) -> Box<dyn StorageEngine>;
}
```

---

## 5.3 Vector Index Plugins

HNSW / IVF / PQ implementations live as plugins.

Trait:

```rust
pub trait VectorIndexPlugin {
    fn create(&self, dims: usize, opts: &IndexOptions)
        -> Arc<dyn VectorIndex>;
}
```

---

## 5.4 Network Protocol Plugins

Plugins can extend PhotonDB’s protocol stack:

- new Cap’n Proto RPC endpoints  
- new HTTP endpoints  
- authentication providers  

Trait:

```rust
pub trait ProtocolPlugin {
    fn register_endpoints(&self, server: &mut HttpServer);
}
```

---

## 5.5 Cluster Plugins

Extend distributed behavior:

- custom partitioning  
- sharding logic  
- replication strategies  
- load balancing policies  

Trait:

```rust
pub trait ClusterPlugin {
    fn on_node_join(&self, info: &NodeInfo);
    fn on_node_leave(&self, info: &NodeInfo);
}
```

---

# 6. Security Model for Plugins

PhotonDB ensures plugins cannot compromise node integrity:

### 6.1 Plugin Sandbox Rules
- No raw pointer access across FFI boundaries  
- Restricted filesystem access (optional seccomp)  
- Memory safety guaranteed by Rust boundaries  
- Plugins must declare required permissions  
- Verification of digital signatures (future)

### 6.2 Cluster Safety
- Nodes exchange plugin lists  
- Only clusters with identical plugin sets are allowed to merge  
- Plugins cannot escape the sandbox or modify system config  

---

# 7. Plugin Loading Configuration

Example config:

```toml
[plugins]
paths = [
    "/opt/photondb/plugins",
    "./plugins"
]

[plugins.enabled]
hnsw = true
ivf = true
custom_auth = false
```

---

# 8. Error Handling

Plugins can raise:

- initialization errors  
- capability registration conflicts  
- runtime errors  

Standardized error type:

```rust
enum PluginError {
    InitFailed(String),
    CapabilityConflict(String),
    RuntimeError(String),
}
```

---

# 9. Future Extensions

### 9.1 WASM Plugin Runtime
Run plugins in a fully isolated WebAssembly sandbox.

### 9.2 GPU Plugin API
Allow CUDA / Metal / Vulkan compute modules.

### 9.3 Distributed Plugin Federation
Plugins that coordinate across nodes.

### 9.4 Plugin Marketplace
Community-driven extension ecosystem.

---

# 10. Summary

The PhotonDB Plugin System is:

- Modular  
- Safe  
- Extensible  
- Cluster-aware  
- Designed for high-performance workloads  

It enables PhotonDB to adapt to emerging AI, vector search, time-series, and
distributed database requirements without requiring core rewrites.
