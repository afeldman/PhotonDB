//! ReQL Abstract Syntax Tree (AST) implementation.
//!
//! This module provides the AST representation for RethinkDB queries.
//! A query is represented as a tree of `Term` nodes, where each node has:
//!
//! - A `TermType` specifying the operation
//! - Positional arguments (`args`): child terms
//! - Optional named arguments (`optargs`): key-value pairs
//! - Optional datum value for literal data
//!
//! # Architecture
//!
//! The AST uses a builder pattern for fluent query construction:
//!
//! ```rust,ignore
//! let query = Term::new(TermType::Filter)
//!     .with_arg(table_term)
//!     .with_arg(predicate_term)
//!     .with_optarg("default", default_value);
//! ```
//!
//! # Example
//!
//! Building a query: `r.table("users").filter({age: 25})`
//!
//! ```rust,ignore
//! use rethinkdb::reql::{Term, TermType, Datum};
//! use std::collections::HashMap;
//!
//! let mut filter_obj = HashMap::new();
//! filter_obj.insert("age".to_string(), Datum::Number(25.0));
//!
//! let query = Term::new(TermType::Filter)
//!     .with_arg(
//!         Term::new(TermType::Table)
//!             .with_arg(Term::datum(Datum::String("users".into())))
//!     )
//!     .with_arg(Term::datum(Datum::Object(filter_obj)));
//! ```

use super::datum::Datum;
use super::terms::TermType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A ReQL Term - the fundamental building block of queries.
///
/// Represents a single node in the query AST tree. Each term consists of:
/// - `term_type`: The operation to perform (e.g., FILTER, MAP, TABLE)
/// - `args`: Positional arguments (child terms)
/// - `optargs`: Named optional arguments
/// - `datum`: For DATUM terms, the actual value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Term {
    /// The type of this term
    pub term_type: TermType,
    
    /// Positional arguments
    pub args: Vec<Term>,
    
    /// Optional named arguments
    pub optargs: HashMap<String, Term>,
    
    /// Datum value (for Datum terms)
    pub datum: Option<Datum>,
}

impl Term {
    /// Create a new term with given type
    pub fn new(term_type: TermType) -> Self {
        Self {
            term_type,
            args: Vec::new(),
            optargs: HashMap::new(),
            datum: None,
        }
    }
    
    /// Create a datum term
    pub fn datum(datum: Datum) -> Self {
        Self {
            term_type: TermType::Datum,
            args: Vec::new(),
            optargs: HashMap::new(),
            datum: Some(datum),
        }
    }
    
    /// Add a positional argument
    pub fn with_arg(mut self, arg: Term) -> Self {
        self.args.push(arg);
        self
    }
    
    /// Add multiple positional arguments
    pub fn with_args(mut self, args: Vec<Term>) -> Self {
        self.args.extend(args);
        self
    }
    
    /// Add an optional named argument
    pub fn with_optarg<S: Into<String>>(mut self, name: S, value: Term) -> Self {
        self.optargs.insert(name.into(), value);
        self
    }
    
    /// Add multiple optional arguments
    pub fn with_optargs(mut self, optargs: HashMap<String, Term>) -> Self {
        self.optargs.extend(optargs);
        self
    }
    
    /// Get the first argument
    pub fn first_arg(&self) -> Option<&Term> {
        self.args.first()
    }
    
    /// Get argument at index
    pub fn arg(&self, index: usize) -> Option<&Term> {
        self.args.get(index)
    }
    
    /// Get optional argument by name
    pub fn optarg(&self, name: &str) -> Option<&Term> {
        self.optargs.get(name)
    }
    
    /// Check if this is a datum term
    pub fn is_datum(&self) -> bool {
        self.term_type == TermType::Datum
    }
    
    /// Get datum value if this is a datum term
    pub fn as_datum(&self) -> Option<&Datum> {
        self.datum.as_ref()
    }
    
