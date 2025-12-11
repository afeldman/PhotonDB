//! ReQL Query Compiler.
//!
//! Compiles JSON query format to ReQL AST for execution.
//!
//! # Wire Protocol Format
//!
//! RethinkDB queries are transmitted as JSON arrays in the format:
//! ```json
//! [term_type, [arg1, arg2, ...], {"optarg1": value1, ...}]
//! ```
//!
//! Where:
//! - `term_type` is a numeric ID (u64) identifying the operation
//! - Second element is an array of positional arguments
//! - Third element (optional) is an object of named arguments
//!
//! # Example
//!
//! JSON query for `r.table("users").filter({active: true})`:
//!
//! ```json
//! [53,  // FILTER
//!   [
//!     [15, ["users"]],  // TABLE("users")
//!     [1, [{"active": true}]]  // MAKE_OBJ({active: true})
//!   ]
//! ]
//! ```

use crate::reql::{Datum, Term, TermType};
use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::collections::HashMap;

/// ReQL Query Compiler for parsing JSON queries into AST.
/// 
/// Parses JSON-encoded queries from the wire protocol into our AST representation.
/// RethinkDB wire protocol encodes queries as JSON arrays:
/// `[term_type, [args...], {optargs...}]`
pub struct QueryCompiler;

impl QueryCompiler {
    /// Compile a JSON value into a ReQL Term
    pub fn compile(query: &Value) -> Result<Term> {
        Self::compile_term(query)
    }
    
