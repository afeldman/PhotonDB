# GraphQL Strategy for RethinkDB 3.0

## Executive Summary

GraphQL is a **perfect fit** for RethinkDB's real-time, declarative query philosophy. However, we will introduce it **incrementally** to maintain stability and backward compatibility.

## Strategic Roadmap

### Phase 1: v3.0 - Foundation (Current)

**Status:** ✅ In Progress  
**Timeline:** Q1 2025

**Focus:**

- ✅ REST API with 7 endpoints
- ✅ Database hierarchy (Databases → Tables → Documents)
- ✅ WebSocket support for changefeeds
- ✅ Security layer (OAuth2, JWT, Honeytrap)
- ✅ CLI with clap

**Decision:** ❌ No GraphQL in v3.0

- Prioritize core stability
- REST API is simpler and proven
- Focus on getting fundamentals right

### Phase 2: v3.1 - GraphQL Beta (Experimental)

**Status:** 📋 Planned  
**Timeline:** Q2 2025

**Scope:**

- ✅ GraphQL endpoint at `/graphql`
- ✅ Basic queries (databases, tables, documents)
- ✅ **GraphQL Subscriptions for Changefeeds** (killer feature!)
- ✅ GraphQL Playground UI at `/graphql/playground`
- ⚠️ Marked as "Beta" - breaking changes possible

**Implementation:**

```rust
// Crate: async-graphql v7.0+
src/server/graphql/
├── mod.rs              // GraphQL setup
├── schema.rs           // Schema definition
├── query.rs            // Query resolvers
├── mutation.rs         // Mutation resolvers
├── subscription.rs     // Changefeed subscriptions
└── dataloaders.rs      // Batch loading (N+1 prevention)
```

**Example Query:**

```graphql
query GetUserPosts {
  database(name: "myapp") {
    table(name: "posts") {
      documents(filter: { author_id: "user123" }) {
        id
        title
        content
        author {
          # Join via resolver
          name
          email
        }
      }
    }
  }
}
```

**Example Subscription:**

```graphql
subscription WatchUsers {
  changes(db: "myapp", table: "users") {
    operation # INSERT, UPDATE, DELETE
    new_val {
      id
      name
      email
    }
    old_val {
      id
    }
  }
}
```

### Phase 3: v3.2 - GraphQL Stable

**Status:** 📋 Planned  
**Timeline:** Q3 2025

**Goals:**

- ✅ Production-ready GraphQL API
- ✅ Full ReQL feature parity
- ✅ Performance optimization (query complexity limits)
- ✅ DataLoader for efficient batch queries
- ✅ Comprehensive documentation

**Features:**

- Query cost analysis
- Rate limiting per query complexity
- Automatic persisted queries (APQ)
- GraphQL Federation support (for microservices)

### Phase 4: v3.5+ - GraphQL First-Class

**Status:** 🔮 Future  
**Timeline:** 2026+

**Vision:**

- GraphQL as **recommended** API for new applications
- REST API remains for simple use cases
- ReQL as ultimate power-user tool
- Three co-equal API layers:
  - **ReQL:** Full power, backward compatible
  - **REST:** Simple CRUD, administrative
  - **GraphQL:** Modern apps, real-time

## Technical Design

### Architecture

```
                    ┌─────────────────┐
                    │   Client App    │
                    └────────┬────────┘
                             │
            ┌────────────────┼────────────────┐
            │                │                │
        ┌───▼────┐     ┌────▼─────┐    ┌────▼────┐
        │  ReQL  │     │   REST   │    │ GraphQL │
        │  (28015)│    │  (8080)  │    │ (8080)  │
        └───┬────┘     └────┬─────┘    └────┬────┘
            │               │               │
            └───────────────┴───────────────┘
                           │
                    ┌──────▼──────┐
                    │   Storage   │
                    │   Engine    │
                    └─────────────┘
```

### Implementation Stack

**Rust Crates:**

```toml
[dependencies]
# GraphQL
async-graphql = "7.0"
async-graphql-axum = "7.0"

# Subscriptions
tokio-stream = "0.1"
futures = "0.3"

# Performance
dataloader = "0.17"
```

### Schema Design

