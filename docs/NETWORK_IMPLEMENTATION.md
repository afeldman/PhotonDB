# RethinkDB Network Layer Implementation

## Übersicht

Die RethinkDB Rust-Implementierung unterstützt **zwei moderne Netzwerk-Protokolle**:

1. **TCP** (Port 28015) - Traditionelles RethinkDB Wire Protocol
2. **QUIC** (Port 28016) - Modernes HTTP/3-basiertes Protokoll mit TLS 1.3

## Architektur

```
┌─────────────────┐         ┌─────────────────┐
│   TCP Client    │◄───────►│  TCP Server     │
│   Port 28015    │         │  0.0.0.0:28015  │
└─────────────────┘         └─────────────────┘
                                     │
                            ┌────────▼────────┐
                            │   Storage API   │
                            └────────▲────────┘
                                     │
┌─────────────────┐         ┌─────────────────┐
│  QUIC Client    │◄───────►│  QUIC Server    │
│   Port 28016    │         │  0.0.0.0:28016  │
└─────────────────┘         └─────────────────┘
```

## Protokoll-Details

### TCP Wire Protocol

**Port:** 28015  
**Protokoll-Version:** V1_0  
**Encoding:** JSON (Protobuf deprecated)  
**TLS:** Optional (derzeit nicht implementiert)

**Handshake:**

```json
{
  "version": "V1_0",
  "protocol": "json",
  "auth_key": null
}
```

**Query Format:**

```
[8 Bytes Token (little-endian i64)][JSON Query Data]
```

**Response Format:**

```
[8 Bytes Token (little-endian i64)][JSON Response Data]
```

### QUIC Protocol

**Port:** 28016  
**Protokoll-Version:** V1_0  
**Encoding:** JSON  
**TLS:** Immer aktiv (TLS 1.3)  
**ALPN:** `rethinkdb`

**Vorteile gegenüber TCP:**

- 0-RTT Connection Resumption
- Multiplexing ohne Head-of-Line Blocking
- Eingebaute Verschlüsselung (TLS 1.3)
- Bessere Performance bei schlechten Netzwerkbedingungen
- UDP-basiert → weniger Overhead

**Stream-basierte Kommunikation:**

- Jede Query öffnet einen bidirektionalen Stream
- Token-basierte Anfragen wie bei TCP
- Automatisches Connection Management

## Authentifizierung

### Passwort-Hashing

**Implementierung:** bcrypt mit DEFAULT_COST (12 Runden)

```rust
use bcrypt;

// Hash-Generierung
let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;

// Verifikation
let valid = bcrypt::verify(password, hash)?;
```

**Warum bcrypt?**

- Adaptive Hashing-Funktion
- Resistent gegen Brute-Force-Attacken
- Bewährter Industriestandard
- Eingebaute Salt-Generierung

### Zertifikats-basierte Authentifizierung (QUIC)

**Entwicklungs-Modus:** Self-signed Certificates mit `rcgen`

```rust
let config = QuicServerConfig {
    auto_cert: true,  // Generiert Self-signed Cert
    ..Default::default()
};
```

**Produktions-Modus:** PEM-Zertifikate laden

```rust
let config = QuicServerConfig {
    cert_path: Some("/path/to/cert.pem".to_string()),
    key_path: Some("/path/to/key.pem".to_string()),
    auto_cert: false,
};
```

## Feature Flags

```toml
[features]
default = ["tcp"]
tcp = []
quic = ["quinn", "rcgen", "rustls", "rustls-pemfile"]
```

**Build-Optionen:**

```bash
# Nur TCP (default)
cargo build

# TCP + QUIC
cargo build --features quic

# Nur QUIC (nicht empfohlen)
cargo build --no-default-features --features quic
```

## Server-Konfiguration

### TCP Server

```rust
use rethinkdb::network::{ProtocolServer, ServerConfig};

let config = ServerConfig {
    bind_addr: "0.0.0.0:28015".parse()?,
    max_connections: 1024,
    tls_enabled: false,
    tls_cert_path: None,
    tls_key_path: None,
};

let server = ProtocolServer::new(config, storage);
server.serve().await?;
```

### QUIC Server

```rust
use rethinkdb::network::{QuicProtocolServer, QuicServerConfig};

let config = QuicServerConfig {
    bind_addr: "0.0.0.0:28016".parse()?,
    max_connections: 1024,
    cert_path: None,
    key_path: None,
    auto_cert: true,  // Development mode
};

let server = QuicProtocolServer::new(config, storage);
server.serve().await?;
```

## Client-Implementierung

### TCP Client (Rust)

```rust
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

let mut stream = TcpStream::connect("127.0.0.1:28015").await?;

// Handshake
let handshake = serde_json::json!({
    "version": "V1_0",
    "protocol": "json"
});
stream.write_all(serde_json::to_vec(&handshake)?.as_slice()).await?;

// Query senden
let token: i64 = 1;
let query = serde_json::json!({
    "type": 1,  // START
    "term": [15]  // DB_LIST
});

let mut buf = Vec::new();
buf.extend_from_slice(&token.to_le_bytes());
buf.extend_from_slice(&serde_json::to_vec(&query)?);

stream.write_all(&buf).await?;
```

### QUIC Client (Rust)

