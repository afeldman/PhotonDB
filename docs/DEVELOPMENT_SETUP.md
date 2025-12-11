# PhotonDB Development Setup Guide

Welcome to **PhotonDB** development! This guide will help you get started with the modern Rust reimplementation of RethinkDB.

**Current Status:** v0.1-alpha (Q4 2025)  
**Target:** v1.0 production release (Q2 2026)

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Development Environment](#development-environment)
3. [Project Structure](#project-structure)
4. [Building & Running](#building--running)
5. [Testing](#testing)
6. [Key Development Areas](#key-development-areas)
7. [Debugging](#debugging)
8. [Contributing](#contributing)

---

## Quick Start

### Prerequisites

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **Cargo** - Included with Rust
- **macOS/Linux/Windows** - Tested on macOS 13+, Ubuntu 22.04+

### Clone & Build

```bash
# Clone repository
git clone https://github.com/afeldman/PhotonDB.git
cd PhotonDB

# Build release binary
cargo build --release

# Binary is at: ./target/release/photondb
```

### Run Development Server

```bash
# Start in dev mode (no security, easier debugging)
./target/release/photondb serve --dev-mode

# Server will listen on:
# - HTTP: http://127.0.0.1:8080
# - Admin UI: http://127.0.0.1:8080/_admin
# - WebSocket: ws://127.0.0.1:8080/changes
# - Metrics: http://127.0.0.1:8080/_metrics
```

### Test Connectivity

```bash
# List databases
photondb db list

# Create a database
photondb db create test_app

# Create a table
photondb table create --db test_app users

# Insert data via REST API
curl -X POST http://127.0.0.1:8080/api/dbs/test_app/tables/users/insert \
  -H "Content-Type: application/json" \
  -d '{"id": "user_1", "name": "Alice", "email": "alice@example.com"}'
```

---

## Development Environment

### Recommended IDE Setup

**VS Code Extensions:**

```
rust-analyzer (rust-lang)
CodeLLDB (vadimcn)
Rust Test Explorer (Swatinem)
Cargo (serayuzgur)
Crates (serayuzgur)
```

### Environment Variables

**Development (default):**

```bash
# Enables debug logging and disables security
DEV_MODE=true
RUST_LOG=debug
```

**Production-like (for testing):**

```bash
DEV_MODE=false
RUST_LOG=info
PHOTONDB_CLUSTER_ENABLED=true
PHOTONDB_K8S_DISCOVERY=true
```

### Running with Custom Data Directory

```bash
./target/release/photondb serve \
  --dev-mode \
  --data-dir ./data \
  --log-dir ./logs \
  --log-level debug
```

---

## Project Structure

```
PhotonDB/
├── src/
│   ├── lib.rs                    # Library root
│   ├── bin/
│   │   └── photondb.rs           # CLI entrypoint
│   ├── server/
│   │   ├── mod.rs                # Server main logic
│   │   ├── handlers.rs           # HTTP endpoint handlers
│   │   ├── middleware.rs         # Request middleware
│   │   ├── routes.rs             # Route definitions
│   │   ├── security.rs           # Security layer
│   │   ├── websocket.rs          # WebSocket handler
│   │   └── database_handlers.rs  # Database operations
│   ├── reql/
│   │   ├── mod.rs                # ReQL module root
│   │   ├── ast.rs                # Query AST
│   │   ├── datum.rs              # Data types
│   │   ├── protocol.rs           # Wire protocol
│   │   └── terms.rs              # Query operations
│   ├── query/
│   │   ├── compiler.rs           # Query compiler
│   │   ├── executor.rs           # Query executor
│   │   └── mod.rs
│   ├── storage/
│   │   ├── mod.rs                # Storage abstraction
│   │   ├── engine.rs             # Storage engine trait
│   │   ├── database.rs           # Database hierarchy
│   │   ├── btree_storage.rs      # B-Tree backend
│   │   ├── slab/                 # Slab allocator
│   │   │   ├── allocator.rs
│   │   │   ├── cache.rs
│   │   │   ├── compression.rs
│   │   │   └── metadata.rs
│   │   └── mock.rs               # Mock storage for testing
│   ├── network/
│   │   ├── mod.rs
│   │   ├── server.rs             # TCP server
│   │   ├── quic.rs               # QUIC server
│   │   ├── connection.rs         # Connection handler
│   │   ├── protocol.rs           # Protocol implementation
│   │   ├── auth.rs               # Authentication
│   │   └── mod.rs
│   ├── cluster/
│   │   ├── mod.rs
│   │   ├── discovery.rs          # Node discovery
│   │   ├── health.rs             # Health checks
│   │   ├── k8s.rs                # Kubernetes integration
│   │   ├── metrics.rs            # Cluster metrics
│   │   └── scaling.rs            # Auto-scaling
│   └── plugin/
│       ├── mod.rs
│       ├── loader.rs             # Plugin loader
│       ├── registry.rs           # Plugin registry
│       └── traits.rs             # Plugin traits
├── docs/
│   ├── DEVELOPMENT_SETUP.md      # This file
│   ├── DEVELOPMENT_ROADMAP.md    # Release phases
│   ├── GRAPHQL_STRATEGY.md       # GraphQL plan
│   ├── architecture/             # Architecture docs
│   ├── api/                      # API reference
│   └── examples/                 # Usage examples
├── tests/
│   ├── integration_network.rs    # Network tests
│   ├── query_execution_test.rs   # Query tests
│   └── k8s_scaling_test.rs       # K8s tests
├── benches/
│   └── storage_bench.rs          # Performance benchmarks
├── proto/
│   └── *.capnp                   # Cap'n Proto schemas
├── Cargo.toml                    # Rust dependencies
├── README.md                     # Project overview
└── Taskfile.yml                  # Task automation
```

---

## Building & Running

### Build Commands

```bash
# Debug build (fast compile, slow runtime)
cargo build

# Release build (slow compile, fast runtime)
cargo build --release

# Build with specific features
cargo build --release --features "vector-search,timeseries"

# Build and run in one command
cargo run --release -- serve --dev-mode

# Build documentation
cargo doc --no-deps --open
```

### Run Commands

```bash
# Development server (recommended for debugging)
RUST_LOG=debug cargo run --bin photondb -- serve --dev-mode

# Release binary direct
./target/release/photondb serve --dev-mode

# With custom data directory
./target/release/photondb \
  serve --dev-mode \
  --data-dir /tmp/photondb_data \
  --log-level debug

# Production-like (full security enabled)
DEV_MODE=false \
PHOTONDB_CLUSTER_ENABLED=true \
./target/release/photondb serve
```

### Docker

```bash
# Build Docker image
docker build -t photondb:0.1 .

# Run in Docker
docker run -p 8080:8080 -p 28015:28015 photondb:0.1
```

---

## Testing

### Run All Tests

```bash
# All tests
cargo test

# Run with logging
RUST_LOG=debug cargo test -- --nocapture

# Specific test module
cargo test storage::

# Specific test function
cargo test test_create_database

# Test a single file
cargo test --test integration_network

# Run tests in release mode (faster)
cargo test --release
```

### Test Organization

- **Unit Tests:** In each module with `#[cfg(test)]` blocks
- **Integration Tests:** `tests/` directory for full system tests
- **Storage Tests:** `src/storage/slab/production_tests.rs`
- **Benchmarks:** `benches/` directory

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_database() {
        let storage = DefaultStorageEngine::new("./test_data").unwrap();
        let result = storage.database_engine
            .create_database("test_db")
            .await;
        assert!(result.is_ok());
    }
}
```

---

## Key Development Areas

### Phase 1 (v0.1) - Current Focus

**What's Complete:**

- ✅ Core storage engine
- ✅ HTTP server & REST API
- ✅ WebSocket support
- ✅ ReQL AST & compiler
- ✅ Query executor
- ✅ CLI interface

**What's In Progress:**

- 🚧 Query executor edge cases
- 🚧 Error handling polish
- 🚧 Integration test coverage
- 🚧 Documentation completeness

### Hot Files to Watch

1. **Query Executor** - `src/query/executor.rs`

   - Core query execution logic
   - Performance critical

2. **Storage Engine** - `src/storage/engine.rs`

   - Storage trait & backends
   - B-Tree implementation

3. **HTTP Handlers** - `src/server/handlers.rs`

   - REST endpoint implementations
   - Request/response handling

4. **ReQL Parser** - `src/reql/compiler.rs`
   - Query parsing & compilation
   - Term resolution

### Performance Hotspots

1. Query execution latency (target: <1ms)
2. Storage throughput (target: >240 ops/sec)
3. Memory usage (target: <100MB for typical workload)
4. WebSocket connection handling

---

## Debugging

### Enable Debug Logging

```bash
# Full debug logging
RUST_LOG=debug cargo run --bin photondb -- serve --dev-mode

# Specific module logging
RUST_LOG=photondb::storage=debug,photondb::query=debug cargo run --bin photondb -- serve --dev-mode

# Trace level (very verbose)
RUST_LOG=trace cargo run --bin photondb -- serve --dev-mode
```

### VS Code Debugging

Create `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "PhotonDB Debug",
      "cargo": {
        "args": ["build", "--bin=photondb", "--release"],
        "filter": {
          "name": "photondb",
          "kind": "bin"
        }
      },
      "args": ["serve", "--dev-mode"],
      "cwd": "${workspaceFolder}",
      "env": {
        "RUST_LOG": "debug"
      }
    }
  ]
}
```

### Common Issues

**Issue:** Server crashes on startup

```bash
# Check logs for details
./target/release/photondb serve --dev-mode --log-level debug

# Remove corrupted data
rm -rf ./data ./logs
./target/release/photondb serve --dev-mode
```

**Issue:** High memory usage

```bash
# Profile memory
cargo build --release
valgrind --tool=massif ./target/release/photondb serve --dev-mode
```

**Issue:** Slow queries

```bash
# Enable query logging
RUST_LOG=photondb::query=debug ./target/release/photondb serve --dev-mode

# Run benchmarks
cargo bench
```

---

## Contributing

### Workflow

1. **Create Branch**

   ```bash
   git checkout -b feature/my-feature develop
   ```

2. **Make Changes**

   - Follow Rust conventions
   - Add tests for new code
   - Update docs if needed

3. **Test Locally**

   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test
   ```

4. **Push & Create PR**
   ```bash
   git push origin feature/my-feature
   ```

### Code Style

- Follow `rustfmt` (auto-formatted)
- No clippy warnings (enforced in CI)
- Prefer explicit error handling
- Add doc comments for public APIs

### Commit Messages

```
feat: Add vector search support
fix: Handle null values in aggregations
docs: Update architecture documentation
perf: Optimize query compilation
test: Add integration tests for clustering
refactor: Simplify storage engine trait
```

### Pull Request Guidelines

- Keep PRs focused (one feature per PR)
- Add tests for new functionality
- Update documentation if needed
- Link related issues
- Ensure CI passes

---

## Next Steps

1. **Read the Architecture** - Check `docs/architecture/`
2. **Start Contributing** - Pick an issue from the roadmap
3. **Join the Community** - Check GitHub discussions
4. **Follow the Roadmap** - See `docs/DEVELOPMENT_ROADMAP.md`

---

## Helpful Commands

```bash
# Format code
cargo fmt

# Check for warnings
cargo clippy

# Run tests with output
cargo test -- --nocapture

# Generate documentation
cargo doc --no-deps --open

# Check dependencies for vulnerabilities
cargo audit

# Update dependencies
cargo update

# View dependency tree
cargo tree

# Profile binary size
cargo bloat --release

# Run in release mode
cargo run --release -- serve --dev-mode
```

---

## Resources

- **RethinkDB Docs:** https://rethinkdb.com/api/
- **Rust Book:** https://doc.rust-lang.org/book/
- **Async Rust:** https://tokio.rs/
- **Axum Framework:** https://docs.rs/axum/
- **Storage Design:** See `src/storage/README.md`

---

**Happy coding! 🚀**

For questions or issues, check the GitHub discussions or open an issue.

**Current Phase:** v0.1 (Q4 2025)  
**Next Phase:** v0.2 (Q1 2026)
