# Security Documentation

## Overview

PhotonDB implements enterprise-grade security with multiple layers of protection:

- **OAuth2 Multi-Provider Authentication**
- **Honeytrap Integration** for automatic threat blocking
- **Rate Limiting** and IP blacklisting
- **JWT Token Validation**
- **Attack Pattern Detection**
- **Security Audit Logging**

## Security Architecture

```
┌─────────────────────────────────────────────┐
│            Client Request                    │
└─────────────────┬───────────────────────────┘
                  ↓
┌─────────────────────────────────────────────┐
│  1. IP Blacklist Check                      │
│     Blocked? → 403 Forbidden                │
└─────────────────┬───────────────────────────┘
                  ↓
┌─────────────────────────────────────────────┐
│  2. Rate Limiting                           │
│     Exceeded? → Block IP + Report           │
└─────────────────┬───────────────────────────┘
                  ↓
┌─────────────────────────────────────────────┐
│  3. JWT Authentication                      │
│     Public endpoint? → Skip                 │
│     Invalid token? → 401 Unauthorized       │
└─────────────────┬───────────────────────────┘
                  ↓
┌─────────────────────────────────────────────┐
│  4. Attack Pattern Detection                │
│     SQL injection, XSS, etc? → Block        │
└─────────────────┬───────────────────────────┘
                  ↓
┌─────────────────────────────────────────────┐
│  5. Honeytrap Reporting                     │
│     Report all suspicious activity          │
└─────────────────┬───────────────────────────┘
                  ↓
┌─────────────────────────────────────────────┐
│         Request Allowed                     │
└─────────────────────────────────────────────┘
```

## Quick Start

### Development Mode (No Security)

```bash
# Disable all security for local development
export DEV_MODE=true
cargo run --bin rethinkdb
```

### Production Mode (Full Security)

```bash
# Enable all security features
export DEV_MODE=false

# Configure OAuth2 providers (see oauth2.md)
export GITHUB_CLIENT_ID=your_client_id
export GITHUB_CLIENT_SECRET=your_secret

# Set JWT secret (MUST be changed!)
export JWT_SECRET=your_super_secure_secret_key

# Configure Honeytrap
export HONEYTRAP_URL=http://honeytrap-server:8888

cargo run --bin rethinkdb --release
```

## Security Features

### 1. OAuth2 Authentication

Supports multiple providers:

- **GitHub** - Developer authentication
- **Google** - Enterprise SSO
- **AWS Cognito** - Cloud integration
- **Amazon AD** - Active Directory

See [oauth2.md](oauth2.md) for detailed setup.

### 2. Honeytrap Integration

