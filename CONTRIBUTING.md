# Contributing to PhotonDB

Thank you for your interest in contributing to **PhotonDB**!  
This document provides guidelines and best practices for contributing code, documentation, tests, and ideas.

---

## 🧭 Project Philosophy

PhotonDB is a **Rust‑native, modular database engine** inspired by RethinkDB but designed for modern workloads such as vector search, time-series data, and distributed clusters.

Contributions should follow these principles:

- **Safety first:** Rust guarantees memory safety; PhotonDB extends this with careful API design.
- **Modularity:** Everything should live behind clear traits and modules.
- **Performance:** Storage, networking, and query execution should be optimized.
- **Extensibility:** Plugins, storage engines, and protocols should remain replaceable.
- **Clarity:** Code must be readable, documented, and tested.

---

## 🛠️ How to Contribute

### 1. Fork & Clone

```bash
git clone https://github.com/afeldman/PhotonDB.git
cd PhotonDB
```

### 2. Create a Feature Branch

```bash
git checkout -b feature/my-new-feature
```

### 3. Build & Test

Ensure the project builds:

```bash
cargo build
```

Run all tests:

```bash
cargo test
```

Run clippy and formatting:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features
```

---

## 🧪 Tests & Benchmarks

PhotonDB includes:

- **Unit tests** (`tests/`, `src/**`)
- **Integration tests** (networking, k8s scaling, query execution)
- **Benchmarks** (`benches/`)

When adding new functionality:

- Write **unit tests** for isolated logic.
- Add **integration tests** if the feature touches storage, query, or network layers.
- Consider adding a **benchmark** if performance-critical.

---

## 🧱 Code Structure Overview

Key areas of the codebase:

- `src/reql/` – ReQL AST & protocol structures  
- `src/query/` – Query compiler & executor  
- `src/storage/` – Slab storage, B‑Tree engine, WAL  
- `src/network/` – Networking, connections, QUIC, protocol  
- `src/server/` – HTTP routes, middleware, admin UI  
- `src/cluster/` – discovery, metrics, scaling  
- `src/plugin/` – plugin loader & traits  
- `proto/` – Cap’n Proto schemas  
- `docs/` – architecture, API, security, deployment  

Please follow existing structure unless your contribution improves it in a clear way.

---

## 🧩 Coding Guidelines

### Rust Style

- Use **Rust 2021** edition.
- Run `cargo fmt` before committing.
- Use `clippy` and resolve warnings when possible.
- Prefer `Arc` + traits instead of generics in many-core contexts.
- Prefer small modules and files; database code grows quickly.

### Documentation

- Public modules/functions must include doc comments.
- Architectural or complex decisions should be documented in `docs/`.

### Commits

Follow conventional commit styles when possible:

```
feat: add vector index plugin
fix: correct WAL recovery bug
docs: update query execution docs
refactor: reorganize slab allocator
test: add integration tests for cluster health checks
```

---

## 🔐 Security

Security-sensitive code lives in:

- `src/network/auth.rs`
- `src/server/security.rs`
- `docs/security/`

If submitting security-related PRs:

- Add tests where possible.
- Document threat models or assumptions.
- Never commit secrets or tokens.

---

## 🗳️ Pull Request Process

1. Ensure your branch is up to date with `develop`.
2. Push your feature branch.
3. Open a Pull Request with:
   - A clear description
   - Linked issue (if applicable)
   - Test coverage where needed
4. The maintainers will review and may request changes.
5. Once approved, your PR will be merged into `develop`.

---

## ❓ Questions / Discussions

Use GitHub Issues for:

- Bug reports
- Feature requests
- Architecture proposals
- Performance discussions

---

## ❤️ Thank You

PhotonDB grows through community-driven development.  
Thank you for contributing your time, ideas, and code!
