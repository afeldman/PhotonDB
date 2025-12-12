# PhotonDB RPC Protocol Design

PhotonDB supports multiple protocols for client communication and internal
coordination. The design balances **compatibility** (ReQL/JSON), **simplicity**
(HTTP/WS), and **performance** (Cap’n Proto RPC).

This document describes the RPC protocol layers, message formats, and roles.

---

# 1. Protocol Stack Overview

PhotonDB’s protocol stack is layered:

```
+-------------------------------+
|   Client Libraries / SDKs     |
+-------------------------------+
|   ReQL JSON  |  Cap'n Proto  |
+-------------------------------+
| HTTP / TCP / QUIC Transport  |
+-------------------------------+
|        OS / Network           |
+-------------------------------+
```

Supported / planned protocols:

- **ReQL JSON over TCP** – compatibility with existing RethinkDB-style clients
- **HTTP/REST + WebSocket** – for admin, health, metrics, and changefeeds
- **Cap’n Proto RPC over TCP/QUIC** – high-performance RPC for PhotonDB clients
- **Internal cluster RPC** – node-to-node communication (Cap’n Proto)

---

# 2. ReQL JSON Protocol (Compatibility Layer)

PhotonDB supports a ReQL-compatible JSON protocol for:

- Basic CRUD operations
- Simple queries
- Migration from existing RethinkDB environments

### 2.1 Connection Model

- Transport: TCP (optionally TLS)
- Framing: length-prefixed JSON frames

```
[u32 length (big-endian)][UTF-8 JSON payload]
```

### 2.2 Message Structure

Requests:

```json
{
  "token": 1,
  "term": {
    "type": "insert",
    "table": "users",
    "doc": { "id": 123, "name": "Anton" }
  },
  "global_optargs": {}
}
```

Responses:

```json
{
  "token": 1,
  "result": {
    "type": "ack"
  }
}
```

or

```json
{
  "token": 2,
  "result": {
    "type": "single",
    "value": { "id": 123, "name": "Anton" }
  }
}
```

### 2.3 Usage

- Suitable for CLI tools & debugging
- Easy to integrate from any language
- Human-readable messages

---

# 3. HTTP & WebSocket Protocol

HTTP/REST endpoints are used for:

- Admin UI
- Health/readiness checks
- Metrics (Prometheus)
- Static content (dashboard)
- Management APIs (future)

WebSockets are used for:

- Live changefeeds
- Streaming query results
- Subscription-based metrics

### 3.1 Health Endpoints

Examples:

- `/health/live`
- `/health/ready`
- `/health/startup`

### 3.2 Metrics

- `/metrics` (Prometheus scrape endpoint)

### 3.3 Admin UI

- `/_admin` – static dashboard served from `static/admin.html`

---

# 4. Cap’n Proto RPC (PhotonDB Native Protocol)

For high-performance and type-safe communication, PhotonDB will provide a
Cap’n Proto RPC interface.

Goals:

- Low-latency, binary protocol
- Strict typing and schema evolution
- Efficient internal and external RPC
- Code generation for multiple languages (Rust, Go, Python, etc.)

---

## 4.1 Cap’n Proto Schemas (`proto/`)

Core schemas:

- `handshake.capnp`
- `query.capnp`
- `response.capnp`
- `types.capnp`

Example high-level interface (conceptual):

```capnp
@0xf0f0f0f0f0f0f0f0;

interface PhotonDb {
  query @0 (request :QueryRequest) -> (response :QueryResponse);
  streamQuery @1 (request :QueryRequest) -> (stream :QueryStream);
  health @2 () -> (status :HealthStatus);
}
```

`QueryRequest` would include:

- database name
- table name
- compiled ReQL or internal query representation
- options (timeout, consistency, profile flags)

---

## 4.2 Transport

Cap’n Proto RPC can run over:

- TCP + TLS
- QUIC + TLS 1.3

Capabilities:

- Bi-directional RPC
- Streaming responses
- Pipeline-friendly messaging

---

## 4.3 Message Flow

1. **Handshake**

   - Client connects and sends a `Handshake` message.
   - Server replies with protocol version and capabilities.
   - Optional authentication: tokens, mTLS, etc.

2. **RPC Calls**

   - Client calls `PhotonDb.query()` or `PhotonDb.streamQuery()`.
   - Messages are encoded using Cap’n Proto binary format.
   - Server decodes, routes to query engine, and replies.

3. **Streaming**

   - `streamQuery()` returns a capability representing a stream.
   - Client pulls batches of results or receives pushed chunks.

---

# 5. Internal Cluster RPC

PhotonDB uses an internal Cap’n Proto–based protocol for:

- Cluster gossip (future)
- Metadata sync
- Replication control
- Health & metrics exchange
- Admin commands

Characteristics:

- mTLS required
- Node roles and IDs embedded
- Signature validation (future)
- Version compatibility checks

---

# 6. Versioning & Compatibility

PhotonDB uses a careful versioning strategy:

- Each Cap’n Proto schema is versioned via field IDs and reserved IDs.
- Deprecated fields are kept but not used, maintaining backward compatibility.
- The handshake reports:
  - Protocol version
  - Feature flags
  - Supported capabilities

Clients can:

- Negotiate features
- Fall back to JSON/HTTP if needed
- Detect incompatible servers

---

# 7. Error Handling

Errors in RPC are categorized as:

- **Protocol errors** – invalid frames, malformed messages
- **Authentication errors** – invalid or expired credentials
- **Authorization errors** – insufficient permissions
- **Query errors** – invalid ReQL, type mismatches, runtime errors
- **Server errors** – storage failures, internal panics, timeouts

Cap’n Proto responses will include:

```capnp
struct RpcError {
  code @0 :UInt32;
  message @1 :Text;
  details @2 :Text;
}
```

JSON/ReQL responses include an `error` result type.

---

# 8. Security Considerations

See `SECURITY_MODEL.md` for full details.

Protocol-specific highlights:

- Require TLS for all external Cap’n Proto RPC
- Protect ReQL/TCP via TLS where possible
- Authenticate before accepting RPC calls
- Implement rate limiting for expensive queries
- Log security-relevant events and anomalies

---

# 9. Client SDKs

Planned SDK targets:

- Rust
- Go
- Python
- Node.js / TypeScript

SDKs provide:

- Connection pooling
- Retry logic
- Typed request/response APIs
- High-level abstractions for queries and streaming

---

# 10. Migration & Coexistence

PhotonDB supports **coexistence of protocols**:

- Legacy / debug: ReQL over JSON/TCP
- Primary user-facing API: HTTP + WebSockets
- High-performance & internal: Cap’n Proto RPC

This allows gradual migration and heterogeneous clients.

---

# 11. Future Extensions

Potential protocol extensions:

- GraphQL endpoint on top of query engine
- gRPC gateway to Cap’n Proto RPC
- HTTP/3-native connections (QUIC)
- Cross-cluster replication protocol
- Changefeed subscriptions over Cap’n Proto

---

# 12. Summary

PhotonDB’s RPC protocol design:

- Provides **compatibility** via ReQL JSON
- Offers **usability** via HTTP/REST + WebSockets
- Delivers **performance & safety** via Cap’n Proto RPC

This layered approach supports both current use cases and future advanced
workloads across standalone and clustered deployments.