```graphql
type Database {
  id: ID!
  name: String!
  createdAt: DateTime!
  tables: [Table!]!
}

type Table {
  id: ID!
  name: String!
  primaryKey: String!
  database: Database!
  documents(filter: JSON, limit: Int, offset: Int): [Document!]!
  documentCount: Int!
  indexes: [String!]!
}

type Document {
  id: ID!
  data: JSON!
}

type Query {
  databases: [Database!]!
  database(name: String!): Database
  table(db: String!, name: String!): Table
  document(db: String!, table: String!, id: ID!): Document
}

type Mutation {
  createDatabase(name: String!): Database!
  dropDatabase(name: String!): Boolean!
  createTable(db: String!, name: String!, primaryKey: String): Table!
  dropTable(db: String!, name: String!): Boolean!

  insertDocument(db: String!, table: String!, data: JSON!): Document!
  updateDocument(db: String!, table: String!, id: ID!, data: JSON!): Document!
  deleteDocument(db: String!, table: String!, id: ID!): Boolean!
}

type Subscription {
  changes(db: String!, table: String!, filter: JSON): Change!
}

type Change {
  operation: ChangeOperation!
  newVal: Document
  oldVal: Document
}

enum ChangeOperation {
  INSERT
  UPDATE
  DELETE
}

scalar JSON
scalar DateTime
```

## Benefits Analysis

### Why GraphQL Fits RethinkDB

1. **Declarative Queries**

   - ReQL is already declarative: `r.table('users').filter({active: true})`
   - GraphQL follows same philosophy
   - Natural mapping between concepts

2. **Real-time First**

   - RethinkDB's killer feature: Changefeeds
   - GraphQL Subscriptions are perfect match
   - Better than custom WebSocket protocol

3. **Flexible Data Fetching**

   - Solve over-fetching problem
   - Clients request only needed fields
   - Reduces bandwidth and improves performance

4. **Type Safety**

   - GraphQL schema = Self-documenting API
   - Code generation for clients
   - Better than OpenAPI/Swagger

5. **Developer Experience**
   - GraphQL Playground = Interactive documentation
   - IntelliSense in queries
   - Better than REST + Postman

### Trade-offs

| Aspect             | REST (Current)   | GraphQL (Planned)       |
| ------------------ | ---------------- | ----------------------- |
| **Simplicity**     | ✅ Very simple   | ⚠️ More complex         |
| **Performance**    | ✅ Predictable   | ⚠️ Requires tuning      |
| **Caching**        | ✅ HTTP standard | ⚠️ Custom needed        |
| **Real-time**      | ⚠️ Custom WS     | ✅ Native subscriptions |
| **Over-fetching**  | ❌ Problem       | ✅ Solved               |
| **Learning Curve** | ✅ Low           | ⚠️ Medium               |
| **Tooling**        | ✅ Mature        | ✅ Excellent            |
| **Type Safety**    | ⚠️ Via OpenAPI   | ✅ Native               |

## Implementation Guidelines

### Phase 2 (v3.1) Checklist

**Setup:**

- [ ] Add `async-graphql` and `async-graphql-axum` dependencies
- [ ] Create `src/server/graphql/` module structure
- [ ] Set up GraphQL endpoint at `/graphql`
- [ ] Enable GraphQL Playground at `/graphql/playground`

**Core Features:**

- [ ] Query: List databases
- [ ] Query: Get database by name
- [ ] Query: List tables in database
- [ ] Query: Get documents with filtering
- [ ] Mutation: Create database
- [ ] Mutation: Create table
- [ ] Mutation: Insert document
- [ ] Subscription: Watch table changes (changefeeds!)

**Performance:**

- [ ] Implement DataLoader for batch queries
- [ ] Add query complexity limits
- [ ] Set max query depth (prevent abuse)
- [ ] Rate limiting per query cost

**Testing:**

- [ ] Integration tests for all resolvers
- [ ] Subscription tests with mock changefeeds
- [ ] Performance benchmarks vs REST
- [ ] Load testing (query complexity attacks)

**Documentation:**

- [ ] GraphQL schema documentation
- [ ] Example queries and mutations
- [ ] Subscription examples
- [ ] Migration guide from REST

### Code Example (v3.1)

