# PhotonDB Development Index

A quick reference to all documentation for PhotonDB development.

## 📚 Start Here

1. **New to PhotonDB?** → Read [README.md](../README.md)
2. **Setting up dev environment?** → Read [DEVELOPMENT_SETUP.md](DEVELOPMENT_SETUP.md)
3. **Understanding the architecture?** → Read [architecture/README.md](architecture/README.md)
4. **Want to know the timeline?** → Read [DEVELOPMENT_ROADMAP.md](DEVELOPMENT_ROADMAP.md)
5. **Current phase status?** → Read [V0.1_STATUS.md](V0.1_STATUS.md)

---

## 📖 Documentation Guide

### Quick Links

| Document                                               | Purpose                        | For Whom                    |
| ------------------------------------------------------ | ------------------------------ | --------------------------- |
| [DEVELOPMENT_SETUP.md](DEVELOPMENT_SETUP.md)           | Step-by-step development guide | Developers setting up       |
| [V0.1_STATUS.md](V0.1_STATUS.md)                       | Current status & next steps    | Everyone joining project    |
| [DEVELOPMENT_ROADMAP.md](DEVELOPMENT_ROADMAP.md)       | Full timeline & features       | Product managers, planners  |
| [GRAPHQL_STRATEGY.md](GRAPHQL_STRATEGY.md)             | GraphQL implementation plan    | API developers              |
| [CLI_IMPLEMENTATION.md](CLI_IMPLEMENTATION.md)         | CLI commands reference         | CLI developers              |
| [NETWORK_IMPLEMENTATION.md](NETWORK_IMPLEMENTATION.md) | Network protocol details       | Network/protocol developers |

### Architecture Docs

| Document                                                                 | Content                                |
| ------------------------------------------------------------------------ | -------------------------------------- |
| [architecture/README.md](architecture/README.md)                         | Database hierarchy & storage structure |
| [architecture/database_hierarchy.md](architecture/database_hierarchy.md) | Detailed schema documentation          |

### API Reference

| Endpoint            | Doc                                  |
| ------------------- | ------------------------------------ |
| HTTP REST API       | [api/http.md](api/http.md)           |
| WebSocket API       | [api/websocket.md](api/websocket.md) |
| ReQL Query Language | [api/reql.md](api/reql.md)           |
| Admin API           | [api/admin.md](api/admin.md)         |

### Deployment

| Document                 | Purpose                     |
| ------------------------ | --------------------------- |
| deployment/production.md | Production deployment guide |
| deployment/docker.md     | Docker containerization     |
| deployment/kubernetes.md | Kubernetes deployment       |
| deployment/monitoring.md | Metrics & monitoring setup  |

---

## 🚀 Quick Start Commands

```bash
# Clone and build
git clone https://github.com/afeldman/PhotonDB.git
cd PhotonDB
cargo build --release

# Run server
./target/release/photondb serve --dev-mode

# Test it
photondb db create test_app
photondb table create --db test_app users

# Run tests
cargo test

# Read development guide
open docs/DEVELOPMENT_SETUP.md
```

---

## 🎯 Development Areas

### Storage & Persistence

