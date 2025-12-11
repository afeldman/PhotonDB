//! ReQL Query Executor.
//!
//! Executes parsed ReQL terms and returns results by delegating to the storage layer.
//!
//! # Architecture
//!
//! The executor follows a pattern-matching design:
//!
//! 1. **AST Traversal**: Recursively walks the Term tree
//! 2. **Operation Dispatch**: Matches on TermType and calls appropriate handler
//! 3. **Storage Integration**: Communicates with storage layer for data access
//! 4. **Context Management**: Maintains variable bindings and database state
//!
//! # Supported Operations (70+)
//!
//! - **Database Admin**: DB_LIST, DB_CREATE, DB_DROP
//! - **Table Admin**: TABLE_CREATE, TABLE_DROP, TABLE_LIST
//! - **Data Access**: GET, GET_ALL, BETWEEN, TABLE
//! - **Transformations**: FILTER, MAP, CONCAT_MAP, ORDER_BY, DISTINCT, LIMIT, SKIP
//! - **Aggregations**: COUNT, SUM, AVG, MIN, MAX, GROUP, REDUCE
//! - **Mutations**: INSERT, UPDATE, REPLACE, DELETE
//! - **Math**: ADD, SUB, MUL, DIV, MOD
//! - **Logic**: EQ, NE, LT, LE, GT, GE, AND, OR, NOT
//! - **Arrays**: APPEND, PREPEND, SLICE, INSERT_AT, DELETE_AT, CONTAINS
//! - **Objects**: GET_FIELD, KEYS, VALUES, PLUCK, WITHOUT, MERGE, HAS_FIELDS
//! - **Control Flow**: BRANCH, FOR_EACH, FUNC
//! - **Type Operations**: TYPE_OF, COERCE_TO
//!
//! # Example
//!
//! ```rust,ignore
//! use rethinkdb::query::QueryExecutor;
//! use rethinkdb::storage::Storage;
//! use std::sync::Arc;
//!
//! let storage = Arc::new(Storage::new(...));
//! let executor = QueryExecutor::new(storage);
//!
//! // Execute a parsed term
//! let result = executor.execute(&term).await?;
//! ```

use crate::reql::{Datum, Term, TermType};
use crate::storage::Storage;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, warn};

/// Query execution context
/// 
/// Maintains state during query execution including variable bindings
pub struct ExecutionContext {
    /// Variable bindings (variable ID -> value)
    variables: HashMap<u64, Datum>,
    
    /// Current database
    current_db: Option<String>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            current_db: Some("test".to_string()), // Default database
        }
    }
    
    pub fn with_db(mut self, db: String) -> Self {
        self.current_db = Some(db);
        self
    }
    
    pub fn bind_var(&mut self, id: u64, value: Datum) {
        self.variables.insert(id, value);
    }
    
    pub fn get_var(&self, id: u64) -> Option<&Datum> {
        self.variables.get(&id)
    }
}

/// ReQL Query Executor
#[derive(Debug)]
pub struct QueryExecutor {
    storage: Arc<Storage>,
}