```rust
// src/server/graphql/mod.rs
use async_graphql::{
    Context, Object, Schema, Subscription, EmptyMutation,
};
use tokio_stream::Stream;

pub type RethinkSchema = Schema<Query, Mutation, Subscription>;

#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    /// List all databases
    async fn databases(&self, ctx: &Context<'_>) -> Vec<Database> {
        let storage = ctx.data::<Arc<Storage>>().unwrap();
        storage.database_engine
            .list_databases()
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|name| Database { name })
            .collect()
    }

    /// Get database by name
    async fn database(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> Option<Database> {
        let storage = ctx.data::<Arc<Storage>>().unwrap();
        storage.database_engine
            .database_exists(&name)
            .await
            .ok()
            .and_then(|exists| {
                if exists {
                    Some(Database { name })
                } else {
                    None
                }
            })
    }
}

#[derive(Default)]
pub struct Mutation;

#[Object]
impl Mutation {
    /// Create a new database
    async fn create_database(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> Result<Database, String> {
        let storage = ctx.data::<Arc<Storage>>().unwrap();
        storage.database_engine
            .create_database(&name)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Database { name })
    }
}

#[derive(Default)]
pub struct Subscription;

#[Subscription]
impl Subscription {
    /// Watch for changes in a table (RethinkDB Changefeed!)
    async fn changes(
        &self,
        ctx: &Context<'_>,
        db: String,
        table: String,
    ) -> impl Stream<Item = Change> {
        let storage = ctx.data::<Arc<Storage>>().unwrap().clone();

        // Create changefeed stream
        async_stream::stream! {
            // Subscribe to changes via storage engine
            // This is where RethinkDB's changefeed magic happens!
            let mut changefeed = storage.watch_table(&db, &table).await;

            while let Some(change) = changefeed.next().await {
                yield Change {
                    operation: change.op,
                    new_val: change.new_val,
                    old_val: change.old_val,
                };
            }
        }
    }
}

// Types
struct Database {
    name: String,
}

#[Object]
impl Database {
    async fn name(&self) -> &str {
        &self.name
    }

    async fn tables(&self, ctx: &Context<'_>) -> Vec<Table> {
        let storage = ctx.data::<Arc<Storage>>().unwrap();
        storage.database_engine
            .list_tables(&self.name)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|name| Table {
                db: self.name.clone(),
                name,
            })
            .collect()
    }
}

struct Table {
    db: String,
    name: String,
}

#[Object]
impl Table {
    async fn name(&self) -> &str {
        &self.name
    }

    async fn database(&self) -> Database {
        Database { name: self.db.clone() }
    }
}
```

## Migration Strategy

### For Existing REST API Users

**No Breaking Changes:**

- REST API continues to work unchanged
- GraphQL is **additive**, not replacement
- Both APIs access same storage backend
- Gradual migration path available

**Migration Path:**

1. Start with REST API (proven, stable)
2. Experiment with GraphQL for new features
3. Use GraphQL subscriptions for real-time
4. Gradually migrate queries to GraphQL
5. Keep REST for simple admin operations

### For New Projects

**Recommendation (v3.2+):**

- Use GraphQL as primary API
- Real-time features via subscriptions
- REST for quick admin scripts
- ReQL for complex analytical queries

## Success Metrics

### Phase 2 (v3.1 Beta)

- [ ] GraphQL API handles 1000 req/s
- [ ] Subscription latency < 50ms
- [ ] Query complexity limits prevent abuse
- [ ] Documentation completeness > 90%
- [ ] Community feedback collected

### Phase 3 (v3.2 Stable)

- [ ] GraphQL adoption > 30% of new projects
- [ ] Performance parity with REST
- [ ] Zero critical bugs for 3 months
- [ ] Client libraries for major languages
- [ ] Conference talks / blog posts

### Phase 4 (v3.5+)

- [ ] GraphQL becomes recommended API
- [ ] 50%+ of traffic via GraphQL
- [ ] Federation support for microservices
- [ ] GraphQL becomes RethinkDB differentiator

## Resources

### Learning Materials

- [GraphQL Official Docs](https://graphql.org/)
- [async-graphql Book](https://async-graphql.github.io/async-graphql/)
- [GraphQL Best Practices](https://graphql.org/learn/best-practices/)

### Inspiration

- [Hasura](https://hasura.io/) - GraphQL for PostgreSQL
- [Prisma](https://www.prisma.io/) - GraphQL ORM
- [Apollo Server](https://www.apollographql.com/) - Node.js GraphQL

### RethinkDB Context

- RethinkDB already has declarative queries (ReQL)
- Changefeeds are perfect for GraphQL subscriptions
- Document model maps naturally to GraphQL types

## Conclusion

GraphQL is the **future** of RethinkDB's API, but we'll introduce it **responsibly**:

1. ✅ **v3.0:** Focus on core (REST + ReQL)
2. 🚧 **v3.1:** GraphQL Beta (experimental)
3. ✅ **v3.2:** GraphQL Stable (production-ready)
4. 🚀 **v3.5+:** GraphQL First-Class (recommended)

This strategy balances **innovation** with **stability**, ensuring we don't sacrifice RethinkDB's reliability for shiny new features.

**The killer feature:** GraphQL Subscriptions + RethinkDB Changefeeds = Real-time paradise! 🚀

---

_Last Updated: December 9, 2025_  
_Author: Anton Feldmann_  
_Status: Strategic Planning Document_
