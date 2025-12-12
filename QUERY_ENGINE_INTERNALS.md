# PhotonDB Query Engine Internals

The PhotonDB Query Engine transforms ReQL-like expressions into executable
plans, evaluates them against the storage layer, and streams results back to
clients. This document provides a deep technical overview for developers.

---

# 1. Query Engine Architecture

```
   Client Request
        |
        v
 +-----------------+
 |   ReQL Parser   |
 |   (AST Layer)   |
 +--------+--------+
          |
          v
 +-----------------+
 |  Query Compiler |
 |   (Validation   |
 |    + Planning)  |
 +--------+--------+
          |
          v
 +-----------------+
 | Query Executor  |
 | (Runtime, IO,   |
 |  Iteration)     |
 +--------+--------+
          |
          v
 +-----------------+
 | Storage Engine  |
 +-----------------+
```

The engine is divided into three layers:

1. **AST** (abstract syntax tree): represents ReQL terms and structures  
2. **Compiler**: validates and converts AST into execution plans  
3. **Executor**: performs iteration, storage access, and result streaming  

---

# 2. AST Layer (`src/reql/`)

The AST (implemented in `ast.rs`, `terms.rs`, `types.rs`) represents:

- Operations (“terms”)
- Function call chains
- Constants and datums
- Query options and parameters

Example AST snippet:

```
Term::Filter(
    Table("users"),
    Predicate(…)
)
```

AST nodes are:

- Strongly typed
- Recursively structured
- Prepared for transformation during compilation

---

# 3. Query Compiler (`src/query/compiler.rs`)

The compiler receives an AST and outputs a **QueryPlan**, a low-level
representation optimized for the executor.

### Responsibilities:

- Validate terms and argument counts  
- Resolve references (db/table names → storage handles)  
- Infer return types  
- Rewrite or optimize certain expressions  
- Generate a stepwise plan (scan → filter → projection → sort, etc.)

The compiler is the main gateway between high-level ReQL and low-level logic.

---

## 3.1 QueryPlan Structure

A `QueryPlan` typically includes:

- **Plan Steps** (`enum QueryStep`)
  - TableScan
  - GetByPrimaryKey
  - Filter
  - Map
  - Reduce
  - IndexScan (future)
  - Sort
  - Limit
  - Insert / Update / Delete

- **Execution Context**
  - table ID
  - index metadata
  - storage handles
  - static expressions

- **Evaluators**
  - closures or trait objects compiled from AST predicates

---

## 3.2 Optimization Passes

PhotonDB implements light-weight optimizations:

- Constant folding  
- Shortcut elimination (e.g., filter(true) → no-op)  
- Pushdown optimizations (future: filter → index scan)  
- Key-range narrowing  

Planned:

- Query rewriting  
- Cost-based selection of indexes  
- Vectorized execution for certain workloads  

---

# 4. Query Executor (`src/query/executor.rs`)

The executor consumes the `QueryPlan` and performs the actual work.

Responsibilities:

- Evaluate plan steps in order  
- Interact with storage engine (`engine.rs` trait)  
- Manage iterators and streaming  
- Handle errors and type coercions  
- Produce result datums (`reql/datum.rs`)  
- Apply backpressure (WebSockets / streaming queries)

---

# 5. Execution Pipeline

Example pipeline for:

```
r.table("users").filter({active: true}).pluck("name")
```

Steps:

1. **Compiler**:
   - Identify the table
   - Build TableScan → Filter → Projection steps

2. **Executor**:
   - Open table iterator  
   - For each row:
     - Test predicate `(active == true)`
     - Extract field `"name"`
   - Stream result chunk-by-chunk

3. **Network layer**:
   - Serialize results and send to client  

---

# 6. Iteration Model

PhotonDB uses a pull-based iterator model:

```
Executor → Iterator.next() → Storage Layer → Next Row
```

This model enables:

- Backpressure handling  
- Streaming large datasets  
- Lazy evaluation  
- Efficient early termination (limit, existence checks)  

Future: vectorized batch iteration for analytic workloads.

---

# 7. Predicates & Expression Evaluation

Expressions are compiled into evaluators which may:

- directly read fields
- apply comparison operators
- evaluate logical expressions recursively
- compute derived values

Expressions are represented by lightweight enums and evaluated quickly.

Future improvements:

- JIT compiled evaluators  
- SIMD-aware execution  
- Expression caching  

---

# 8. Write Operations

Writes follow a structured process:

### 8.1 Insert

- Validate primary key  
- Encode row  
- Use storage engine for write  
- Return inserted keys/row count  

### 8.2 Update

- Evaluate update expression  
- Write changed fields  
- Support partial updates  

### 8.3 Delete

- Identify keys via scan or direct lookup  
- Delete via storage engine  

All writes pass through WAL for durability.

---

# 9. Transactions (Future)

Planned features:

- MVCC-based isolation
- Multi-operation transactions
- Savepoints
- Serializable isolation levels
- Distributed transactions (cluster mode)

---

# 10. Error Handling

Query-level errors include:

- Invalid term  
- Type mismatch  
- Missing table / database  
- Invalid options  
- Storage errors (IO, WAL)  
- Serialization issues  

Errors unify into a `QueryError` type passed through the executor.

---

# 11. Index Usage (Future)

PhotonDB will support:

- Primary indexes (already present via B-Tree)
- Secondary indexes (B-Tree variants)
- Full-text plugins
- Vector search indexes (ANN)

Compiler → Executor → Storage will jointly coordinate index usage.

---

# 12. Performance Considerations

The query engine is designed for:

- Minimal allocations  
- Direct value references  
- Predictable iteration  
- Lock minimization  
- Hot-path optimization  

Future optimizations:

- SIMD-accelerated filters  
- Cache-friendly operator fusion  
- Vectorized execution batches  

---

# 13. Summary

The PhotonDB Query Engine:

- Converts ReQL AST → optimized plan  
- Executes the plan using iterators and storage engine  
- Streams results efficiently  
- Provides a modular design for future optimization, indexing, and analytics  

It is purpose-built for real-time, scientific, and vector-based workloads.

