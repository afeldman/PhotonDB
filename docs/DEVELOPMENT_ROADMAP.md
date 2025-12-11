# PhotonDB 1.0 Development Roadmap

## Vision: v0.1 → v1.0

**PhotonDB** is a modern Rust reimplementation of RethinkDB, starting from Q4 2025 toward production-ready release (v1.0) in Q2 2026.

This roadmap outlines the 10-phase development plan to modernize RethinkDB with:

- Ground-up Rust implementation for performance
- Vector search & AI/ML capabilities
- Time-series optimizations
- Cloud-native architecture
- Preserved RethinkDB query philosophy and wisdom

---

## Release Phases (Q4 2025 → Q2 2026)

### **Phase 1: v0.1 - Core Engine (Current - Q4 2025)**

**Status:** ✅ In Progress  
**Timeline:** Q4 2024 - Q1 2025 → **Q4 2025** (now)
**Focus:** Foundation

**What ships:**

- ✅ Rust web server (Axum 0.7)
- ✅ Custom slab + B-Tree storage engine
- ✅ HTTP/REST API (7 core endpoints)
- ✅ Database hierarchy (Databases → Tables → Documents)
- ✅ ReQL-style query language (basic AST, compiler, executor)
- ✅ WebSocket support for changefeeds
- ✅ Basic CLI with clap
- ✅ Security layer (OAuth2/JWT-ready, Honeytrap integration)

**Key Features:**

- Basic CRUD operations via REST API
- JSON wire protocol (RethinkDB-compatible subset)
- WAL and crash recovery
- Prometheus metrics and health endpoints
- Admin dashboard (static/admin.html)

**Not included:**

- ❌ GraphQL
- ❌ Vector search indexing
- ❌ Time-series features
- ❌ Cap'n Proto RPC
- ❌ Production clustering

---

### **Phase 2: v0.2 - Storage & Optimization**

**Status:** 📋 Planned  
**Timeline:** Q1 2025  
**Focus:** Performance, Persistence, Stability

**What ships:**

- ✅ Enhanced B-Tree implementation with better performance
- ✅ Improved WAL with batch writes
- ✅ Full query compiler optimization
- ✅ Indexing support (B-Tree indices on tables)
- ✅ Transactions (ACID compliance for single-node)
- ✅ Memory-mapped I/O for faster disk access
- ✅ Compression support in slab allocator

**Key Features:**

- Better query execution plans
- Index creation and management
- Transaction isolation levels
- Cache optimization
- Benchmarking suite

**Target Improvements:**

- 5-10x throughput improvement over v0.1
- Sub-millisecond latency for simple queries
- 90% reduction in memory overhead

---

### **Phase 3: v0.3 - Clustering & Distribution**

**Status:** 📋 Planned  
**Timeline:** Q2 2025  
**Focus:** Multi-node deployment, HA, replication

**What ships:**

- ✅ Cluster discovery (Kubernetes + static configuration)
- ✅ Master-replica replication (eventual consistency)
- ✅ Health checks and failover
- ✅ Distributed transactions (basic 2-phase commit)
- ✅ Cluster-aware query routing
- ✅ Prometheus metrics for cluster monitoring

**Key Features:**

- 3-node cluster setup
- Automatic failover
- Consistent read replicas
- Data rebalancing on node failure
- Helm chart for Kubernetes

**Target Metrics:**

- Sub-second failover
- 99.9% uptime SLA
- Linear scaling up to 10 nodes

---

### **Phase 4: v0.4 - Advanced Query & ReQL**

**Status:** 📋 Planned  
**Timeline:** Q2-Q3 2025  
**Focus:** Query completeness, ReQL parity

**What ships:**

- ✅ Full ReQL compatibility (99% of terms)
- ✅ Advanced query operators (join, group, aggregate)
- ✅ Subqueries and nested expressions
- ✅ User-defined functions (UDF)
- ✅ Query optimization engine (cost-based planner)
- ✅ Explain plans and query analysis

**Key Features:**

- Joins across tables and databases
- Complex aggregations (windowing, etc.)
- Expressions and computed fields
- Query caching
- Performance profiling tools

---

### **Phase 5: v0.5 - GraphQL Beta**

**Status:** 📋 Planned  
**Timeline:** Q3 2025  
**Focus:** GraphQL support, subscriptions

**What ships:**

- ✅ GraphQL query endpoint (`/graphql`)
- ✅ GraphQL subscriptions for changefeeds
- ✅ GraphQL Playground UI (`/graphql/playground`)
- ✅ Automatic schema generation
- ✅ Federation support (experimental)

**Key Features:**

- Full GraphQL type system mapping
- Real-time subscriptions
- Batch data loading (N+1 prevention)
- Schema introspection
- GraphQL-to-ReQL compiler

**Note:** Marked as Beta - breaking changes possible in v0.6+

---

### **Phase 6: v0.6 - Vector & ML Integration**

**Status:** 📋 Planned  
**Timeline:** Q3-Q4 2025  
**Focus:** Vector search, ML tooling

**What ships:**

- ✅ HNSW-based vector index plugin
- ✅ Vector search operators (similarity, distance)
- ✅ Embedding generation helpers
- ✅ Python integration (PyO3 bindings)
- ✅ Basic statistical functions

**Key Features:**

- Hybrid search (full-text + vector)
- Vector quantization for memory efficiency
- Batch similarity search
- Python client library
- ML-optimized data types

---

### **Phase 7: v0.7 - Time-Series & Analytics**

**Status:** 📋 Planned  
**Timeline:** Q4 2025  
**Focus:** Time-series workloads, analytics

**What ships:**

