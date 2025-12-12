# PhotonDB

PhotonDB is a modular, Rust-native document and scientific computing database,
inspired by RethinkDB but reimagined for modern Rust, vectors and time-series
workloads.

It currently implements a full Rust server with HTTP/REST API, WebSockets,
a custom storage engine, clustering, and a ReQL-compatible query language.

> **Status:** Experimental / early prototype. Not production-ready.

---

## 🚀 Features (Current)

### Core

- Rust web server (Axum-based)
- HTTP/REST API and WebSocket support (changefeeds, live updates)
- ReQL-style query language (`src/reql`) with AST, terms and types
- Query compiler and executor (`src/query`)
- JSON wire protocol with partial RethinkDB compatibility
- CLI support via the existing binary entrypoint

### Storage

- Custom slab + B-Tree based storage engine
- Write-Ahead Log (WAL) & crash recovery
- Pluggable storage backends (mock, slab, B-Tree)
- Benchmarks and production tests for slab engine

### Clustering & Observability

- Cluster discovery & health checks (`src/cluster`)
- Kubernetes/Helm support (`k8s/`, `helm/`)
- Prometheus metrics (`/_metrics`)
- Health endpoints (`/health/...`)
- Security middleware & future OAuth2/JWT support (`docs/security`)

---

## 📁 Repository Layout

- `src/`
  - `bin/rethinkdb.rs` — current server/CLI entrypoint  
    *(will later be renamed to `photondb`)*
  - `reql/` — AST, datum implementation, terms, protocol
  - `query/` — compiler & executor
  - `storage/` — engines and slab/B‑Tree implementation
  - `network/` — protocol, connections, QUIC transport
  - `server/` — HTTP routes, middleware, admin UI
  - `cluster/` — discovery, metrics, scaling
  - `plugin/` — plugin loader, registry, traits
- `proto/` — Cap’n Proto schemas
- `docs/` — architecture, API, CLI, security, packaging
- `helm/`, `k8s/` — deployment manifests
- `tests/` — integration tests
- `benches/` — storage/query benchmarks
- `static/` — admin dashboard

---

## 🧪 Running PhotonDB

### Build

```sh
cargo build --release
```

### Run (dev mode)

```sh
./target/release/rethinkdb serve --dev-mode
```

- HTTP API: `http://127.0.0.1:8080`
- Admin UI: `http://127.0.0.1:8080/_admin`

> The binary is still called `rethinkdb` for compatibility but will be renamed
> in a future PhotonDB refactor.

---

## 🧩 CLI Examples

```sh
rethinkdb db create myapp
rethinkdb table create --db myapp users
rethinkdb db list
rethinkdb table list --db myapp
```

More examples: `docs/CLI_IMPLEMENTATION.md`.

---

## 📚 Documentation

See the `docs/` directory for:

- Architecture diagrams
- Storage system internals
- API definitions (`docs/api/http.md`)
- CLI design
- Network/protocol details
- Security concept
- Packaging & deployment guides

---

## 🗺️ Roadmap

1. **Rename** all components from `rethinkdb` → `photondb`
2. Extend ReQL support & improve error diagnostics
3. Add vector indexing (ANN/HNSW)
4. Add time-series storage & retention
5. Implement Cap’n Proto RPC protocol + SDKs
6. Production-ready Helm chart & packaging

---

## License

See the `LICENSE` file for details.