    /// Pretty print the term tree
    pub fn pretty_print(&self, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        let mut result = format!("{}{}(", indent_str, self.term_type.name());
        
        if let Some(datum) = &self.datum {
            result.push_str(&format!("{:?}", datum));
        }
        
        if !self.args.is_empty() {
            result.push('\n');
            for (i, arg) in self.args.iter().enumerate() {
                result.push_str(&arg.pretty_print(indent + 1));
                if i < self.args.len() - 1 {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push_str(&indent_str);
        }
        
        if !self.optargs.is_empty() {
            result.push_str(" {");
            for (key, value) in &self.optargs {
                result.push_str(&format!("\n{}  {}: ", indent_str, key));
                result.push_str(&value.pretty_print(indent + 2));
            }
            result.push_str(&format!("\n{}}}", indent_str));
        }
        
        result.push(')');
        result
    }
}

/// Builder for creating ReQL terms fluently
pub struct TermBuilder {
    term: Term,
}

impl TermBuilder {
    /// Start building a new term
    pub fn new(term_type: TermType) -> Self {
        Self {
            term: Term::new(term_type),
        }
    }
    
    /// Add an argument
    pub fn arg(mut self, arg: Term) -> Self {
        self.term.args.push(arg);
        self
    }
    
    /// Add optional argument
    pub fn optarg<S: Into<String>>(mut self, name: S, value: Term) -> Self {
        self.term.optargs.insert(name.into(), value);
        self
    }
    
    /// Build the term
    pub fn build(self) -> Term {
        self.term
    }
}

// === Convenience constructors ===

impl Term {
    // Database operations
    pub fn db<S: Into<String>>(name: S) -> Self {
        Term::new(TermType::Db)
            .with_arg(Term::datum(Datum::String(name.into())))
    }
    
    pub fn table<S: Into<String>>(name: S) -> Self {
        Term::new(TermType::Table)
            .with_arg(Term::datum(Datum::String(name.into())))
    }
    
    pub fn db_list() -> Self {
        Term::new(TermType::DbList)
    }
    
    pub fn table_list() -> Self {
        Term::new(TermType::TableList)
    }
    
    // Data access
    pub fn get(table: Term, key: Datum) -> Self {
        Term::new(TermType::Get)
            .with_arg(table)
            .with_arg(Term::datum(key))
    }
    
    pub fn get_all(table: Term, keys: Vec<Datum>) -> Self {
        let key_terms: Vec<Term> = keys.into_iter()
            .map(Term::datum)
            .collect();
        
        Term::new(TermType::GetAll)
            .with_arg(table)
            .with_args(key_terms)
    }
    
    pub fn filter(sequence: Term, predicate: Term) -> Self {
        Term::new(TermType::Filter)
            .with_arg(sequence)
            .with_arg(predicate)
    }
    
    // Transformations
    pub fn map(sequence: Term, mapping: Term) -> Self {
        Term::new(TermType::Map)
            .with_arg(sequence)
            .with_arg(mapping)
    }
    
    pub fn order_by(sequence: Term, fields: Vec<Term>) -> Self {
        Term::new(TermType::OrderBy)
            .with_arg(sequence)
            .with_args(fields)
    }
    
    pub fn limit(sequence: Term, n: i64) -> Self {
        Term::new(TermType::Limit)
            .with_arg(sequence)
            .with_arg(Term::datum(Datum::Number(n as f64)))
    }
    
    pub fn skip(sequence: Term, n: i64) -> Self {
        Term::new(TermType::Skip)
            .with_arg(sequence)
            .with_arg(Term::datum(Datum::Number(n as f64)))
    }
    
    // Aggregations
    pub fn count(sequence: Term) -> Self {
        Term::new(TermType::Count)
            .with_arg(sequence)
    }
    
    pub fn sum(sequence: Term, field: Option<String>) -> Self {
        let mut term = Term::new(TermType::Sum)
            .with_arg(sequence);
        
        if let Some(f) = field {
            term = term.with_arg(Term::datum(Datum::String(f)));
        }
        
        term
    }
    
    pub fn avg(sequence: Term, field: Option<String>) -> Self {
        let mut term = Term::new(TermType::Avg)
            .with_arg(sequence);
        
        if let Some(f) = field {
            term = term.with_arg(Term::datum(Datum::String(f)));
        }
        
        term
    }
    
    // Write operations
    pub fn insert(table: Term, documents: Vec<Datum>) -> Self {
        let doc_terms: Vec<Term> = documents.into_iter()
            .map(Term::datum)
            .collect();
        
        Term::new(TermType::Insert)
            .with_arg(table)
            .with_args(doc_terms)
    }
    
    pub fn update(selection: Term, update_doc: Datum) -> Self {
        Term::new(TermType::Update)
            .with_arg(selection)
            .with_arg(Term::datum(update_doc))
    }
    
    pub fn delete(selection: Term) -> Self {
        Term::new(TermType::Delete)
            .with_arg(selection)
    }
    
    // Math operations
    pub fn add(terms: Vec<Term>) -> Self {
        Term::new(TermType::Add)
            .with_args(terms)
    }
    
    pub fn sub(terms: Vec<Term>) -> Self {
        Term::new(TermType::Sub)
            .with_args(terms)
    }
    
    pub fn mul(terms: Vec<Term>) -> Self {
        Term::new(TermType::Mul)
            .with_args(terms)
    }
    
    pub fn div(terms: Vec<Term>) -> Self {
        Term::new(TermType::Div)
            .with_args(terms)
    }
    
    // Logic operations
    pub fn eq(left: Term, right: Term) -> Self {
        Term::new(TermType::Eq)
            .with_arg(left)
            .with_arg(right)
    }
    
    pub fn ne(left: Term, right: Term) -> Self {
        Term::new(TermType::Ne)
            .with_arg(left)
            .with_arg(right)
    }
    
    pub fn lt(left: Term, right: Term) -> Self {
        Term::new(TermType::Lt)
            .with_arg(left)
            .with_arg(right)
    }
    
    pub fn gt(left: Term, right: Term) -> Self {
        Term::new(TermType::Gt)
            .with_arg(left)
            .with_arg(right)
    }
    
    pub fn and(terms: Vec<Term>) -> Self {
        Term::new(TermType::And)
            .with_args(terms)
    }
    
    pub fn or(terms: Vec<Term>) -> Self {
        Term::new(TermType::Or)
            .with_args(terms)
    }
    
    pub fn not(term: Term) -> Self {
        Term::new(TermType::Not)
            .with_arg(term)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_term_creation() {
        let term = Term::new(TermType::Db);
        assert_eq!(term.term_type, TermType::Db);
        assert!(term.args.is_empty());
    }
    
    #[test]
    fn test_datum_term() {
        let term = Term::datum(Datum::String("test".to_string()));
        assert!(term.is_datum());
        assert_eq!(term.as_datum().unwrap().as_string(), Some("test"));
    }
    
    #[test]
    fn test_db_term() {
        let term = Term::db("mydb");
        assert_eq!(term.term_type, TermType::Db);
        assert_eq!(term.args.len(), 1);
        
        let db_name = term.first_arg().unwrap();
        assert!(db_name.is_datum());
        assert_eq!(db_name.as_datum().unwrap().as_string(), Some("mydb"));
    }
    
    #[test]
    fn test_filter_term() {
        let table = Term::table("users");
        let predicate = Term::datum(Datum::Boolean(true));
        let filter = Term::filter(table, predicate);
        
        assert_eq!(filter.term_type, TermType::Filter);
        assert_eq!(filter.args.len(), 2);
    }
    
    #[test]
    fn test_builder() {
        let term = TermBuilder::new(TermType::Get)
            .arg(Term::table("users"))
            .arg(Term::datum(Datum::String("id123".to_string())))
            .optarg("read_mode", Term::datum(Datum::String("single".to_string())))
            .build();
        
        assert_eq!(term.term_type, TermType::Get);
        assert_eq!(term.args.len(), 2);
        assert!(term.optarg("read_mode").is_some());
    }
}
