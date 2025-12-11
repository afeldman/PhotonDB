//! Datum - RethinkDB's JSON-like data type.
//!
//! A `Datum` represents any value that can be stored or manipulated in RethinkDB.
//! It's similar to JSON but designed specifically for database operations.
//!
//! # Supported Types
//!
//! - **Null**: Absence of a value
//! - **Boolean**: true or false
//! - **Number**: f64 floating point numbers
//! - **String**: UTF-8 encoded text
//! - **Array**: Ordered list of datums
//! - **Object**: Key-value map (like JSON object)
//!
//! # Example
//!
//! ```rust,ignore
//! use rethinkdb::reql::Datum;
//! use std::collections::HashMap;
//!
//! let null_val = Datum::Null;
//! let bool_val = Datum::Boolean(true);
//! let num_val = Datum::Number(42.5);
//! let str_val = Datum::String("hello".into());
//! let arr_val = Datum::Array(vec![num_val.clone(), str_val.clone()]);
//!
//! let mut obj = HashMap::new();
//! obj.insert("name".to_string(), Datum::String("Alice".into()));
//! obj.insert("age".to_string(), Datum::Number(30.0));
//! let obj_val = Datum::Object(obj);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Datum represents a value in RethinkDB.
///
/// This is the fundamental data type for all values stored and manipulated
/// in RethinkDB queries. It's JSON-compatible with serde serialization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Datum {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Datum>),
    Object(HashMap<String, Datum>),
}

impl Datum {
    /// Check if datum is null
    pub fn is_null(&self) -> bool {
        matches!(self, Datum::Null)
    }

    /// Get as string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Datum::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as number
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Datum::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get as boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Datum::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as array
    pub fn as_array(&self) -> Option<&Vec<Datum>> {
        match self {
            Datum::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Get as object
    pub fn as_object(&self) -> Option<&HashMap<String, Datum>> {
        match self {
            Datum::Object(obj) => Some(obj),
            _ => None,
        }
    }
}

// Conversions
impl From<bool> for Datum {
    fn from(b: bool) -> Self {
        Datum::Boolean(b)
    }
}

impl From<i32> for Datum {
    fn from(n: i32) -> Self {
        Datum::Number(n as f64)
    }
}

impl From<f64> for Datum {
    fn from(n: f64) -> Self {
        Datum::Number(n)
    }
}

impl From<String> for Datum {
    fn from(s: String) -> Self {
        Datum::String(s)
    }
}

impl From<&str> for Datum {
    fn from(s: &str) -> Self {
        Datum::String(s.to_string())
    }
}

impl From<serde_json::Value> for Datum {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Datum::Null,
            serde_json::Value::Bool(b) => Datum::Boolean(b),
            serde_json::Value::Number(n) => {
                Datum::Number(n.as_f64().unwrap_or(0.0))
            }
            serde_json::Value::String(s) => Datum::String(s),
            serde_json::Value::Array(arr) => {
                Datum::Array(arr.into_iter().map(Datum::from).collect())
            }
            serde_json::Value::Object(obj) => {
                Datum::Object(
                    obj.into_iter()
                        .map(|(k, v)| (k, Datum::from(v)))
                        .collect(),
                )
            }
        }
    }
}

impl From<Datum> for serde_json::Value {
    fn from(datum: Datum) -> Self {
        match datum {
            Datum::Null => serde_json::Value::Null,
            Datum::Boolean(b) => serde_json::Value::Bool(b),
            Datum::Number(n) => {
                serde_json::Value::Number(
                    serde_json::Number::from_f64(n).unwrap_or_else(|| serde_json::Number::from(0))
                )
            }
            Datum::String(s) => serde_json::Value::String(s),
            Datum::Array(arr) => {
                serde_json::Value::Array(arr.into_iter().map(serde_json::Value::from).collect())
            }
            Datum::Object(obj) => {
                serde_json::Value::Object(
                    obj.into_iter()
                        .map(|(k, v)| (k, serde_json::Value::from(v)))
                        .collect(),
                )
            }
        }
    }
}

impl std::fmt::Display for Datum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Datum::Null => write!(f, "null"),
            Datum::Boolean(b) => write!(f, "{}", b),
            Datum::Number(n) => write!(f, "{}", n),
            Datum::String(s) => write!(f, "\"{}\"", s),
            Datum::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Datum::Object(obj) => {
                write!(f, "{{")?;
                for (i, (key, value)) in obj.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "\"{}\": {}", key, value)?;
                }
                write!(f, "}}")
            }
        }
    }
}