Automatic threat detection and blocking using [github.com/afeldman/honeytrap](https://github.com/afeldman/honeytrap).

Features:

- Real-time IP blocking
- Threat intelligence sharing
- Automatic report generation
- Pattern-based detection

See [honeytrap.md](honeytrap.md) for integration guide.

### 3. Rate Limiting

Default: **100 requests per minute** per IP address

```rust
let config = SecurityConfig {
    max_requests_per_minute: 100,
    ..Default::default()
};
```

Exceeded? → IP blocked + Reported to Honeytrap

See [rate-limiting.md](rate-limiting.md) for configuration.

### 4. JWT Authentication

All protected endpoints require valid JWT token:

```bash
# Login to get token
curl -X POST http://localhost:8080/auth/login \
  -d '{"username":"admin","password":"secret"}'

# Response:
{
  "token": "eyJhbGc...",
  "expires_in": 3600
}

# Use token for API calls
curl -H "Authorization: Bearer eyJhbGc..." \
  http://localhost:8080/api/query
```

See [jwt.md](jwt.md) for token management.

### 5. Attack Pattern Detection

Automatically detects and blocks:

**SQL Injection:**

```
' OR '1'='1
DROP TABLE users
UNION SELECT
```

**XSS (Cross-Site Scripting):**

```
<script>alert(1)</script>
javascript:void(0)
```

**Path Traversal:**

```
../../../etc/passwd
..\\..\\..\\windows\\system32
```

**Command Injection:**

```
; rm -rf /
| cat /etc/passwd
```

## Public vs Protected Endpoints

### Public (No authentication required)

```
/_health     - Health check
/_ready      - Readiness check
/_metrics    - Prometheus metrics
/auth/*      - OAuth2 callbacks
```

### Protected (JWT token required)

```
/api/query         - Execute ReQL queries
/api/tables        - Table management
/api/tables/:name  - Table operations
/_admin            - Admin dashboard
```

## Security Best Practices

### 1. Change Default Secrets

```bash
# NEVER use default JWT secret in production!
export JWT_SECRET=$(openssl rand -base64 32)
```

### 2. Enable HTTPS/TLS

```rust
let config = ServerConfig {
    enable_tls: true,
    cert_path: "/path/to/cert.pem",
    key_path: "/path/to/key.pem",
    ..Default::default()
};
```

### 3. Configure Firewall

```bash
# Allow only necessary ports
sudo ufw allow 8080/tcp  # RethinkDB HTTP
sudo ufw allow 8888/tcp  # Honeytrap
sudo ufw enable
```

### 4. Enable Audit Logging

All security events are logged:

```
2025-12-09T01:30:00Z WARN rethinkdb::security: Rate limit exceeded ip=192.168.1.100
2025-12-09T01:30:01Z WARN rethinkdb::security: IP blocked ip=192.168.1.100 reason=rate_limit
2025-12-09T01:30:02Z INFO rethinkdb::security: Reporting to Honeytrap ip=192.168.1.100
```

### 5. Regular Security Audits

```bash
# Check for blocked IPs
curl http://localhost:8080/_admin/security/blocked

# Review audit logs
tail -f logs/rethinkdb.log | grep WARN
```

## Configuration Reference

### SecurityConfig

```rust
pub struct SecurityConfig {
    /// Enable security middleware (false in dev mode)
    pub enabled: bool,

    /// Enable Honeytrap integration
    pub honeytrap_enabled: bool,

    /// Honeytrap server URL
    pub honeytrap_url: String,

    /// OAuth2 provider configurations
    pub oauth2_providers: Vec<OAuth2Provider>,

    /// JWT signing secret
    pub jwt_secret: String,

    /// Maximum requests per minute per IP
    pub max_requests_per_minute: u32,
}
```

### Environment Variables

| Variable               | Description      | Required | Default                 |
| ---------------------- | ---------------- | -------- | ----------------------- |
| `DEV_MODE`             | Disable security | No       | `false`                 |
| `JWT_SECRET`           | JWT signing key  | **Yes**  | N/A                     |
| `GITHUB_CLIENT_ID`     | GitHub OAuth2    | No       | -                       |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth2    | No       | -                       |
| `GOOGLE_CLIENT_ID`     | Google OAuth2    | No       | -                       |
| `GOOGLE_CLIENT_SECRET` | Google OAuth2    | No       | -                       |
| `AWS_CLIENT_ID`        | AWS Cognito      | No       | -                       |
| `AWS_CLIENT_SECRET`    | AWS Cognito      | No       | -                       |
| `HONEYTRAP_URL`        | Honeytrap server | No       | `http://localhost:8888` |

## Testing Security

### 1. Test Rate Limiting

```bash
# Send 150 requests (should block after 100)
for i in {1..150}; do
  curl http://localhost:8080/_health
done

# Check if IP is blocked
curl http://localhost:8080/_health
# → 403 Forbidden (if blocked)
```

### 2. Test Attack Detection

```bash
# SQL injection attempt
curl "http://localhost:8080/api?q=' OR '1'='1"
# → 403 Forbidden + IP blocked

# Check logs
tail logs/rethinkdb.log
# → WARN: Suspicious request detected
```

### 3. Test JWT Authentication

```bash
# Without token (should fail)
curl http://localhost:8080/api/query
# → 401 Unauthorized

# With invalid token (should fail)
curl -H "Authorization: Bearer invalid" \
  http://localhost:8080/api/query
# → 401 Unauthorized

# With valid token (should work)
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/query
# → 200 OK
```

## Troubleshooting

### "IP is blocked"

**Cause**: Rate limit exceeded or suspicious activity detected

**Solution**:

```bash
# Clear blocked IPs (requires admin access)
curl -X DELETE http://localhost:8080/_admin/security/blocked/192.168.1.100
```

### "JWT token invalid"

**Cause**: Expired token or wrong secret

**Solution**:

1. Get new token via `/auth/login`
2. Verify `JWT_SECRET` is correct
3. Check token expiration time

### "OAuth2 callback failed"

**Cause**: Misconfigured OAuth2 provider

**Solution**:

1. Verify `CLIENT_ID` and `CLIENT_SECRET`
2. Check redirect URI matches OAuth2 app settings
3. Review provider-specific setup in [oauth2.md](oauth2.md)

## Related Documentation

- [OAuth2 Setup](oauth2.md) - Multi-provider configuration
- [Honeytrap Integration](honeytrap.md) - Threat detection
- [JWT Authentication](jwt.md) - Token management
- [Rate Limiting](rate-limiting.md) - Request throttling

## Security Contact

Found a security vulnerability? Please report to: **security@rethinkdb.com**

---

**Security is not optional in production!** 🔒
