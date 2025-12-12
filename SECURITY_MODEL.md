# PhotonDB Security Model

PhotonDB is designed with a multilayered, defense‑in‑depth security architecture.
This document describes the security principles, components, and mechanisms that
protect PhotonDB in standalone and clustered deployments.

---

# 1. Security Philosophy

PhotonDB follows these principles:

- **Zero Trust by Default**  
  No component trusts external input, clients, or nodes without verification.

- **Least Privilege**  
  Every subsystem, handler, and cluster node receives only the permissions
  required for its function.

- **Secure by Design**  
  Authentication, authorization, input validation, and encryption are mandatory.

- **Defense in Depth**  
  Server, network, storage, plugins, and cluster layers each enforce protection.

- **Auditability & Observability**  
  Security‑related events must be logged and observable.

---

# 2. Threat Model Overview

PhotonDB defends against:

### ✔ External attackers
- Unauthorized API access  
- Credential theft  
- Injection attacks  
- Replay attacks  
- Passive traffic sniffing  
- Active man‑in‑the‑middle

### ✔ Malicious or compromised nodes
- Fake cluster nodes  
- Unauthorized replication / data modification  
- Poisoned metadata  
- Invalid WAL / page data

### ✔ Internal misconfigurations
- Over‑permissive access  
- Unsafe defaults  
- Plugin vulnerabilities  

### ✖ Out of scope (for now)
- Side‑channel attacks  
- Hardware compromise  
- Kernel‑level intrusion  
- Untrusted user‑provided code execution inside the server  

---

# 3. Security Architecture Layers

```
        +-----------------------------+
        |      HTTP / WS / ReQL      |
        +--------------+--------------+
                       |
                       v
        +-----------------------------+
        |  Authentication & Identity  |
        +--------------+--------------+
                       |
                       v
        +-----------------------------+
        |      Authorization Layer    |
        +--------------+--------------+
                       |
                       v
        +-----------------------------+
        |   Network Transport (TLS)   |
        +--------------+--------------+
                       |
                       v
        +-----------------------------+
        |   Storage Integrity / WAL   |
        +--------------+--------------+
                       |
                       v
        +-----------------------------+
        |    Cluster Trust / Gossip   |
        +-----------------------------+
```

---

# 4. Authentication

PhotonDB supports layered authentication mechanisms.

## 4.1 HTTP / WebSocket Auth

### Supported:
- Static API keys
- JWT / OAuth2 bearer tokens (future)
- Session‑bound tokens (admin UI)

### Characteristics:
- Authorization header:  
  ```
  Authorization: Bearer <token>
  ```
- API keys hashed and stored in configuration or secret provider.

---

## 4.2 ReQL / TCP Authentication

ReQL connections must perform an authentication handshake:

1. Client initiates handshake
2. Server sends challenge
3. Client responds with signed/authenticated message
4. Server verifies and grants session permissions

Password‑based or HMAC-based challenge–response.

---

## 4.3 Cluster Node Authentication

Cluster nodes *never* trust each other implicitly.

PhotonDB uses:

- Node identity certificates (X.509 or Ed25519 keys)
- Mutual TLS (mTLS)
- Rotating cluster tokens
- Signed cluster configuration manifests

---

# 5. Authorization

PhotonDB will support role‑based and capability‑based authorization:

### Roles
- `admin`
- `cluster-admin`
- `db-owner`
- `reader`
- `writer`

### Permissions
- Table read/write
- DB create/delete
- Cluster join/remove
- Storage inspection
- Metrics/health read

Authorization occurs **after** authentication in:

- HTTP routers  
- WebSocket channels  
- TCP/ReQL commands  
- Cluster operations  

---

# 6. Transport Security

PhotonDB enforces:

- TLS for all external endpoints
- QUIC + TLS1.3 for internal data paths
- Optional encryption offload for performance nodes

Supported algorithms:

- AES-GCM
- ChaCha20-Poly1305
- Ed25519 signatures
- SHA-256/512 HMAC

Cipher configuration is managed via:

```
PHOTONDB_TLS_CERT
PHOTONDB_TLS_KEY
PHOTONDB_TLS_DISABLE_VERIFY (dev only)
```

---

# 7. Storage Security

PhotonDB’s storage layer enforces:

### 7.1 WAL Integrity
- Checksummed WAL segments
- Replay validation
- Corruption detection and recovery

### 7.2 Page Integrity
- Page‑level checksums
- Prevent partial writes
- Detect torn writes

### 7.3 Optional Encryption at Rest (future)
- Page‑level or file‑level AES‑GCM
- Key rotation via KMS (Vault, AWS KMS…)

---

# 8. Plugin Security Model

Plugins run in‑process but must:

- Register capabilities explicitly
- Declare version and compatibility metadata
- Be loaded only from trusted paths
- Pass integrity verification (checksum/signature)

A compromised plugin must not be able to:

- Access raw storage
- Intercept network traffic
- Modify cluster configuration  
(unless granted explicit rights)

Plugins run behind traits enforced by the host process.

---

# 9. Cluster Security

PhotonDB clusters rely on:

### 9.1 Gossip Trust Boundary
Nodes verify:

- Node identity
- Protocol version
- Shared cluster secret
- Allowed node roles

### 9.2 Secure Bootstrap
A node can join only via:

- mTLS verification  
- Signed cluster manifests  
- Admin approval mechanisms  

### 9.3 Anti‑Sybil Protections (future)
Distributed identity verification.

---

# 10. Auditing & Observability

PhotonDB logs security‑critical events:

- Authentication success/failure
- Role escalation
- Cluster join/leave attempts
- Storage corruption events
- WAL recovery operations
- Suspicious network behavior

Debugging/tracing:

```
RUST_LOG=photondb_security=trace
```

---

# 11. Development Modes

PhotonDB includes dev‑mode options:

```
PHOTONDB_DEV_MODE=1
PHOTONDB_DISABLE_TLS=1
PHOTONDB_LOGGING=debug
```

Dev mode:
- Simplifies TLS
- Disables strict authentication
- Enables introspection endpoints

Never use in production.

---

# 12. Roadmap for Security Enhancements

- Cap’n Proto RPC authentication
- Capability system for plugins
- Signed storage pages
- Encrypted WAL segments
- Cross-node tamper detection
- Full role-based access control (RBAC)
- Query-level permissions
- Integration with Vault / AWS KMS

---

# 13. Summary

PhotonDB provides a hardened security architecture with layered defenses:
network, storage, cluster, and plugin isolation.  
Security is a first‑class design goal and continuously evolving.

