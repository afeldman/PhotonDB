# RethinkDB Cap'n Proto Schemas

This directory contains the Cap'n Proto schema definitions for the RethinkDB wire protocol.

## Schema Files

The protocol has been modularized into separate schema files for better organization:

### Core Types (`types.capnp`)

- `Datum`: RethinkDB's JSON-like data type (null, bool, number, string, array, object)
- `AssocPair`: Key-value pairs for objects
- `Frame`: Backtrace frame for error reporting
- `Backtrace`: Stack trace information

### Handshake Protocol (`handshake.capnp`)

- `VersionDummy`: Protocol version information
- Version constants (V0_1 through V1_0)
- Protocol type (Protobuf/JSON)
- Magic number constants for connection handshake

### Term Operations (`term.capnp`)

- `TermType`: Enum with 190 RethinkDB operations
  - Core: datum, makeArray, makeObj
  - Database ops: db, table, get, getAll
  - Comparisons: eq, ne, lt, le, gt, ge
  - Arithmetic: add, sub, mul, div, mod, floor, ceil, round
  - Array ops: append, prepend, slice, skip, limit
  - Object ops: getField, keys, values, pluck, without, merge
  - Transformations: map, filter, reduce, orderBy, distinct
  - Joins: innerJoin, outerJoin, eqJoin
  - Admin: dbCreate, tableCreate, indexCreate
  - Vector ops: indexCreateVector, getNearestVector (AI/ML)
- `Term`: Represents a query operation with args and optargs

### Query Protocol (`query.capnp`)

- `QueryType`: START, CONTINUE, STOP, NOREPLY_WAIT, SERVER_INFO
- `Query`: Client request message with token, query term, and options

### Response Protocol (`response.capnp`)

- `ResponseType`: Success types (ATOM, SEQUENCE, PARTIAL) and error types
- `ErrorType`: INTERNAL, RESOURCE_LIMIT, QUERY_LOGIC, etc.
- `ResponseNote`: Changefeed stream metadata
- `Response`: Server response message with data, backtrace, profile

## Protocol Flow

1. **Handshake**:

   - Client sends 32-bit version magic number (little-endian)
   - Client sends authorization key (length + string)
   - Client sends 32-bit protocol magic number
   - Server responds with "SUCCESS\0" or error message

2. **Query/Response**:
   - Client constructs Query with Term and token
   - Client sends: 4-byte size + Query blob
   - Server sends: 4-byte size + Response blob
   - For partial results, client sends CONTINUE with same token

## Advantages of Cap'n Proto

- **Zero-copy**: No encoding/decoding overhead
- **Fast**: Direct memory access to serialized data
- **Type-safe**: Strong typing with code generation
- **Compact**: Efficient wire format
- **Evolving**: Schema evolution support

## Usage in Rust

```rust
use rethinkdb::{query_capnp, response_capnp, types_capnp};

// Create a query
let mut message = capnp::message::Builder::new_default();
let mut query = message.init_root::<query_capnp::query::Builder>();
query.set_type(query_capnp::QueryType::Start);
query.set_token(1);

// Parse a response
let response = message.get_root::<response_capnp::response::Reader>()?;
let data = response.get_response()?;
```

## Migration from Protobuf

Cap'n Proto was chosen over Protobuf for:

- Better performance (zero-copy)
- Simpler API
- Modern design
- Better Rust integration

The protocol maintains compatibility with RethinkDB's wire format through careful schema design.
