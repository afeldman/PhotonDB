# PhotonDB Naming & Refactoring Guide
This document provides a **step‑by‑step, safe refactoring plan** for renaming the existing
RethinkDB‑based Rust project into **PhotonDB**, including binaries, crates, modules,
environment variables, config paths, documentation, and protocol identifiers.

The goal is to complete the renaming **without breaking compatibility** and without
disrupting existing APIs until the new PhotonDB identity is fully adopted.

---

# 1. Goals of the Renaming
### ✔ Create a clean PhotonDB identity  
### ✔ Maintain temporary backward compatibility  
### ✔ Avoid breaking builds during transition  
### ✔ Support “dual name mode” during migration

PhotonDB will eventually:
- expose `photondb` as the binary name  
- use `PHOTONDB_*` environment variables  
- export crate name `photondb`  
- use updated module paths and docs  
- ship updated protocol handshake identifiers  

---

# 2. Overview of Items to Rename

| Component | Current | Target | Notes |
|----------|---------|--------|-------|
| Binary | `rethinkdb` | `photondb` | staged rollout recommended |
| Crate Name | `rethinkdb` in Cargo.toml | `photondb` | rename + module refactor |
| Module Paths | `src/bin/rethinkdb.rs` | `src/bin/photondb.rs` | keep compatibility binary for now |
| Env Vars | `RETHINKDB_*` | `PHOTONDB_*` | dual read mode for transition |
| Network Handshake ID | “RethinkDB 3.0” | “PhotonDB 0.x” | still accept legacy ID |
| HTTP Paths | `/rethinkdb/...` | `/photondb/...` | admin UI migration |
| Docs | “RethinkDB 3.0 in Rust” | “PhotonDB” | update architecture diagrams |

---

# 3. Renaming Strategy

PhotonDB should use a **4‑phase renaming plan**:

---

## Phase 1 — *Soft Migration* (non‑breaking)
This can be done immediately.

### 1.1 Add new binary name, keep old
```
src/bin/photondb.rs   # new
src/bin/rethinkdb.rs  # wrapper → calls photondb::main()
```

### 1.2 Dual environment variable support
Implement a helper:

```rust
pub fn get_env(key: &str) -> Option<String> {
    std::env::var(format!("PHOTONDB_{}", key))
        .ok()
        .or_else(|| std::env::var(format!("RETHINKDB_{}", key)).ok())
}
```

### 1.3 Update README, docs, CI to mention PhotonDB

---

## Phase 2 — *Refactor Internals*
Update internal identifiers:

### 2.1 Cargo.toml
```toml
[package]
name = "photondb"
```

If you must keep the old name temporarily, add:
```toml
[lib]
name = "photondb"
```

### 2.2 Module tree
Refactor:

```
src/rethinkdb/** → src/photondb/**
```

Apply only to internally referenced names.
Keep external protocol compatibility untouched.

### 2.3 Logging identifiers
Change:

- `rethinkdb_network` → `photondb_network`
- `rethinkdb_storage` → `photondb_storage`

Use global constant for product name:

```rust
pub const PRODUCT_NAME: &str = "PhotonDB";
```

---

## Phase 3 — *Protocol Layer Migration*

PhotonDB currently identifies itself in several places:

### 3.1 ReQL JSON handshake
Legacy:
```
{"server_version": "rethinkdb-rust", "id": ...}
```

New handshake:
```
{"server": "PhotonDB", "version": "0.1", "compat": "rethinkdb-json"}
```

### 3.2 Cap’n Proto handshake update
In `handshake.capnp`:

```
serverName @0 :Text;   # "PhotonDB"
protocolVersion @1 :UInt32;
compatibilityMode @2 :Bool;
```

### 3.3 HTTP UI paths
Old:
```
/rethinkdb/
/rethinkdb/api/
/rethinkdb/admin
```

New:
```
/photondb/
```

During transition, support both routes.

---

## Phase 4 — *Deprecate Legacy Names*

### Remove old binary:
```
cargo build --bin rethinkdb   # remove support
```

### Remove `RETHINKDB_*` env var parsing  
### Remove compatibility handshake  
### Update Docker image names  
### Update Helm chart metadata  

This phase happens once PhotonDB reaches ≥ **v0.5**.

---

# 4. Files to Update (Checklist)

## Source Code
- `Cargo.toml`
- `src/bin/rethinkdb.rs` → `photondb.rs`
- `src/lib.rs`
- directories under `src/reql`, `src/server`, etc. using `rethinkdb_` prefixes
- logging targets (`tracing::info!(target="...")`)

## Documentation
- `README.md`
- `docs/architecture/**`
- `docs/api/**`
- `docs/security/**`

## Protocol Schemas
- `proto/handshake.capnp`
- `proto/query.capnp`
- `proto/types.capnp`

## Deployment
- `helm/rethinkdb/**` → rename chart
- `k8s/*.yaml` → update labels, env vars
- container image repository name

---

# 5. Compatibility Considerations

### The following should remain unchanged to avoid breaking existing clients:

- JSON ReQL protocol format  
- Basic message frames  
- Token-based request/response pattern  
- Old handshake accepted for at least two releases  

### Optional dual-mode period:
Allow selecting protocol identity:

```
PHOTONDB_COMPAT_MODE=rethinkdb
PHOTONDB_COMPAT_MODE=off
```

---

# 6. Example Final Identity (v1.0 target)

| Component | Value |
|----------|-------|
| Product | PhotonDB |
| Binary | photondb |
| Env vars | PHOTONDB_* |
| HTTP paths | /photondb/... |
| Protocol | “PhotonDB/1.0” |
| Repo name | github.com/afeldman/photondb |

---

# 7. Summary

This guide ensures:

- Safe incremental migration  
- Backward compatibility  
- Clean long‑term PhotonDB identity  

Once these steps are followed, the entire project will be fully renamed without
breaking existing deployments or developer workflows.