    /// Compile a term from JSON
    fn compile_term(json: &Value) -> Result<Term> {
        // Check if this is a datum (primitive value)
        if !json.is_array() {
            return Ok(Term::datum(Self::json_to_datum(json)?));
        }
        
        let arr = json.as_array()
            .ok_or_else(|| anyhow!("Expected array for term"))?;
        
        if arr.is_empty() {
            return Err(anyhow!("Empty term array"));
        }
        
        // Parse term type
        let term_type_num = arr[0].as_u64()
            .ok_or_else(|| anyhow!("Invalid term type: expected number, got {:?}", arr[0]))?;
        
        let term_type = TermType::from_u64(term_type_num)
            .ok_or_else(|| anyhow!("Unknown term type: {}", term_type_num))?;
        
        // Handle Datum terms specially
        if term_type == TermType::Datum {
            if arr.len() < 2 {
                return Err(anyhow!("DATUM term requires value argument"));
            }
            return Ok(Term::datum(Self::json_to_datum(&arr[1])?));
        }
        
        // Parse args (index 1)
        let args = if arr.len() > 1 {
            if let Some(args_array) = arr[1].as_array() {
                args_array.iter()
                    .map(|arg| Self::compile_term(arg))
                    .collect::<Result<Vec<_>>>()
                    .context("Failed to parse term arguments")?
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        
        // Parse optargs (index 2)
        let optargs = if arr.len() > 2 {
            if let Some(optargs_obj) = arr[2].as_object() {
                optargs_obj.iter()
                    .map(|(key, value)| {
                        Self::compile_term(value)
                            .map(|term| (key.clone(), term))
                    })
                    .collect::<Result<HashMap<String, Term>>>()
                    .context("Failed to parse optional arguments")?
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };
        
        Ok(Term::new(term_type)
            .with_args(args)
            .with_optargs(optargs))
    }
    
    /// Convert JSON value to Datum
    fn json_to_datum(json: &Value) -> Result<Datum> {
        match json {
            Value::Null => Ok(Datum::Null),
            Value::Bool(b) => Ok(Datum::Boolean(*b)),
            Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    Ok(Datum::Number(f))
                } else {
                    Err(anyhow!("Invalid number: {}", n))
                }
            }
            Value::String(s) => Ok(Datum::String(s.clone())),
            Value::Array(arr) => {
                let datums: Result<Vec<Datum>> = arr.iter()
                    .map(Self::json_to_datum)
                    .collect();
                Ok(Datum::Array(datums?))
            }
            Value::Object(obj) => {
                let mut datum_obj = HashMap::new();
                for (key, value) in obj {
                    datum_obj.insert(key.clone(), Self::json_to_datum(value)?);
                }
                Ok(Datum::Object(datum_obj))
            }
        }
    }
    
    /// Convert Datum to JSON value
    pub fn datum_to_json(datum: &Datum) -> Value {
        match datum {
            Datum::Null => Value::Null,
            Datum::Boolean(b) => Value::Bool(*b),
            Datum::Number(n) => {
                serde_json::Number::from_f64(*n)
                    .map(Value::Number)
                    .unwrap_or(Value::Null)
            }
            Datum::String(s) => Value::String(s.clone()),
            Datum::Array(arr) => {
                Value::Array(arr.iter().map(Self::datum_to_json).collect())
            }
            Datum::Object(obj) => {
                let json_obj: serde_json::Map<String, Value> = obj.iter()
                    .map(|(k, v)| (k.clone(), Self::datum_to_json(v)))
                    .collect();
                Value::Object(json_obj)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compile_datum() {
        // Simple string datum
        let json = serde_json::json!("hello");
        let term = QueryCompiler::compile(&json).unwrap();
        
        assert!(term.is_datum());
        assert_eq!(term.as_datum().unwrap().as_string(), Some("hello"));
    }
    
    #[test]
    fn test_compile_db_list() {
        // DB_LIST: [59]
        let json = serde_json::json!([59]);
        let term = QueryCompiler::compile(&json).unwrap();
        
        assert_eq!(term.term_type, TermType::DbList);
        assert!(term.args.is_empty());
    }
    
    #[test]
    fn test_compile_db() {
        // DB("test"): [14, ["test"]]
        let json = serde_json::json!([14, ["test"]]);
        let term = QueryCompiler::compile(&json).unwrap();
        
        assert_eq!(term.term_type, TermType::Db);
        assert_eq!(term.args.len(), 1);
        
        let db_name = term.first_arg().unwrap();
        assert!(db_name.is_datum());
        assert_eq!(db_name.as_datum().unwrap().as_string(), Some("test"));
    }
    
    #[test]
    fn test_compile_table() {
        // r.table("users"): [15, ["users"]]
        let json = serde_json::json!([15, ["users"]]);
        let term = QueryCompiler::compile(&json).unwrap();
        
        assert_eq!(term.term_type, TermType::Table);
        assert_eq!(term.args.len(), 1);
    }
    
    #[test]
    fn test_compile_filter() {
        // r.table("users").filter({age: 25})
        // [39, [[15, ["users"]], {age: 25}]]
        let json = serde_json::json!([
            39, // FILTER
            [
                [15, ["users"]], // TABLE
                {"age": 25}      // predicate (as datum)
            ]
        ]);
        
        let term = QueryCompiler::compile(&json).unwrap();
        
        assert_eq!(term.term_type, TermType::Filter);
        assert_eq!(term.args.len(), 2);
        
        // First arg is table
        let table = term.arg(0).unwrap();
        assert_eq!(table.term_type, TermType::Table);
        
        // Second arg is predicate
        let predicate = term.arg(1).unwrap();
        assert!(predicate.is_datum());
    }
    
    #[test]
    fn test_compile_with_optargs() {
        // r.table("users", {use_outdated: true})
        // [15, ["users"], {use_outdated: true}]
        let json = serde_json::json!([
            15,
            ["users"],
            {"use_outdated": true}
        ]);
        
        let term = QueryCompiler::compile(&json).unwrap();
        
        assert_eq!(term.term_type, TermType::Table);
        assert_eq!(term.args.len(), 1);
        assert!(term.optarg("use_outdated").is_some());
        
        let use_outdated = term.optarg("use_outdated").unwrap();
        assert!(use_outdated.is_datum());
        assert_eq!(use_outdated.as_datum().unwrap().as_bool(), Some(true));
    }
    
    #[test]
    fn test_compile_nested() {
        // r.table("users").filter({age: 25}).count()
        // [43, [[39, [[15, ["users"]], {age: 25}]]]]
        let json = serde_json::json!([
            43, // COUNT
            [
                [39, [[15, ["users"]], {"age": 25}]] // FILTER
            ]
        ]);
        
        let term = QueryCompiler::compile(&json).unwrap();
        
        assert_eq!(term.term_type, TermType::Count);
        assert_eq!(term.args.len(), 1);
        
        let filter = term.first_arg().unwrap();
        assert_eq!(filter.term_type, TermType::Filter);
    }
    
    #[test]
    fn test_json_to_datum_object() {
        let json = serde_json::json!({
            "name": "Alice",
            "age": 30,
            "active": true
        });
        
        let datum = QueryCompiler::json_to_datum(&json).unwrap();
        
        if let Datum::Object(obj) = datum {
            assert_eq!(obj.get("name").and_then(|d| d.as_string()), Some("Alice"));
            assert_eq!(obj.get("age").and_then(|d| d.as_number()), Some(30.0));
            assert_eq!(obj.get("active").and_then(|d| d.as_bool()), Some(true));
        } else {
            panic!("Expected Datum::Object");
        }
    }
    
    #[test]
    fn test_datum_to_json() {
        let datum = Datum::Object({
            let mut map = HashMap::new();
            map.insert("name".to_string(), Datum::String("Bob".to_string()));
            map.insert("age".to_string(), Datum::Number(25.0));
            map
        });
        
        let json = QueryCompiler::datum_to_json(&datum);
        
        assert_eq!(json["name"], "Bob");
        assert_eq!(json["age"], 25.0);
    }
}
