# PhotonDB Query Language Specification (ReQL-P)

PhotonDB extends the classic RethinkDB ReQL protocol with a modern, modular,
and AI‑ready query language called **ReQL‑P (Photon ReQL)**.  
This document fully specifies the syntax, semantics, term definitions,
execution rules, and language extensions unique to PhotonDB.

---

# 1. Design Goals

ReQL‑P is built to be:

- **Backward compatible** with RethinkDB ReQL  
- **Type‑safe** and consistent  
- **Composable** (functions are chains)  
- **Extensible** (vector search, time-series, plugins)  
- **Streaming-friendly** (pipelines, cursors)  
- **Deterministic** in local evaluation  
- **Distributed-aware** in cluster mode  

---

# 2. Core Structure of ReQL‑P Queries

A ReQL‑P query always follows the same shape:

```
r.<root>(...).<term>(...).<term>(...).run()
```

Examples:

```
r.table("users").filter({active: true}).pluck("name")
```

```
r.table("vectors").nearest(query_vec, {k: 20})
```

```
r.ts("metrics").between(start, end).avg("value")
```

---

# 3. Fundamental Terms

## 3.1 `db(name)`
Selects database.

## 3.2 `table(name)`
Selects table inside DB.

## 3.3 `get(key)`
Primary-key lookup.

## 3.4 `insert(object | [objects])`
Insert one or multiple documents.

## 3.5 `update(object | function)`
Partial or full update.

## 3.6 `delete()`
Delete by primary key or via filter.

## 3.7 `filter(predicate)`
Predicate must evaluate to boolean.

## 3.8 `map(function)`
Transform rows.

## 3.9 `reduce(function, base)`
Fold operation.

## 3.10 `pluck(fields...)`
Select subset of fields.

## 3.11 `without(fields...)`
Drop specified fields.

---

# 4. Control Flow Terms

## 4.1 `branch(condition, then, else)`
Equivalent to ternary:

```
branch(x > 10, "big", "small")
```

## 4.2 `default(value)`
Fallback for missing data.

## 4.3 `merge(object)`
Deep merge for objects.

---

# 5. Sorting, Limits & Pagination

## 5.1 `orderBy(field | index("name"))`
Sorts dataset.

## 5.2 `skip(n)`
Skip n rows.

## 5.3 `limit(n)`
Limit to n rows.

---

# 6. Aggregations

## 6.1 `count()`
Count number of results.

## 6.2 `avg(field)`
Average of numeric field.

## 6.3 `sum(field)`
Sum of numeric field.

## 6.4 `min(field)` / `max(field)`
Min / max values.

## 6.5 `group(field | function)`
Group results into buckets.

---

# 7. Document Manipulation

## 7.1 Field access

```
row("name")
```

## 7.2 Arithmetic

```
row("count") + 5
row("x") * row("y")
```

## 7.3 Logic

```
row("active") & (row("age") > 18)
```

---

# 8. ReQL‑P Type System

PhotonDB defines **Datum** types:

- `null`
- `bool`
- `number` (int, float)
- `string`
- `array`
- `object`
- `binary`
- `timestamp`
- `vector` (PhotonDB extension)

Vectors introduce:

```
[1.2, 0.3, 9.1] :: vector
```

---

# 9. PhotonDB Extensions

## 9.1 Vector Search

### Syntax:

```
nearest(query_vector, {index: "hnsw", k: 10})
```

### Options:

- `k`: number of neighbors
- `index`: vector index name
- `metric`: optional (l2, cosine, inner_product)

### Return format:

```
[
  {id: 123, score: 0.991, row: {...}},
  {id: 913, score: 0.987, row: {...}},
  ...
]
```

---

## 9.2 Time-Series Queries

PhotonDB adds special TS terms.

### 9.2.1 `ts(table)`
Access time-series table.

### 9.2.2 `between(start, end)`
Timestamp range.

### 9.2.3 `downsample("1m", {agg: "avg"})`
Downsampling window.

### 9.2.4 `rate(field)`
Compute derivative/delta per second.

### 9.2.5 `integral(field)`
Time integral.

---

## 9.3 Plugin Bound Functions

Plugins may register custom terms:

```
r.table("events").my_custom_term(args)
```

Registered via:

```rust
registry.register_query_function("my_custom_term", handler);
```

---

# 10. Execution Semantics

## 10.1 Out-of-order evaluation forbidden
Expressions must remain deterministic.

## 10.2 Lazy Evaluation
Filter/map/reduce operate streamingly.

## 10.3 Short-circuit boolean evaluation

```
x && y   → do not evaluate y if x is false
```

---

# 11. Distributed Semantics (Cluster Mode)

When PhotonDB is clustered:

### 11.1 Query routing
Coordinator distributes:

- range queries
- nearest neighbor searches
- time-series window scans

### 11.2 Aggregation pushdown
Aggregations executed locally where data resides.

### 11.3 Consistency Modes

```
"eventual" | "timeline" | "strong"
```

---

# 12. Error Semantics

Errors use standard format:

```
{
  "error": "TypeError",
  "msg": "Expected number, got array",
  "location": "term[3]"
}
```

---

# 13. Grammar (EBNF)

```
query        = root, { term }, "run" ;
root         = "r.table" | "r.db" | "r.ts" ;
term         = ".", identifier, "(", args, ")" ;
args         = [ expression, { ",", expression } ] ;
expression   = literal | object | array | function | identifier ;
literal      = string | number | boolean | null ;
```

The grammar is intentionally minimal; Atom-level semantics are in the AST.

---

# 14. Example Reference Queries

### 14.1 Standard document query

```
r.table("users")
 .filter({country: "DE"})
 .pluck("id", "name")
```

### 14.2 Vector search with metadata filter

```
r.table("items")
 .nearest(vec, {k: 5})
 .filter({category: "electronics"})
```

### 14.3 Time-series downsampling

```
r.ts("metrics")
 .between(ts - 86400, ts)
 .downsample("1m", {agg: "avg"})
```

---

# 15. Summary

ReQL‑P (Photon ReQL):

- Extends classic ReQL  
- Adds ANN vector search  
- Adds time-series operators  
- Supports plugin-defined language terms  
- Clean, modular, deterministic  
- Cluster-aware execution  

It is the foundation for PhotonDB’s modern AI + time-series + document workloads.