- ✅ Time-series specific indexes
- ✅ Retention policies and auto-compaction
- ✅ Time-bucketing and downsampling
- ✅ Window functions (rolling, cumulative)
- ✅ InfluxDB-like query operators

**Key Features:**

- Efficient time-series storage
- High-cardinality tag support
- Automatic data expiration
- Time-aligned aggregations
- Time travel queries

---

### **Phase 8: v0.8 - Security & Auth Hardening**

**Status:** 📋 Planned  
**Timeline:** Q1 2026  
**Focus:** Enterprise security

**What ships:**

- ✅ Full OAuth2/OIDC support
- ✅ Role-based access control (RBAC)
- ✅ Field-level encryption
- ✅ Audit logging
- ✅ TLS 1.3 everywhere
- ✅ Honeytrap deep integration

**Key Features:**

- Multi-tenant isolation
- Fine-grained permissions
- Data classification
- Compliance features (GDPR, HIPAA)
- Security event streaming

---

### **Phase 9: v0.9 - Production Hardening**

**Status:** 📋 Planned  
**Timeline:** Q1-Q2 2026  
**Focus:** Stability, observability, performance

**What ships:**

- ✅ Extended testing suite (chaos engineering)
- ✅ Advanced monitoring and alerting
- ✅ Performance profiling tools
- ✅ Diagnostic utilities
- ✅ Migration tools from other databases

**Key Features:**

- Comprehensive observability (tracing, metrics, logs)
- High-load testing (1M+ QPS)
- Backup & restore capabilities
- Disaster recovery procedures
- Documentation completion

---

### **Phase 10: v1.0 - Production Ready**

**Status:** 📋 Planned  
**Timeline:** Q2 2026  
**Focus:** Stability, completeness, polish

**What ships:**

- ✅ Stable public API
- ✅ LTS release with 3-year support
- ✅ Official client libraries (Rust, Python, Node.js, Go)
- ✅ Production deployment guides
- ✅ Community edition
- ✅ Enterprise support options

**Key Features:**

- Rock-solid stability
- Comprehensive documentation
- Migration guides from RethinkDB
- SLA commitments
- Backward compatibility guarantee

**Target Metrics:**

- 99.99% uptime SLA
- Sub-millisecond query latency (p99)
- 1M+ QPS throughput
- Clustering up to 100+ nodes
- Vector search on 1B+ embeddings

---

## Feature Matrix

| Feature         | v0.1 | v0.2 | v0.3 | v0.4 | v0.5 | v0.6 | v0.7 | v0.8 | v0.9 | v1.0 |
| --------------- | ---- | ---- | ---- | ---- | ---- | ---- | ---- | ---- | ---- | ---- |
| REST API        | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   |
| ReQL            | ✅   | ✅   | ✅   | ✅✅ | ✅✅ | ✅✅ | ✅✅ | ✅✅ | ✅✅ | ✅✅ |
| WebSocket       | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   |
| Indexing        | ⏳   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   |
| Transactions    | ⏳   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   |
| Clustering      | ⏳   | ⏳   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   |
| GraphQL         | ❌   | ❌   | ❌   | ❌   | ✅   | ✅   | ✅   | ✅   | ✅   | ✅   |
| Vector Search   | ❌   | ❌   | ❌   | ❌   | ❌   | ✅   | ✅   | ✅   | ✅   | ✅   |
| Time-Series     | ❌   | ❌   | ❌   | ❌   | ❌   | ❌   | ✅   | ✅   | ✅   | ✅   |
| Python Bindings | ❌   | ❌   | ❌   | ❌   | ❌   | ✅   | ✅   | ✅   | ✅   | ✅   |
| RBAC            | ⏳   | ⏳   | ⏳   | ⏳   | ⏳   | ⏳   | ⏳   | ✅   | ✅   | ✅   |

---

## Current Status: v0.1 (Q4 2025)

**Completion:** ~40%

### What's done in v0.1

- ✅ Core storage engine (slab + B-Tree)
- ✅ HTTP server and basic REST API
- ✅ WebSocket support
- ✅ ReQL AST and basic compiler
- ✅ Query executor
- ✅ Basic CLI
- ✅ Kubernetes manifests

### What's left for v0.1

By end of Q4 2025:

- 🚧 Query executor edge cases
- 🚧 Error handling polish
- 🚧 Documentation completeness
- 🚧 Integration test coverage

---

## Next Steps (v0.2 Planning)

1. **Performance Profiling** - Benchmark current implementation
2. **Index Implementation** - B-Tree based table indices
3. **Transaction Support** - Single-node ACID transactions
4. **WAL Optimization** - Batch writes and compression
5. **Memory Management** - Cache optimization

---

## Design Principles

1. **Stability First** - Compatibility and breaking changes are carefully managed
2. **Feature Completeness** - Each version is feature-complete for its scope
3. **Performance** - Every version should improve performance over previous
4. **Documentation** - Each phase includes comprehensive docs and examples
5. **Community** - Feedback drives prioritization

---

## Long-term Vision (Post v1.0)

After v1.0 stability is achieved:

- **Distributed Query Execution** - Cross-node query planning
- **Machine Learning Integration** - TensorFlow/PyTorch support
- **Calculus Engine** - ODE/PDE solving
- **Real-time Data Pipelines** - Streaming data ingestion
- **AI-powered Query Optimization** - ML-based planner
- **Commercial Offerings** - Managed cloud service

---

**Last Updated:** December 12, 2025  
**Current Status:** v0.1-alpha (Q4 2025)  
**Next Phase:** v0.2 (Q1 2026)  
**Production Target:** v1.0 (Q2 2026)