- **Files:** `src/storage/`, `src/btree/`
- **Docs:** [DEVELOPMENT_SETUP.md#storage](DEVELOPMENT_SETUP.md#key-development-areas)
- **Status:** ✅ v0.1 Complete, working on indexing for v0.2
- **Next:** Add B-Tree indices, transactions

### Query Execution

- **Files:** `src/reql/`, `src/query/`
- **Docs:** [DEVELOPMENT_SETUP.md#query](DEVELOPMENT_SETUP.md#key-development-areas)
- **Status:** ✅ Basic execution, edge cases pending
- **Next:** Advanced operators, optimization

### HTTP Server & API

- **Files:** `src/server/`, `src/network/`
- **Docs:** [api/http.md](api/http.md)
- **Status:** ✅ Core endpoints done
- **Next:** GraphQL support (v0.5)

### Clustering

- **Files:** `src/cluster/`
- **Docs:** [NETWORK_IMPLEMENTATION.md](NETWORK_IMPLEMENTATION.md)
- **Status:** 🚧 Basic structure, full implementation in v0.3
- **Next:** Node discovery, replication

### Vector Search

- **Planned:** v0.6
- **Type:** Plugin system
- **Status:** 📋 Not yet started

### Time-Series

- **Planned:** v0.7
- **Features:** Retention, bucketing, downsampling
- **Status:** 📋 Not yet started

---

## 📋 Current Tasks (v0.1)

### Immediate (Q4 2025)

- [ ] Complete query executor edge cases
- [ ] Polish error handling
- [ ] Expand integration tests
- [ ] Complete API documentation
- [ ] Security review

### Short-term (Q1 2026 - v0.2)

- [ ] Performance profiling
- [ ] Index implementation
- [ ] Transaction support
- [ ] Query optimization
- [ ] Cache improvements

---

## 🔧 Useful Commands

```bash
# Build & Run
cargo build --release
cargo run --release -- serve --dev-mode
RUST_LOG=debug cargo run -- serve --dev-mode

# Testing
cargo test
cargo test storage::
RUST_LOG=debug cargo test -- --nocapture

# Quality
cargo fmt
cargo clippy -- -D warnings
cargo doc --no-deps --open

# Performance
cargo bench
cargo build --release && valgrind ./target/release/photondb serve

# Deployment
docker build -t photondb:0.1 .
helm install photondb ./helm/photondb/
```

---

## 📊 Project Status

**Version:** v0.1-alpha (Q4 2025)  
**Completion:** ~40%  
**Target:** v1.0 (Q2 2026)

### v0.1 Progress

- ✅ Core engine (storage, query, API)
- ✅ HTTP server & REST endpoints
- ✅ WebSocket support
- 🚧 Edge cases & error handling
- 🚧 Documentation & tests

### Roadmap

- **v0.1** (Q4 2025) - Core engine ← Current
- **v0.2** (Q1 2026) - Storage optimization & indexing
- **v0.3** (Q2 2026) - Clustering & replication
- **v0.4** (Q2-Q3 2026) - Full ReQL compatibility
- **v0.5** (Q3 2026) - GraphQL beta
- **v0.6** (Q3-Q4 2026) - Vector search & ML
- **v0.7** (Q4 2026) - Time-series features
- **v0.8** (Q1 2026) - Security hardening
- **v0.9** (Q1-Q2 2026) - Production testing
- **v1.0** (Q2 2026) - Production release

---

## 🤝 Contributing

### First Time?

1. Read [DEVELOPMENT_SETUP.md](DEVELOPMENT_SETUP.md)
2. Build locally
3. Run tests
4. Pick a small issue to start
5. Follow code style (rustfmt, clippy)

### Code Style

- Rust 2021 edition
- Follow `rustfmt` conventions
- No clippy warnings
- Doc comments on public APIs
- Tests for new code

### Commit Style

```
feat: Add new feature
fix: Fix bug
docs: Update documentation
perf: Performance improvement
test: Add tests
refactor: Code cleanup
```

---

## 🔗 External Resources

### RethinkDB

- [RethinkDB API](https://rethinkdb.com/api/)
- [RethinkDB Documentation](https://rethinkdb.com/docs/)
- [RethinkDB GitHub](https://github.com/rethinkdb/rethinkdb)

### Rust

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Tokio Async Runtime](https://tokio.rs/)
- [Axum Web Framework](https://docs.rs/axum/)

### Tools

- [VS Code Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Clippy Linter](https://github.com/rust-lang/rust-clippy)

---

## ❓ FAQ

**Q: Where do I start?**  
A: Read [DEVELOPMENT_SETUP.md](DEVELOPMENT_SETUP.md), build locally, run tests.

**Q: How do I run the server?**  
A: `./target/release/photondb serve --dev-mode`

**Q: How do I test my changes?**  
A: `cargo test` for all tests, `RUST_LOG=debug cargo test` for verbose output

**Q: Where's the architecture documented?**  
A: Check [architecture/README.md](architecture/README.md)

**Q: When's GraphQL coming?**  
A: Planned for v0.5 (Q3 2026) - see [GRAPHQL_STRATEGY.md](GRAPHQL_STRATEGY.md)

**Q: Can I contribute?**  
A: Yes! See contributing section above. Start with a small issue.

---

## 📞 Support

- **GitHub Issues:** Report bugs, request features
- **GitHub Discussions:** Ask questions, share ideas
- **Docs:** Check relevant documentation files
- **Code Comments:** Look for inline documentation

---

**Last Updated:** December 12, 2025  
**Ready to develop?** Start with [DEVELOPMENT_SETUP.md](DEVELOPMENT_SETUP.md)!

🚀 Let's build PhotonDB!
