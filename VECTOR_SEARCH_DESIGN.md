# PhotonDB Vector Search Design

PhotonDB is designed to support **native high‑performance vector search** for AI,
scientific computing, embeddings, and real‑time analytics.  
This document outlines the architecture, algorithms, data formats, indexing
strategies, and cluster‑level behavior for vector search in PhotonDB.

---

# 1. Goals of PhotonDB Vector Search

PhotonDB’s vector subsystem aims to provide:

- **High‑performance ANN (Approximate Nearest Neighbor) search**
- **Multiple index types:** HNSW, IVF‑Flat, IVF‑PQ, Scalar Quantization
- **Real‑time inserts and deletes**
- **Persistence via PhotonDB’s slab + WAL storage system**
- **Integration with the plugin system**
- **Cluster‑aware distributed vector search**
- **Compatibility with ML embeddings (OpenAI, HuggingFace, q.ANT chips, etc.)**

---

# 2. Architecture Overview

```
                Query Engine
                     |
                     v
             Vector Query Planner
                     |
                     v
        +-----------------------------+
        |   Vector Index Subsystem    |
        |  (Plugin-Based Architecture)|
        +-----------------------------+
             /            |            \
            v             v             v
      HNSW Index     IVF-Flat Index   IVF-PQ Index
            |             |             |
            +-------------+-------------+
                          |
                          v
            PhotonDB Storage Engine +
            Slab allocator + WAL + B-Tree
```

---

# 3. Vector Index Plugin Interface

All vector index implementations follow a common trait:

```rust
pub trait VectorIndex {
    fn insert(&self, id: u64, embedding: &[f32]);
    fn delete(&self, id: u64);
    fn search(&self, query: &[f32], k: usize) -> Vec<SearchResult>;
    fn dimension(&self) -> usize;
    fn index_type(&self) -> VectorIndexType;
    fn flush(&self);
}
```

### Why plugin-based?

- Decouples vector logic from core DB
- Allows multiple ANN algorithms
- Enables GPU/accelerator engines (future)
- Allows users to customize index behavior

---

# 4. Index Types

## 4.1 HNSW (Hierarchical Navigable Small World)

### Strengths:
- Extremely fast search
- High recall
- Great for dynamic insert/delete workloads

### Weaknesses:
- Higher memory usage
- Complex consistency model when persisting

### Persistence Strategy:
- Each layer stored in slab pages
- Node connections stored as adjacency lists
- WAL stores node additions + edge changes

---

## 4.2 IVF‑Flat (Inverted File Index)

### Strengths:
- Very scalable
- Good for large embedding corpora
- Great performance when combined with batching

### Weaknesses:
- Rebuilding clusters required for best performance
- Slower inserts compared to HNSW

### Data Layout:
- Centroid table (dense)
- Lists/buckets stored as slab segments
- Periodic reclustering (background job)

---

## 4.3 IVF‑PQ (Product Quantization)

### Strengths:
- Large compression ratios
- Good for 100M+ vector scale

### Weaknesses:
- Lower recall unless tuned
- Re-training PQ codebooks required

### Storage Layout:
- PQ codebooks stored as metadata
- Encoded vectors stored in compact slots
- Search applies asymmetric distance computation

---

# 5. Embedding Storage Format

PhotonDB stores each vector as:

```
struct VectorEntry {
    id: u64,
    dims: u32,
    data: Vec<f32>,      // or PQ codes
    metadata: Datum,     // arbitrary JSON-like metadata
}
```

Metadata may include:

- namespace
- timestamp
- external ID
- labels/classes
- version

---

# 6. Query Language Extensions

PhotonDB will extend ReQL with:

### Vector Insert

```
r.table("vectors").insert({
  id: 1,
  embedding: [0.1, 0.2, 0.3, ...]
})
```

### Vector Search

```
r.table("vectors").nearest([0.1, 0.2, ...], { index: "hnsw", k: 10 })
```

### Hybrid Queries

```
r.table("vectors")
 .nearest(vector, {k: 5})
 .filter({ category: "science" })
```

These become:

- VectorIndex.search()
- Filtered by Query Engine
- Joined with metadata stored as regular documents

---

# 7. Integration with PhotonDB Storage Engine

Vector index files persist through:

- **Slab pages** (node data, PQ codes, lists)
- **WAL** (mutations, inserts, deletes, codebook updates)
- **B‑Tree metadata** (index definitions, dimensionality, algorithm type)

Persistence model:

```
Insert vector → WAL log → Slab pages → Index rebuild (async)
```

Consistency guarantees:
- WAL replay reconstructs full index state
- HNSW mutation logs ensure edge consistency
- IVF lists reconstructed from slabs

---

# 8. Cluster‑Level Vector Search

PhotonDB distributes vector search across nodes.

## 8.1 Node Types

- **Vector Node** (stores ANN partitions)
- **Coordinator Node** (dispatches multi-shard search)
- **Hybrid Node** (runs both ANN + document queries)

## 8.2 Sharding Strategy

### Option A — Hash‑based
```
shard = hash(id) % N
```

### Option B — K‑Means partitioning (better)
```
embedding → centroid → assigned node
```

### Option C — Hybrid
Balanced shard sizes + locality-aware routing.

---

# 9. Distributed ANN Search Pipeline

```
Client Query
   |
Coordinator Node
   |
Broadcast query to ANN shards
   |
Each shard performs K-NN locally
   |
Coordinator merges results (top‑k)
   |
Client receives full result set
```

Latency target: **5–20 ms** for typical ANN workloads.

---

# 10. Distance Metrics

PhotonDB will support:

- L2 (Euclidean)
- Inner Product
- Cosine Similarity
- Hamming (for PQ / binary vectors)

Distance selection is part of index creation options.

---

# 11. Index Management

PhotonDB will support:

### Creating an index
```
r.table("vectors").indexCreate("ann_hnsw", {
  type: "hnsw",
  dims: 768,
  m: 32,
  efConstruction: 200
})
```

### Listing indexes
```
r.table("vectors").indexList()
```

### Deleting an index
```
r.table("vectors").indexDrop("ann_hnsw")
```

### Rebuilding (IVF/PQ)
```
r.table("vectors").indexRebuild("ivf_pq")
```

---

# 12. Background Jobs

PhotonDB will include:

- HNSW graph compaction
- IVF reclustering
- PQ codebook training
- Index statistics and health maintenance

These jobs run through the upcoming **PhotonDB Task Engine**.

---

# 13. Future Directions

### GPU‑accelerated vector search
- CUDA kernels for distance computation  
- Tensor cores for high‑dim inferences  
- External accelerators via plugin API  

### q.ANT / photonic chip integration
- Ultralow-latency optical compute  
- Native support for PQ/HNSW acceleration  

### Hybrid ANN + Symbolic Querying
- Combine embedding similarity with structured filters
- Vector search inside join pipelines

### Temporal Vector Indexes
- ANN + time-decay weighting
- Sliding window embedding search

---

# 14. Summary

PhotonDB’s vector search subsystem is:

- Plugin-based  
- High-performance  
- Persistent  
- Cluster-ready  
- Extensible to future ML workloads  

It integrates cleanly with PhotonDB’s storage, query engine, and replication
roadmap—making PhotonDB capable of powering AI-native applications at scale.
