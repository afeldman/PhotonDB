//! ReQL (RethinkDB Query Language) implementation.
//!
//! This module provides a complete implementation of RethinkDB's query language,
//! including:
//!
//! - **Term Types**: All 70+ ReQL operations (DB, TABLE, FILTER, MAP, etc.)
//! - **AST**: Abstract Syntax Tree for representing queries
//! - **Datum**: JSON-like data type for values
//! - **Protocol**: Wire protocol definitions
//!
//! # Architecture
//!
//! The ReQL implementation follows a three-layer design:
//!
//! 1. **Terms Layer** (`terms.rs`): Defines all operation types as an enum
//! 2. **AST Layer** (`ast.rs`): Represents query structure with Term nodes
//! 3. **Execution Layer** (`query::executor`): Executes AST and returns results
//!
//! # Example
//!
//! ```rust,ignore
//! use rethinkdb::reql::{Term, TermType, Datum};
//!
//! // Build a query: r.db("test").table("users")
//! let query = Term::new(TermType::Table)
//!     .with_arg(Term::new(TermType::Db)
//!         .with_arg(Term::datum(Datum::String("test".into()))))
//!     .with_arg(Term::datum(Datum::String("users".into())));
//! ```

pub mod ast;
pub mod datum;
pub mod protocol;
pub mod terms;
pub mod types;

pub use ast::{Term, TermBuilder};
pub use datum::Datum;
pub use terms::TermType;
pub use types::*;