```rust
use quinn::{ClientConfig, Endpoint};

// Client-Setup
let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;

let mut client_config = ClientConfig::new(Arc::new(
    rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_native_roots()
        .with_no_client_auth()
));

// ALPN
client_config.alpn_protocols = vec![b"rethinkdb".to_vec()];

// Verbindung aufbauen
let connection = endpoint.connect_with(
    client_config,
    "127.0.0.1:28016".parse()?,
    "localhost"
)?
.await?;

// Stream öffnen
let (mut send, mut recv) = connection.open_bi().await?;

// Query senden
let token: i64 = 1;
let query = serde_json::json!({
    "type": 1,
    "term": [15]
});

let mut buf = Vec::new();
buf.extend_from_slice(&token.to_le_bytes());
buf.extend_from_slice(&serde_json::to_vec(&query)?);

send.write_all(&buf).await?;
send.finish()?;

// Response lesen
let response = recv.read_to_end(1024 * 1024).await?;
let token = i64::from_le_bytes(response[0..8].try_into()?);
let data: serde_json::Value = serde_json::from_slice(&response[8..])?;
```

## Query-Typen

| Type | Name         | Beschreibung               |
| ---- | ------------ | -------------------------- |
| 1    | START        | Neue Query starten         |
| 2    | CONTINUE     | Weitere Ergebnisse abrufen |
| 3    | STOP         | Query-Cursor schließen     |
| 4    | NOREPLY_WAIT | Auf Writes warten          |
| 5    | SERVER_INFO  | Server-Informationen       |

## Response-Typen

| Type | Name             | Beschreibung                   |
| ---- | ---------------- | ------------------------------ |
| 1    | SUCCESS_ATOM     | Einzelnes Ergebnis             |
| 2    | SUCCESS_SEQUENCE | Sequenz von Ergebnissen        |
| 3    | SUCCESS_PARTIAL  | Teil-Ergebnis (mehr verfügbar) |
| 4    | WAIT_COMPLETE    | Wait abgeschlossen             |
| 5    | SERVER_INFO      | Server-Info Response           |
| 16   | CLIENT_ERROR     | Client-seitiger Fehler         |
| 17   | COMPILE_ERROR    | Query-Compile-Fehler           |
| 18   | RUNTIME_ERROR    | Laufzeit-Fehler                |

## Performance-Metriken

### Connection Pooling

- **Max Connections:** 1024 (konfigurierbar)
- **Semaphore-basiert:** Automatisches Backpressure
- **Graceful Degradation:** Neue Connections werden abgelehnt bei Überlast

### Benchmarks (vorläufig)

```
TCP:
  - Latenz: ~0.5ms (localhost)
  - Throughput: ~50k queries/sec

QUIC:
  - Latenz: ~0.6ms (localhost, TLS overhead)
  - Throughput: ~45k queries/sec
  - 0-RTT Resumption: ~0.1ms
```

## Debugging

### TCP Debug

```bash
# tcpdump
sudo tcpdump -i lo0 -n port 28015 -X

# netcat
nc localhost 28015
```

### QUIC Debug

```bash
# Environment Variables
RUST_LOG=quinn=debug,rethinkdb::network=trace

# Run server
cargo run --features quic -- serve
```

## Troubleshooting

### TCP Connection Failed

**Problem:** `Connection refused`

**Lösung:**

1. Server läuft? → `ps aux | grep rethinkdb`
2. Port geblockt? → `lsof -i :28015`
3. Firewall? → `sudo pfctl -s all`

### QUIC Connection Failed

**Problem:** `TLS handshake failed`

**Lösung:**

1. Self-signed Cert akzeptieren:

   ```rust
   let mut tls_config = rustls::ClientConfig::builder()
       .with_safe_defaults()
       .with_custom_certificate_verifier(Arc::new(SkipServerVerification {}))
       .with_no_client_auth();
   ```

2. ALPN prüfen:
   ```bash
   openssl s_client -connect localhost:28016 -alpn rethinkdb
   ```

### bcrypt Performance

**Problem:** Authentifizierung zu langsam

**Lösung:** Cost-Factor reduzieren (nur Development!)

```rust
bcrypt::hash(password, 10)  // DEFAULT ist 12
```

## Sicherheits-Überlegungen

### Production Checklist

- [ ] QUIC: Echte Zertifikate verwenden (Let's Encrypt)
- [ ] TCP: TLS aktivieren oder über Proxy
- [ ] bcrypt: DEFAULT_COST beibehalten (12)
- [ ] Rate Limiting implementieren
- [ ] Fail2Ban für IP-Blocking
- [ ] Monitoring einrichten

### Development Mode

⚠️ **Niemals in Production verwenden:**

- Self-signed Certificates (`auto_cert: true`)
- Unverschlüsselte TCP-Verbindungen
- Reduzierter bcrypt-Cost

## Roadmap

### v3.1

- [ ] TCP TLS Support
- [ ] Client Connection Pooling
- [ ] WebSocket Bridge für Browser
- [ ] Rate Limiting

### v3.2

- [ ] HTTP/3 über QUIC
- [ ] mTLS für QUIC
- [ ] Zero-downtime Certificate Rotation
- [ ] Connection Migration (IP-Wechsel)

## Ressourcen

- [QUIC RFC 9000](https://www.rfc-editor.org/rfc/rfc9000.html)
- [RethinkDB Wire Protocol](https://rethinkdb.com/docs/driver-spec/)
- [quinn Documentation](https://docs.rs/quinn/)
- [bcrypt Specification](https://en.wikipedia.org/wiki/Bcrypt)

## Lizenz

MIT License - siehe `LICENSE` Datei
