# PhotonDB Developer Guide

Welcome to the **PhotonDB Developer Guide**.  
This document describes how the codebase is structured, how to navigate it, how to run and debug components, and how to contribute effectively.

PhotonDB is a Rust-native, modular database engine inspired by RethinkDB, designed for modern workloads such as vector search, time-series data, and distributed operations.

---

# 1. Development Philosophy

PhotonDB emphasizes:

- **Modularity:** cleanly separated subsystems (query, storage, networking…)
- **Performance:** lock efficiency, cache-aware data structures, async networking
- **Safety:** correctness, memory safety, WAL recovery, predictable state
- **Extensibility:** plugin systems, replaceable storage engines, protocol layers
- **Clarity:** small modules, clear ownership, documented architecture

---

# 2. Project Setup

## Requirements

- Rust toolchain (stable)
- Linux or macOS recommended
- For QUIC: recent `rustls` ecosystem
- For docs: optional `graphviz`, `make`

Clone:

```sh
git clone https://github.com/afeldman/PhotonDB.git
cd PhotonDB
```

Build:

```sh
cargo build --all
```

Run:

```sh
cargo run --bin rethinkdb -- serve --dev-mode
```

(Will be renamed to `photondb` later.)

Run tests:

```sh
cargo test --all
```

Run clippy + fmt:

```sh
cargo fmt --all
cargo clippy --all-targets --all-features
```

---

# 3. Repository Structure (Deep Dive)

```
src/
├── reql/          # ReQL AST, terms, protocol translation
├── query/         # query compiler & executor
├── storage/       # slab storage, B-tree backend, WAL
├── network/       # TCP/QUIC protocol, framing, auth
├── server/        # HTTP routes, middleware, admin UI
├── cluster/       # discovery, metrics, scaling
├── plugin/        # plugin loader, traits, registry
└── bin/rethinkdb.rs  # CLI entrypoint
```

Additional directories:

- `proto/` → Cap’n Proto schemas
- `docs/` → architecture, security, API, packaging, CLI design
- `k8s/`, `helm/` → deployment tooling
- `tests/` → integration tests
- `benches/` → performance benchmarks
- `static/` → admin UI

---

# 4. Core Subsystems

## 4.1 ReQL Layer (`src/reql/`)

Responsibilities:

- Parsing/representing ReQL AST
- Converting network messages → internal query representations
- Datums, types, terms

Important files:

- `ast.rs`
- `terms.rs`
- `types.rs`
- `protocol.rs`

This layer isolates ReQL from the underlying execution engine.

---

## 4.2 Query Execution (`src/query/`)

Responsibilities:

- Validating and compiling queries
- Executing operations against storage engine
- Managing scopes, indexes, iterators
- Handling query return types

Important files:

- `compiler.rs`
- `executor.rs`

---

## 4.3 Storage Engine (`src/storage/`)

PhotonDB currently uses a modular, pluggable storage layer:

- **Slab allocator**: segmentation, caching, compression
- **B‑Tree engine**: indexing and point lookups
- **WAL**: durability guarantees

Key files:

- `engine.rs` – storage trait + high-level API
- `slab/` – allocator, metadata, slots
- `btree/` – pager, nodes, internal/external representation
- `mock.rs` – in-memory backend for testing

---

## 4.4 Network Layer (`src/network/`)

Responsibilities:

- Framing, handshake, authentication
- QUIC/TCP connection handling
- Translating incoming ReQL messages to internal queries
- Sending results to clients

Files:

- `protocol.rs`
- `connection.rs`
- `auth.rs`
- `quic.rs`
- `server.rs`

---

## 4.5 Server Layer (`src/server/`)

Responsibilities:

- Axum HTTP router for REST and admin UI
- WebSocket handler for changefeeds
- Middleware (security, logging, compression)
- Health & metrics endpoints

Files:

- `routes.rs`
- `handlers.rs`
- `middleware.rs`
- `websocket.rs`
- `security.rs`

---

## 4.6 Cluster Layer (`src/cluster/`)

Responsibilities:

- Node discovery
- Heartbeats
- Scaling logic
- Exporting cluster metrics

---

## 4.7 Plugin System (`src/plugin/`)

PhotonDB supports modular extensions:

- Vector search
- AI-driven indexes
- Time-series engines
- Custom query functions

Components:

- `traits.rs`
- `loader.rs`
- `registry.rs`

---

# 5. Development Workflows

## 5.1 Adding a New Query

Steps:

1. Add new ReQL term in `src/reql/terms.rs`
2. Extend AST/Datum definitions
3. Update `query/compiler.rs`
4. Implement execution logic in `query/executor.rs`
5. Update tests and docs

---

## 5.2 Adding a New Storage Feature

1. Modify or extend trait in `storage/engine.rs`
2. Implement in slab/B-Tree layers
3. Update WAL if persistence is required
4. Add integration tests (`tests/`)
5. Add benchmark if performance-critical

---

## 5.3 Debugging

Enable logs:

```sh
RUST_LOG=debug cargo run --bin rethinkdb -- serve
```

Enable storage and WAL tracing:

```sh
RUST_LOG=photondb_storage=trace,photondb_wal=trace
```

Use `RUST_BACKTRACE=1` for detailed crash output.

---

# 6. Testing & Benchmarking

### Run all tests:

```sh
cargo test --all
```

### Run integration tests:

```sh
cargo test -p photondb --test '*'
```

### Run benchmarks:

```sh
cargo bench
```

Benchmarks exist for:

- Slab allocator
- B‑Tree operations
- WAL performance

---

# 7. Coding Standards

- Use `cargo fmt` and `clippy` before commits
- Avoid unnecessary clones
- Prefer small, focused modules
- Document every public function
- Use traits to abstract subsystems cleanly
- Prefer `Arc<dyn Trait>` for runtime polymorphism

---

# 8. Roadmap for Developers

Near-term:

- Rename CLI & environment variables (`rethinkdb` → `photondb`)
- Strengthen test coverage
- Cap’n Proto RPC protocol implementation
- Vector search plugin prototype
- Time-series engine prototype

Long-term:

- Distributed consensus
- Multi-node replication
- GraphQL layer
- ML-driven query optimizer

---

# 9. Useful Commands

Rebuild quickly:

```sh
cargo build -q
```

Run server with verbose logs:

```sh
RUST_LOG=info,photondb=debug cargo run -- serve
```

Open admin UI:

```
http://127.0.0.1:8080/_admin
```

---

# 10. Need Help?

Use:

- GitHub Issues → bugs, proposals, questions
- Discussions → architecture debates
- PRs → code contributions

---

Thank you for contributing to PhotonDB!
Your work helps build the next-generation open-source database.
