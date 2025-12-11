# PhotonDB 1.0

**PhotonDB** is a modern Rust reimplementation and modernization of RethinkDB, built for vectors, time-series, and analytical workloads.

PhotonDB takes the proven architecture, query philosophy, and operational wisdom from RethinkDB and rebuilds it from the ground up in Rust with:

- Performance optimizations for 2025+ workloads
- Modern async/await patterns with Tokio
- Vector search and AI/ML capabilities
- Time-series data support
- Cloud-native deployment (Kubernetes-first)

> **Status:** v0.1-alpha (Q4 2025) - Core engine complete. Rapid development toward v1.0 production release (Q2 2026).

---

## Features

### Core

- ✅ Rust web server (Axum 0.7)
- ✅ HTTP/REST API and WebSocket support (changefeeds, live updates)
- ✅ ReQL-style query language (`src/reql`) with AST, terms and types
- ✅ Query compiler and executor (`src/query`)
- ✅ JSON wire protocol compatible with classic RethinkDB clients (subset)

### Storage

- ✅ Custom slab + B-Tree based storage engine (`src/storage`, `src/btree`)
- ✅ WAL and crash recovery
- ✅ Pluggable storage backends (mock, slab, B-Tree)
- ✅ Benchmarks and production tests for the slab engine

### Clustering & Observability

- ✅ Cluster discovery, health checks and scaling (`src/cluster`)
- ✅ Kubernetes integration (`k8s/`, `helm/`)
- ✅ Prometheus metrics (`/_metrics`)
- ✅ Health endpoints (`/health`, `/health/live`, `/health/ready`, `/health/startup`)

### Security & Server

- ✅ Middleware, auth hooks and security layer (`src/server`)
- ✅ OAuth2/JWT-ready design (see `docs/security`)
- ✅ Admin dashboard (`static/admin.html`)

## Roadmap: v0.1 → v1.0

PhotonDB follows a clear 10-phase release plan from Q4 2025 to Q2 2026, each phase adding significant functionality:

- **v0.1** (Q4 2025 - current) - Core engine & REST API
- **v0.2** (Q1 2026) - Storage optimization & indexing
- **v0.3** (Q2 2026) - Clustering & replication
- **v0.4** (Q2-Q3 2026) - Full ReQL compatibility
- **v0.5** (Q3 2026) - GraphQL beta
- **v0.6** (Q3-Q4 2026) - Vector search & ML integration
- **v0.7** (Q4 2026) - Time-series features
- **v0.8** (Q1 2026) - Security hardening
- **v0.9** (Q1-Q2 2026) - Production testing
- **v1.0** (Q2 2026) - Production-ready stable release

See [DEVELOPMENT_ROADMAP.md](docs/DEVELOPMENT_ROADMAP.md) for detailed phase plans.

---

## Repository Layout

- `src/`

  - `bin/photondb.rs` – current server/CLI entrypoint
    > Will be renamed to `photondb` in a future refactor.
  - `reql/` – ReQL AST, datum types and protocol helpers
  - `query/` – query compiler & executor
  - `storage/` – storage engine abstraction and implementations
    - `slab/` – slab allocator, cache, compression, metadata & tests
    - `btree/` – B-Tree structures and pager
  - `network/` – auth, connections, wire protocol, QUIC, server
  - `server/` – HTTP routes, WebSocket handlers, middleware, security
  - `cluster/` – discovery, metrics, scaling, k8s helpers
  - `plugin/` – plugin loader, registry and traits

- `proto/` – Cap’n Proto schemas
- `docs/` – architecture, network, CLI, security, packaging, clustering, deployment
- `helm/`, `k8s/` – Helm chart & Kubernetes manifests
- `packaging/` – build scripts for `.deb`, `.rpm`, `.dmg`, `.msi`
- `tests/` – integration tests (network, k8s scaling, query execution)
- `benches/` – benchmarks

---

## Quick Start

```bash
cargo build --release
./target/release/photondb serve --dev-mode
```

- HTTP API: `http://127.0.0.1:8080`
- Admin UI: `http://127.0.0.1:8080/_admin`

> The binary is still named `rethinkdb` for compatibility and will be renamed
> to `photondb` in a future update.

---

## CLI Examples

```bash
rethinkdb db create myapp
rethinkdb table create --db myapp users
rethinkdb db list
rethinkdb table list --db myapp
```

More examples in `docs/CLI_IMPLEMENTATION.md`.

---

## Roadmap

1. **PhotonDB identity**

   - Rename binaries/CLI (`rethinkdb` → `photondb`)
   - Replace `PHOTONDB_*` env vars with `PHOTONDB_*`
   - Update docs & examples

2. **Core hardening**

   - Expand tests, fuzzing, benchmarks
   - Improve transaction semantics

3. **Vector & time-series**

   - Add vector index plugin (ANN/HNSW)
   - Add time-series engine + retention

4. **Cap’n Proto RPC**

   - Implement fast binary RPC
   - Provide example client SDKs

5. **Deployment**
   - Production-ready Helm chart
   - Packaging for Linux/macOS/Windows

---

## License

See `LICENSE` for details.