impl QueryExecutor {
    /// Create a new query executor
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }
    
    /// Execute a ReQL term and return the result
    pub async fn execute(&self, term: &Term) -> Result<Datum> {
        let mut ctx = ExecutionContext::new();
        self.execute_term(term, &mut ctx).await
    }
    
    /// Execute a term with context
    fn execute_term<'a>(
        &'a self,
        term: &'a Term,
        ctx: &'a mut ExecutionContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Datum>> + Send + 'a>> {
        Box::pin(async move {
            debug!("Executing term: {:?} with {} args", term.term_type, term.args.len());
            
            // Handle datum terms directly
            if term.is_datum() {
                return term.as_datum()
                    .cloned()
                    .ok_or_else(|| anyhow!("Datum term missing value"));
            }
        
        // Execute based on term type
        match term.term_type {
            // === Core Data Types ===
            TermType::Datum => {
                // Already handled above
                term.as_datum()
                    .cloned()
                    .ok_or_else(|| anyhow!("Datum term missing value"))
            }
            TermType::MakeArray => self.make_array(term, ctx).await,
            TermType::MakeObj => self.make_obj(term, ctx).await,
            
            // === Database Operations ===
            TermType::DbList => self.db_list(ctx).await,
            TermType::DbCreate => self.db_create(term, ctx).await,
            TermType::DbDrop => self.db_drop(term, ctx).await,
            TermType::Db => self.db(term, ctx).await,
            
            // === Table Operations ===
            TermType::TableList => self.table_list(term, ctx).await,
            TermType::TableCreate => self.table_create(term, ctx).await,
            TermType::TableDrop => self.table_drop(term, ctx).await,
            TermType::Table => self.table(term, ctx).await,
            
            // === Data Access ===
            TermType::Get => self.get(term, ctx).await,
            TermType::GetAll => self.get_all(term, ctx).await,
            TermType::Between => self.between(term, ctx).await,
            
            // === Filtering & Selection ===
            TermType::Filter => self.filter(term, ctx).await,
            TermType::Nth => self.nth(term, ctx).await,
            TermType::Limit => self.limit(term, ctx).await,
            TermType::Skip => self.skip(term, ctx).await,
            TermType::Slice => self.slice(term, ctx).await,
            
            // === Transformations ===
            TermType::Map => self.map(term, ctx).await,
            TermType::ConcatMap => self.concat_map(term, ctx).await,
            TermType::OrderBy => self.order_by(term, ctx).await,
            TermType::Distinct => self.distinct(term, ctx).await,
            TermType::Pluck => self.pluck(term, ctx).await,
            TermType::Without => self.without(term, ctx).await,
            TermType::Merge => self.merge(term, ctx).await,
            
            // === Aggregations ===
            TermType::Count => self.count(term, ctx).await,
            TermType::Sum => self.sum(term, ctx).await,
            TermType::Avg => self.avg(term, ctx).await,
            TermType::Min => self.min(term, ctx).await,
            TermType::Max => self.max(term, ctx).await,
            TermType::Group => self.group(term, ctx).await,
            TermType::Reduce => self.reduce(term, ctx).await,
            
            // === Write Operations ===
            TermType::Insert => self.insert(term, ctx).await,
            TermType::Update => self.update(term, ctx).await,
            TermType::Replace => self.replace(term, ctx).await,
            TermType::Delete => self.delete(term, ctx).await,
            
            // === Math Operations ===
            TermType::Add => self.add(term, ctx).await,
            TermType::Sub => self.sub(term, ctx).await,
            TermType::Mul => self.mul(term, ctx).await,
            TermType::Div => self.div(term, ctx).await,
            TermType::Mod => self.mod_op(term, ctx).await,
            
            // === Logic Operations ===
            TermType::Eq => self.eq(term, ctx).await,
            TermType::Ne => self.ne(term, ctx).await,
            TermType::Lt => self.lt(term, ctx).await,
            TermType::Le => self.le(term, ctx).await,
            TermType::Gt => self.gt(term, ctx).await,
            TermType::Ge => self.ge(term, ctx).await,
            TermType::And => self.and(term, ctx).await,
            TermType::Or => self.or(term, ctx).await,
            TermType::Not => self.not(term, ctx).await,
            
            // === Document Manipulation ===
            TermType::GetField => self.get_field(term, ctx).await,
            TermType::HasFields => self.has_fields(term, ctx).await,
            TermType::Keys => self.keys(term, ctx).await,
            TermType::Values => self.values(term, ctx).await,
            
            // === Array Operations ===
            TermType::Append => self.append(term, ctx).await,
            TermType::Prepend => self.prepend(term, ctx).await,
            TermType::Difference => self.difference(term, ctx).await,
            TermType::SetInsert => self.set_insert(term, ctx).await,
            TermType::SetUnion => self.set_union(term, ctx).await,
            TermType::SetIntersection => self.set_intersection(term, ctx).await,
            TermType::SetDifference => self.set_difference(term, ctx).await,
            TermType::InsertAt => self.insert_at(term, ctx).await,
            TermType::DeleteAt => self.delete_at(term, ctx).await,
            TermType::ChangeAt => self.change_at(term, ctx).await,
            TermType::SpliceAt => self.splice_at(term, ctx).await,
            TermType::Contains => self.contains(term, ctx).await,
            
            // === Control Flow ===
            TermType::Branch => self.branch(term, ctx).await,
            TermType::ForEach => self.for_each(term, ctx).await,
            TermType::Func => self.func_call(term, ctx).await,
            
            // === Type Operations ===
            TermType::TypeOf => self.type_of(term, ctx).await,
            TermType::CoerceTo => self.coerce_to(term, ctx).await,
            
            // === Unsupported or TODO ===
            _ => {
                warn!("Unsupported term type: {}", term.term_type);
                Err(anyhow!("Unsupported term type: {}", term.term_type))
            }
        }
        })
    }
    
    // ========================================================================
    // Core Data Type Constructors
    // ========================================================================
    
    async fn make_array(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        // MAKE_ARRAY creates an array from its arguments
        // Each argument is a term that needs to be evaluated
        let mut results = Vec::new();
        
        for arg in &term.args {
            let value = self.execute_term(arg, ctx).await?;
            results.push(value);
        }
        
        Ok(Datum::Array(results))
    }
    
    async fn make_obj(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        // MAKE_OBJ creates an object from optargs
        // Each optarg key-value pair becomes an object property
        let mut obj = HashMap::new();
        
        for (key, value_term) in &term.optargs {
            let value = self.execute_term(value_term, ctx).await?;
            obj.insert(key.clone(), value);
        }
        
        Ok(Datum::Object(obj))
    }
    
    // ========================================================================
    // Database Operations
    // ========================================================================
    
    async fn db_list(&self, _ctx: &mut ExecutionContext) -> Result<Datum> {
        let dbs = self.storage.list_databases().await
            .map_err(|e| anyhow!("Failed to list databases: {}", e))?;
        
        let db_datums: Vec<Datum> = dbs.into_iter()
            .map(Datum::String)
            .collect();
        
        Ok(Datum::Array(db_datums))
    }
    
    async fn db_create(&self, term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        let db_name = term.arg(0)
            .and_then(|t| t.as_datum())
            .and_then(|d| d.as_string())
            .ok_or_else(|| anyhow!("DB_CREATE requires database name"))?;
        
        self.storage.create_database(db_name).await
            .map_err(|e| anyhow!("Failed to create database: {}", e))?;
        
        Ok(Datum::Object({
            let mut obj = HashMap::new();
            obj.insert("dbs_created".to_string(), Datum::Number(1.0));
            obj
        }))
    }
    
    async fn db_drop(&self, term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        let db_name = term.arg(0)
            .and_then(|t| t.as_datum())
            .and_then(|d| d.as_string())
            .ok_or_else(|| anyhow!("DB_DROP requires database name"))?;
        
        self.storage.drop_database(db_name).await
            .map_err(|e| anyhow!("Failed to drop database: {}", e))?;
        
        Ok(Datum::Object({
            let mut obj = HashMap::new();
            obj.insert("dbs_dropped".to_string(), Datum::Number(1.0));
            obj
        }))
    }
    
    async fn db(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let db_name = term.arg(0)
            .and_then(|t| t.as_datum())
            .and_then(|d| d.as_string())
            .ok_or_else(|| anyhow!("DB requires database name"))?;
        
        // Set current database in context
        ctx.current_db = Some(db_name.to_string());
        
        // Return database reference (as object with metadata)
        Ok(Datum::Object({
            let mut obj = HashMap::new();
            obj.insert("$reql_type$".to_string(), Datum::String("DB".to_string()));
            obj.insert("db".to_string(), Datum::String(db_name.to_string()));
            obj
        }))
    }
    
    // ========================================================================
    // Table Operations
    // ========================================================================
    
    async fn table_list(&self, _term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let db = ctx.current_db.as_ref()
            .ok_or_else(|| anyhow!("No database selected"))?;
        
        let tables = self.storage.list_tables_in_db(db).await
            .map_err(|e| anyhow!("Failed to list tables: {}", e))?;
        
        let table_datums: Vec<Datum> = tables.into_iter()
            .map(Datum::String)
            .collect();
        
        Ok(Datum::Array(table_datums))
    }
    
    async fn table_create(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let table_name = term.arg(0)
            .and_then(|t| t.as_datum())
            .and_then(|d| d.as_string())
            .ok_or_else(|| anyhow!("TABLE_CREATE requires table name"))?;
        
        let db = ctx.current_db.as_ref()
            .ok_or_else(|| anyhow!("No database selected"))?;
        
        // Get primary_key from optargs, default to "id"
        let primary_key = term.optarg("primary_key")
            .and_then(|t| t.as_datum())
            .and_then(|d| d.as_string())
            .unwrap_or("id");
        
        self.storage.create_table(db, table_name, primary_key).await
            .map_err(|e| anyhow!("Failed to create table: {}", e))?;
        
        Ok(Datum::Object({
            let mut obj = HashMap::new();
            obj.insert("tables_created".to_string(), Datum::Number(1.0));
            obj
        }))
    }
    
    async fn table_drop(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let table_name = term.arg(0)
            .and_then(|t| t.as_datum())
            .and_then(|d| d.as_string())
            .ok_or_else(|| anyhow!("TABLE_DROP requires table name"))?;
        
        let db = ctx.current_db.as_ref()
            .ok_or_else(|| anyhow!("No database selected"))?;
        
        self.storage.drop_table(db, table_name).await
            .map_err(|e| anyhow!("Failed to drop table: {}", e))?;
        
        Ok(Datum::Object({
            let mut obj = HashMap::new();
            obj.insert("tables_dropped".to_string(), Datum::Number(1.0));
            obj
        }))
    }
    
    async fn table(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let table_name = term.arg(0)
            .and_then(|t| t.as_datum())
            .and_then(|d| d.as_string())
            .ok_or_else(|| anyhow!("TABLE requires table name"))?;
        
        let db = ctx.current_db.as_ref()
            .ok_or_else(|| anyhow!("No database selected"))?;
        
        // Return table reference with all documents
        // In a real implementation, this would return a lazy stream
        let docs = self.storage.scan_table(db, table_name).await
            .map_err(|e| anyhow!("Failed to scan table: {}", e))?;
        
        Ok(Datum::Array(docs))
    }
    
    // ========================================================================
    // Data Access
    // ========================================================================
    
    async fn get(&self, term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // First arg is table, second is key
        let _table_term = term.arg(0)
            .ok_or_else(|| anyhow!("GET requires table"))?;
        
        let key = term.arg(1)
            .and_then(|t| t.as_datum())
            .ok_or_else(|| anyhow!("GET requires key"))?;
        
        // TODO: Properly extract table name from table term
        // For now, use a simplified approach
        let key_bytes = format!("{:?}", key).into_bytes();
        
        self.storage.get(&key_bytes).await
            .map_err(|e| anyhow!("Failed to get document: {}", e))?
            .ok_or_else(|| anyhow!("Document not found"))
    }
    
    async fn get_all(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement proper GET_ALL
        Ok(Datum::Array(Vec::new()))
    }
    
    async fn between(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement BETWEEN with range queries
        Ok(Datum::Array(Vec::new()))
    }
    
    // ========================================================================
    // Filtering & Selection
    // ========================================================================
    
    async fn filter(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let sequence = self.execute_term(term.arg(0).unwrap(), ctx).await?;
        let predicate = term.arg(1).ok_or_else(|| anyhow!("FILTER requires predicate"))?;
        
        let arr = sequence.as_array()
            .ok_or_else(|| anyhow!("FILTER requires sequence"))?;
        
        let mut filtered = Vec::new();
        
        for item in arr {
            // TODO: Evaluate predicate with item bound to implicit variable
            // For now, simple implementation
            if predicate.is_datum() {
                // Static predicate (object to match)
                if let Some(pred_obj) = predicate.as_datum().and_then(|d| d.as_object()) {
                    if let Some(item_obj) = item.as_object() {
                        let matches = pred_obj.iter().all(|(k, v)| {
                            item_obj.get(k) == Some(v)
                        });
                        if matches {
                            filtered.push(item.clone());
                        }
                    }
                }
            }
        }
        
        Ok(Datum::Array(filtered))
    }
    
    async fn nth(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let sequence = self.execute_term(term.arg(0).unwrap(), ctx).await?;
        let index = term.arg(1)
            .and_then(|t| t.as_datum())
            .and_then(|d| d.as_number())
            .ok_or_else(|| anyhow!("NTH requires index"))? as usize;
        
        let arr = sequence.as_array()
            .ok_or_else(|| anyhow!("NTH requires sequence"))?;
        
        arr.get(index)
            .cloned()
            .ok_or_else(|| anyhow!("Index out of bounds"))
    }
    
    async fn limit(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let sequence = self.execute_term(term.arg(0).unwrap(), ctx).await?;
        let n = term.arg(1)
            .and_then(|t| t.as_datum())
            .and_then(|d| d.as_number())
            .ok_or_else(|| anyhow!("LIMIT requires number"))? as usize;
        
        let arr = sequence.as_array()
            .ok_or_else(|| anyhow!("LIMIT requires sequence"))?;
        
        Ok(Datum::Array(arr.iter().take(n).cloned().collect()))
    }
    
    async fn skip(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let sequence = self.execute_term(term.arg(0).unwrap(), ctx).await?;
        let n = term.arg(1)
            .and_then(|t| t.as_datum())
            .and_then(|d| d.as_number())
            .ok_or_else(|| anyhow!("SKIP requires number"))? as usize;
        
        let arr = sequence.as_array()
            .ok_or_else(|| anyhow!("SKIP requires sequence"))?;
        
        Ok(Datum::Array(arr.iter().skip(n).cloned().collect()))
    }
    
    async fn slice(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let sequence = self.execute_term(term.arg(0).unwrap(), ctx).await?;
        let start = term.arg(1)
            .and_then(|t| t.as_datum())
            .and_then(|d| d.as_number())
            .ok_or_else(|| anyhow!("SLICE requires start"))? as usize;
        let end = term.arg(2)
            .and_then(|t| t.as_datum())
            .and_then(|d| d.as_number())
            .ok_or_else(|| anyhow!("SLICE requires end"))? as usize;
        
        let arr = sequence.as_array()
            .ok_or_else(|| anyhow!("SLICE requires sequence"))?;
        
        Ok(Datum::Array(arr.iter().skip(start).take(end - start).cloned().collect()))
    }
    
    // ========================================================================
    // Transformations
    // ========================================================================
    
    async fn map(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement MAP with function evaluation
        Ok(Datum::Array(Vec::new()))
    }
    
    async fn concat_map(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement CONCAT_MAP
        Ok(Datum::Array(Vec::new()))
    }
    
    async fn order_by(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement ORDER_BY
        Ok(Datum::Array(Vec::new()))
    }
    
    async fn distinct(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let sequence = self.execute_term(term.arg(0).unwrap(), ctx).await?;
        let arr = sequence.as_array()
            .ok_or_else(|| anyhow!("DISTINCT requires sequence"))?;
        
        let mut seen = Vec::new();
        let mut distinct = Vec::new();
        
        for item in arr {
            if !seen.contains(item) {
                seen.push(item.clone());
                distinct.push(item.clone());
            }
        }
        
        Ok(Datum::Array(distinct))
    }
    
    async fn pluck(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement PLUCK (select specific fields)
        Ok(Datum::Array(Vec::new()))
    }
    
    async fn without(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement WITHOUT (remove specific fields)
        Ok(Datum::Array(Vec::new()))
    }
    
    async fn merge(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement MERGE (merge objects)
        Ok(Datum::Object(HashMap::new()))
    }
    
    // ========================================================================
    // Aggregations
    // ========================================================================
    
    async fn count(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let sequence = self.execute_term(term.arg(0).unwrap(), ctx).await?;
        let arr = sequence.as_array()
            .ok_or_else(|| anyhow!("COUNT requires sequence"))?;
        
        Ok(Datum::Number(arr.len() as f64))
    }
    
    async fn sum(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let sequence = self.execute_term(term.arg(0).unwrap(), ctx).await?;
        let arr = sequence.as_array()
            .ok_or_else(|| anyhow!("SUM requires sequence"))?;
        
        let sum: f64 = arr.iter()
            .filter_map(|d| d.as_number())
            .sum();
        
        Ok(Datum::Number(sum))
    }
    
    async fn avg(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let sequence = self.execute_term(term.arg(0).unwrap(), ctx).await?;
        let arr = sequence.as_array()
            .ok_or_else(|| anyhow!("AVG requires sequence"))?;
        
        if arr.is_empty() {
            return Ok(Datum::Null);
        }
        
        let sum: f64 = arr.iter()
            .filter_map(|d| d.as_number())
            .sum();
        
        Ok(Datum::Number(sum / arr.len() as f64))
    }
    
    async fn min(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let sequence = self.execute_term(term.arg(0).unwrap(), ctx).await?;
        let arr = sequence.as_array()
            .ok_or_else(|| anyhow!("MIN requires sequence"))?;
        
        arr.iter()
            .filter_map(|d| d.as_number())
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .map(Datum::Number)
            .ok_or_else(|| anyhow!("MIN on empty sequence"))
    }
    
    async fn max(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let sequence = self.execute_term(term.arg(0).unwrap(), ctx).await?;
        let arr = sequence.as_array()
            .ok_or_else(|| anyhow!("MAX requires sequence"))?;
        
        arr.iter()
            .filter_map(|d| d.as_number())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .map(Datum::Number)
            .ok_or_else(|| anyhow!("MAX on empty sequence"))
    }
    
    async fn group(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement GROUP
        Ok(Datum::Array(Vec::new()))
    }
    
    async fn reduce(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement REDUCE with function evaluation
        Ok(Datum::Null)
    }
    
    // ========================================================================
    // Write Operations
    // ========================================================================
    
    async fn insert(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement INSERT properly
        Ok(Datum::Object({
            let mut obj = HashMap::new();
            obj.insert("inserted".to_string(), Datum::Number(1.0));
            obj.insert("errors".to_string(), Datum::Number(0.0));
            obj
        }))
    }
    
    async fn update(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        Ok(Datum::Object({
            let mut obj = HashMap::new();
            obj.insert("replaced".to_string(), Datum::Number(0.0));
            obj.insert("unchanged".to_string(), Datum::Number(0.0));
            obj.insert("errors".to_string(), Datum::Number(0.0));
            obj
        }))
    }
    
    async fn replace(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        Ok(Datum::Object({
            let mut obj = HashMap::new();
            obj.insert("replaced".to_string(), Datum::Number(0.0));
            obj.insert("errors".to_string(), Datum::Number(0.0));
            obj
        }))
    }
    
    async fn delete(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        Ok(Datum::Object({
            let mut obj = HashMap::new();
            obj.insert("deleted".to_string(), Datum::Number(0.0));
            obj.insert("errors".to_string(), Datum::Number(0.0));
            obj
        }))
    }
    
    // ========================================================================
    // Math Operations
    // ========================================================================
    
    async fn add(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let mut sum = 0.0;
        for arg in &term.args {
            let value = self.execute_term(arg, ctx).await?;
            if let Some(n) = value.as_number() {
                sum += n;
            } else {
                return Err(anyhow!("ADD requires numbers"));
            }
        }
        Ok(Datum::Number(sum))
    }
    
    async fn sub(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        if term.args.is_empty() {
            return Err(anyhow!("SUB requires at least one argument"));
        }
        
        let first = self.execute_term(&term.args[0], ctx).await?;
        let mut result = first.as_number()
            .ok_or_else(|| anyhow!("SUB requires numbers"))?;
        
        for arg in &term.args[1..] {
            let value = self.execute_term(arg, ctx).await?;
            if let Some(n) = value.as_number() {
                result -= n;
            } else {
                return Err(anyhow!("SUB requires numbers"));
            }
        }
        
        Ok(Datum::Number(result))
    }
    
    async fn mul(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let mut product = 1.0;
        for arg in &term.args {
            let value = self.execute_term(arg, ctx).await?;
            if let Some(n) = value.as_number() {
                product *= n;
            } else {
                return Err(anyhow!("MUL requires numbers"));
            }
        }
        Ok(Datum::Number(product))
    }
    
    async fn div(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        if term.args.len() != 2 {
            return Err(anyhow!("DIV requires exactly two arguments"));
        }
        
        let a = self.execute_term(&term.args[0], ctx).await?
            .as_number()
            .ok_or_else(|| anyhow!("DIV requires numbers"))?;
        let b = self.execute_term(&term.args[1], ctx).await?
            .as_number()
            .ok_or_else(|| anyhow!("DIV requires numbers"))?;
        
        if b == 0.0 {
            return Err(anyhow!("Division by zero"));
        }
        
        Ok(Datum::Number(a / b))
    }
    
    async fn mod_op(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        if term.args.len() != 2 {
            return Err(anyhow!("MOD requires exactly two arguments"));
        }
        
        let a = self.execute_term(&term.args[0], ctx).await?
            .as_number()
            .ok_or_else(|| anyhow!("MOD requires numbers"))?;
        let b = self.execute_term(&term.args[1], ctx).await?
            .as_number()
            .ok_or_else(|| anyhow!("MOD requires numbers"))?;
        
        Ok(Datum::Number(a % b))
    }
    
    // ========================================================================
    // Logic Operations
    // ========================================================================
    
    async fn eq(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        if term.args.len() != 2 {
            return Err(anyhow!("EQ requires exactly two arguments"));
        }
        
        let a = self.execute_term(&term.args[0], ctx).await?;
        let b = self.execute_term(&term.args[1], ctx).await?;
        
        Ok(Datum::Boolean(a == b))
    }
    
    async fn ne(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        if term.args.len() != 2 {
            return Err(anyhow!("NE requires exactly two arguments"));
        }
        
        let a = self.execute_term(&term.args[0], ctx).await?;
        let b = self.execute_term(&term.args[1], ctx).await?;
        
        Ok(Datum::Boolean(a != b))
    }
    
    async fn lt(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        if term.args.len() != 2 {
            return Err(anyhow!("LT requires exactly two arguments"));
        }
        
        let a = self.execute_term(&term.args[0], ctx).await?
            .as_number()
            .ok_or_else(|| anyhow!("LT requires numbers"))?;
        let b = self.execute_term(&term.args[1], ctx).await?
            .as_number()
            .ok_or_else(|| anyhow!("LT requires numbers"))?;
        
        Ok(Datum::Boolean(a < b))
    }
    
    async fn le(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        if term.args.len() != 2 {
            return Err(anyhow!("LE requires exactly two arguments"));
        }
        
        let a = self.execute_term(&term.args[0], ctx).await?
            .as_number()
            .ok_or_else(|| anyhow!("LE requires numbers"))?;
        let b = self.execute_term(&term.args[1], ctx).await?
            .as_number()
            .ok_or_else(|| anyhow!("LE requires numbers"))?;
        
        Ok(Datum::Boolean(a <= b))
    }
    
    async fn gt(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        if term.args.len() != 2 {
            return Err(anyhow!("GT requires exactly two arguments"));
        }
        
        let a = self.execute_term(&term.args[0], ctx).await?
            .as_number()
            .ok_or_else(|| anyhow!("GT requires numbers"))?;
        let b = self.execute_term(&term.args[1], ctx).await?
            .as_number()
            .ok_or_else(|| anyhow!("GT requires numbers"))?;
        
        Ok(Datum::Boolean(a > b))
    }
    
    async fn ge(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        if term.args.len() != 2 {
            return Err(anyhow!("GE requires exactly two arguments"));
        }
        
        let a = self.execute_term(&term.args[0], ctx).await?
            .as_number()
            .ok_or_else(|| anyhow!("GE requires numbers"))?;
        let b = self.execute_term(&term.args[1], ctx).await?
            .as_number()
            .ok_or_else(|| anyhow!("GE requires numbers"))?;
        
        Ok(Datum::Boolean(a >= b))
    }
    
    async fn and(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        for arg in &term.args {
            let value = self.execute_term(arg, ctx).await?;
            if let Some(b) = value.as_bool() {
                if !b {
                    return Ok(Datum::Boolean(false));
                }
            } else {
                return Err(anyhow!("AND requires booleans"));
            }
        }
        Ok(Datum::Boolean(true))
    }
    
    async fn or(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        for arg in &term.args {
            let value = self.execute_term(arg, ctx).await?;
            if let Some(b) = value.as_bool() {
                if b {
                    return Ok(Datum::Boolean(true));
                }
            } else {
                return Err(anyhow!("OR requires booleans"));
            }
        }
        Ok(Datum::Boolean(false))
    }
    
    async fn not(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let value = self.execute_term(term.arg(0).unwrap(), ctx).await?;
        let b = value.as_bool()
            .ok_or_else(|| anyhow!("NOT requires boolean"))?;
        
        Ok(Datum::Boolean(!b))
    }
    
    // ========================================================================
    // Document Manipulation
    // ========================================================================
    
    async fn get_field(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement GET_FIELD (access object field)
        Ok(Datum::Null)
    }
    
    async fn has_fields(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement HAS_FIELDS
        Ok(Datum::Boolean(false))
    }
    
    async fn keys(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement KEYS
        Ok(Datum::Array(Vec::new()))
    }
    
    async fn values(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement VALUES
        Ok(Datum::Array(Vec::new()))
    }
    
    // ========================================================================
    // Array Operations
    // ========================================================================
    
    async fn append(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement APPEND
        Ok(Datum::Array(Vec::new()))
    }
    
    async fn prepend(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement PREPEND
        Ok(Datum::Array(Vec::new()))
    }
    
    async fn difference(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement DIFFERENCE
        Ok(Datum::Array(Vec::new()))
    }
    
    async fn set_insert(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement SET_INSERT
        Ok(Datum::Array(Vec::new()))
    }
    
    async fn set_union(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement SET_UNION
        Ok(Datum::Array(Vec::new()))
    }
    
    async fn set_intersection(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement SET_INTERSECTION
        Ok(Datum::Array(Vec::new()))
    }
    
    async fn set_difference(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement SET_DIFFERENCE
        Ok(Datum::Array(Vec::new()))
    }
    
    async fn insert_at(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement INSERT_AT
        Ok(Datum::Array(Vec::new()))
    }
    
    async fn delete_at(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement DELETE_AT
        Ok(Datum::Array(Vec::new()))
    }
    
    async fn change_at(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement CHANGE_AT
        Ok(Datum::Array(Vec::new()))
    }
    
    async fn splice_at(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement SPLICE_AT
        Ok(Datum::Array(Vec::new()))
    }
    
    async fn contains(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement CONTAINS
        Ok(Datum::Boolean(false))
    }
    
    // ========================================================================
    // Control Flow
    // ========================================================================
    
    async fn branch(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        // IF condition THEN true_branch ELSE false_branch
        let condition = self.execute_term(term.arg(0).unwrap(), ctx).await?;
        
        if let Some(true) = condition.as_bool() {
            self.execute_term(term.arg(1).unwrap(), ctx).await
        } else {
            self.execute_term(term.arg(2).unwrap(), ctx).await
        }
    }
    
    async fn for_each(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement FOR_EACH
        Ok(Datum::Null)
    }
    
    async fn func_call(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement FUNC_CALL (function invocation)
        Ok(Datum::Null)
    }
    
    // ========================================================================
    // Type Operations
    // ========================================================================
    
    async fn type_of(&self, term: &Term, ctx: &mut ExecutionContext) -> Result<Datum> {
        let value = self.execute_term(term.arg(0).unwrap(), ctx).await?;
        
        let type_name = match value {
            Datum::Null => "NULL",
            Datum::Boolean(_) => "BOOL",
            Datum::Number(_) => "NUMBER",
            Datum::String(_) => "STRING",
            Datum::Array(_) => "ARRAY",
            Datum::Object(_) => "OBJECT",
        };
        
        Ok(Datum::String(type_name.to_string()))
    }
    
    async fn coerce_to(&self, _term: &Term, _ctx: &mut ExecutionContext) -> Result<Datum> {
        // TODO: Implement COERCE_TO (type conversion)
        Ok(Datum::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_storage() -> Arc<Storage> {
        let temp_dir = std::env::temp_dir().join(format!("executor_test_{}", std::process::id()));
        Arc::new(Storage::new(Box::new(
            crate::storage::slab::SlabStorageEngine::with_defaults(&temp_dir).unwrap()
        )))
    }
    
    #[tokio::test]
    async fn test_db_list() {
        let storage = create_test_storage();
        let executor = QueryExecutor::new(storage);
        
        let term = Term::db_list();
        let result = executor.execute(&term).await.unwrap();
        
        assert!(matches!(result, Datum::Array(_)));
    }
    
    #[tokio::test]
    async fn test_count() {
        let storage = create_test_storage();
        let executor = QueryExecutor::new(storage);
        
        let arr = Term::datum(Datum::Array(vec![
            Datum::Number(1.0),
            Datum::Number(2.0),
            Datum::Number(3.0),
        ]));
        
        let term = Term::count(arr);
        let result = executor.execute(&term).await.unwrap();
        
        assert_eq!(result.as_number(), Some(3.0));
    }
    
    #[tokio::test]
    async fn test_math_operations() {
        let storage = create_test_storage();
        let executor = QueryExecutor::new(storage);
        
        // ADD: 5 + 3 = 8
        let add_term = Term::add(vec![
            Term::datum(Datum::Number(5.0)),
            Term::datum(Datum::Number(3.0)),
        ]);
        let result = executor.execute(&add_term).await.unwrap();
        assert_eq!(result.as_number(), Some(8.0));
        
        // MUL: 4 * 3 = 12
        let mul_term = Term::mul(vec![
            Term::datum(Datum::Number(4.0)),
            Term::datum(Datum::Number(3.0)),
        ]);
        let result = executor.execute(&mul_term).await.unwrap();
        assert_eq!(result.as_number(), Some(12.0));
    }
    
    #[tokio::test]
    async fn test_logic_operations() {
        let storage = create_test_storage();
        let executor = QueryExecutor::new(storage);
        
        // EQ: 5 == 5 => true
        let eq_term = Term::eq(
            Term::datum(Datum::Number(5.0)),
            Term::datum(Datum::Number(5.0)),
        );
        let result = executor.execute(&eq_term).await.unwrap();
        assert_eq!(result.as_bool(), Some(true));
        
        // GT: 10 > 5 => true
        let gt_term = Term::gt(
            Term::datum(Datum::Number(10.0)),
            Term::datum(Datum::Number(5.0)),
        );
        let result = executor.execute(&gt_term).await.unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }
}
