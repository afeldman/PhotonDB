# HTTP API Reference

## Base URL

```
http://localhost:8080
```

## Authentication

Most endpoints require JWT authentication. Include the token in the `Authorization` header:

```bash
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

## Endpoints

### Query API

#### Execute ReQL Query

```http
POST /api/query
```

**Request:**

```json
{
  "query": "r.table('users').filter({active: true})"
}
```

**Response:**

```json
{
  "success": true,
  "data": [
    { "id": "user:1", "name": "Alice", "active": true },
    { "id": "user:2", "name": "Bob", "active": true }
  ],
  "execution_time_ms": 42
}
```

**Example:**

```bash
curl -X POST http://localhost:8080/api/query \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"query":"r.db_list()"}'
```

### Table Management

#### List All Tables

```http
GET /api/tables
```

**Response:**

```json
{
  "success": true,
  "tables": ["users", "posts", "comments"]
}
```

**Example:**

```bash
curl http://localhost:8080/api/tables \
  -H "Authorization: Bearer $TOKEN"
```

#### Get Table Info

```http
GET /api/tables/:name
```

**Response:**

```json
{
  "success": true,
  "table": {
    "name": "users",
    "db": "test",
    "primary_key": "id",
    "doc_count": 1234,
    "indexes": ["email", "created_at"]
  }
}
```

**Example:**

```bash
curl http://localhost:8080/api/tables/users \
  -H "Authorization: Bearer $TOKEN"
```

### Admin & Monitoring

#### Admin Dashboard

```http
GET /_admin
```

Returns HTML dashboard for web UI.

#### Health Check

```http
GET /_health
```

**Response:**

```json
{
  "status": "healthy",
  "version": "3.0.0-alpha",
  "timestamp": "2025-12-09T01:30:00+00:00"
}
```

**Public endpoint** - No authentication required.

#### Readiness Check

```http
GET /_ready
```

**Response:**

```json
{
  "ready": true
}
```

**Public endpoint** - No authentication required.

#### Prometheus Metrics

```http
GET /_metrics
```

**Response:**

```
# RethinkDB 3.0 Metrics
rethinkdb_queries_total 1234
rethinkdb_queries_duration_seconds_sum 45.6
rethinkdb_storage_size_bytes 1048576
```

**Public endpoint** - No authentication required.

## Error Responses

### 400 Bad Request

```json
{
  "success": false,
  "error": "Invalid query syntax"
}
```

### 401 Unauthorized

```json
{
  "error": "Missing or invalid JWT token"
}
```

### 403 Forbidden

```json
{
  "error": "IP address blocked"
}
```

### 429 Too Many Requests

```json
{
  "error": "Rate limit exceeded"
}
```

### 500 Internal Server Error

```json
{
  "success": false,
  "error": "Internal server error"
}
```

## Rate Limiting

Default: **100 requests per minute** per IP address.

Response headers include rate limit information:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1670544000
```

## Related Documentation

- [WebSocket API](websocket.md) - Real-time changefeeds
- [ReQL Reference](reql.md) - Query language
- [Security](../security/README.md) - Authentication

---

**Full API reference for RethinkDB 3.0**
