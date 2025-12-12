# PhotonDB Documentation

Welcome to the **PhotonDB Developer Documentation**.  
This landing page links all major architecture, design, protocol, and developer-related documents for the PhotonDB project.

---

# 📘 Overview

PhotonDB is a high‑performance, AI‑native, vector‑ready, and time‑series‑capable document database written in Rust.  
Its architecture is modular, plugin‑driven, distributed, and optimized for modern workloads such as:

- ANN Vector Search (HNSW, IVF, PQ)
- Real‑time Time‑Series Streams
- Classic Document Queries (ReQL‑P)
- Clustering, Replication & Scaling
- Cap’n Proto RPC + JSON Protocols
- Extensible Plugin System

---

# 📚 Core Documents

### 1. Architecture

- **ARCHITECTURE_OVERVIEW.md**
- **STORAGE_ENGINE_OVERVIEW.md**
- **QUERY_ENGINE_INTERNALS.md**
- **EXECUTION_PIPELINE.md**
- **CLUSTER_DESIGN.md**
- **SECURITY_MODEL.md**

---

# 🧠 AI & Vector Features

- **VECTOR_SEARCH_DESIGN.md**
- **INDEXING_OVERVIEW.md**
- **TIME_SERIES_ENGINE_DESIGN.md**

---

# 📝 Query Language & API

- **PHOTONDB_QUERY_LANGUAGE_SPEC.md**
- **RPC_PROTOCOL_DESIGN.md**
- **ADMIN_API_SPEC.md** *(coming soon)*

---

# 🔌 Plugin System

- **PLUGIN_SYSTEM_OVERVIEW.md**
- Developer API for query, storage, vector, network, and cluster plugins.

---

# 🏗 Developer Resources

- **DEVELOPER_GUIDE.md**
- **CONTRIBUTING.md**
- **PHOTONDB_NAMING_GUIDE.md**
- Coding conventions, module layout, naming patterns.

---

# 🚀 Deployment

- Kubernetes manifests  
- Helm charts (`helm/rethinkdb` → `helm/photondb` roadmap)  
- Packaging tools (`.deb`, `.rpm`, `.msi`, `.dmg`)  

*(Dedicated deployment docs coming soon)*

---

# 🔭 Roadmap Documents

Future roadmap includes:

- Distributed transactions  
- Learned indexes  
- GPU/Photonic acceleration  
- WASM plugin sandbox  
- Multi‑region sharding  
- Full SQL‑ish query engine (optional layer)

---

# 📂 Repository Structure (Quick Reference)

```
src/
  storage/
  query/
  reql/
  network/
  cluster/
  plugin/
  server/

docs/
  (All design docs listed above)

proto/
  (Cap’n Proto schemas)
```

---

# 🧭 How to Navigate This Documentation

If you're new to the project, start here:

1. **ARCHITECTURE_OVERVIEW.md**  
2. **EXECUTION_PIPELINE.md**  
3. **QUERY_ENGINE_INTERNALS.md**  
4. **STORAGE_ENGINE_OVERVIEW.md**  

For AI/vector workloads:

5. **VECTOR_SEARCH_DESIGN.md**  
6. **INDEXING_OVERVIEW.md**

For time‑series workloads:

7. **TIME_SERIES_ENGINE_DESIGN.md**

---

# 🆘 Getting Help

Join the PhotonDB developer circle, open issues, propose PRs, or request architectural reviews.

PhotonDB welcomes contributors from systems programming, databases, AI/ML, distributed systems, and Rust communities.

---

# 🌟 Summary

This documentation hub provides:

- A complete technical map of PhotonDB internals  
- Clear extension points for plugins and new engines  
- A unified language spec (ReQL‑P)  
- Foundations for distributed, AI‑accelerated database workloads  

PhotonDB is built to power the next generation of real‑time, intelligent, and high‑performance applications.

---

*Generated automatically — sync this file to `docs/index.md` in your repository.*
