//! ReQL Term Types mapped from Cap'n Proto term.capnp.
//!
//! This module defines all RethinkDB query operations (terms) as an enum.
//! The discriminant values match the Cap'n Proto @-ordinals exactly to ensure
//! wire protocol compatibility.
//!
//! # Term Categories
//!
//! - **Core Data**: DATUM, MAKE_ARRAY, MAKE_OBJ
//! - **Database Operations**: DB, DB_CREATE, DB_DROP, DB_LIST
//! - **Table Operations**: TABLE, TABLE_CREATE, TABLE_DROP, TABLE_LIST
//! - **Data Access**: GET, GET_ALL, BETWEEN
//! - **Transformations**: FILTER, MAP, CONCAT_MAP, ORDER_BY, DISTINCT
//! - **Aggregations**: COUNT, SUM, AVG, MIN, MAX, GROUP, REDUCE
//! - **Math Operations**: ADD, SUB, MUL, DIV, MOD
//! - **Logic Operations**: EQ, NE, LT, LE, GT, GE, AND, OR, NOT
//! - **Array Operations**: APPEND, PREPEND, SLICE, INSERT_AT, DELETE_AT
//! - **Object Operations**: GET_FIELD, KEYS, VALUES, PLUCK, WITHOUT, MERGE
//! - **Control Flow**: BRANCH, FOR_EACH, FUNC
//! - **Type Operations**: TYPE_OF, COERCE_TO
//!
//! # Example
//!
//! ```rust,ignore
//! use rethinkdb::reql::TermType;
//!
//! // Convert from Cap'n Proto wire format
//! let term_type = TermType::from_u64(51).unwrap(); // MAP operation
//! assert_eq!(term_type, TermType::Map);
//! assert_eq!(term_type.name(), "MAP");
//! ```

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u64)]
pub enum TermType {
    // Core data types
    Datum = 0,
    MakeArray = 1,
    MakeObj = 2,
    
    // Variables
    Var = 3,
    
    // JavaScript evaluation
    Javascript = 4,
    
    // Database operations
    Db = 9,
    Table = 10,
    Get = 11,
    GetAll = 12,
    
    // Comparison operators
    Eq = 13,
    Ne = 14,
    Lt = 15,
    Le = 16,
    Gt = 17,
    Ge = 18,
    
    // Logic operators
    Not = 19,
    
    // Math operators
    Add = 20,
    Sub = 21,
    Mul = 22,
    Div = 23,
    Mod = 24,
    
    // Array/Set operations
    Append = 28,
    Prepend = 29,
    Difference = 30,
    SetInsert = 31,
    SetIntersection = 32,
    SetUnion = 33,
    SetDifference = 34,
    
    // Sequence operations
    Slice = 35,
    Skip = 36,
    Limit = 37,
    Contains = 39,
    
    // Object operations
    GetField = 40,
    Keys = 41,
    Values = 42,
    HasFields = 44,
    Pluck = 46,
    Without = 47,
    Merge = 48,
    
    // Data access
    Between = 49,
    
    // Aggregations & transformations
    Reduce = 50,
    Map = 51,
    Filter = 53,
    ConcatMap = 54,
    OrderBy = 55,
    Distinct = 56,
    Count = 57,
    Nth = 60,
    
    // Array mutations
    InsertAt = 67,
    DeleteAt = 68,
    ChangeAt = 69,
    SpliceAt = 70,
    
    // Type operations
    CoerceTo = 71,
    TypeOf = 72,
    
    // Write operations
    Update = 73,
    Delete = 74,
    Replace = 75,
    Insert = 76,
    
    // Database admin
    DbCreate = 77,
    DbDrop = 78,
    DbList = 79,
    
    // Table admin
    TableCreate = 80,
    TableDrop = 81,
    TableList = 82,
    
    // Control flow
    Branch = 99,
    Or = 100,
    And = 101,
    ForEach = 102,
    Func = 103,  // Renamed from FuncCall to match Cap'n Proto
    
    // Grouping & aggregations (higher numbers)
    Group = 152,
    Sum = 153,
    Avg = 154,
    Min = 155,
    Max = 156,
}

impl TermType {
    /// Converts from a u64 term type ID (from Cap'n Proto wire protocol).
    ///
    /// # Arguments
    ///
    /// * `value` - The numeric term type ID from the wire protocol
    ///
    /// # Returns
    ///
    /// * `Some(TermType)` - If the value maps to a known term type
    /// * `None` - If the value is unknown/unsupported
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let term_type = TermType::from_u64(13).unwrap();
    /// assert_eq!(term_type, TermType::Eq);
    /// ```
    pub fn from_u64(value: u64) -> Option<Self> {
        match value {
            0 => Some(TermType::Datum),
            1 => Some(TermType::MakeArray),
            2 => Some(TermType::MakeObj),
            3 => Some(TermType::Var),
            4 => Some(TermType::Javascript),
            9 => Some(TermType::Db),
            10 => Some(TermType::Table),
            11 => Some(TermType::Get),
            12 => Some(TermType::GetAll),
            13 => Some(TermType::Eq),
            14 => Some(TermType::Ne),
            15 => Some(TermType::Lt),
            16 => Some(TermType::Le),
            17 => Some(TermType::Gt),
            18 => Some(TermType::Ge),
            19 => Some(TermType::Not),
            20 => Some(TermType::Add),
            21 => Some(TermType::Sub),
            22 => Some(TermType::Mul),
            23 => Some(TermType::Div),
            24 => Some(TermType::Mod),
            28 => Some(TermType::Append),
            29 => Some(TermType::Prepend),
            30 => Some(TermType::Difference),
            31 => Some(TermType::SetInsert),
            32 => Some(TermType::SetIntersection),
            33 => Some(TermType::SetUnion),
            34 => Some(TermType::SetDifference),
            35 => Some(TermType::Slice),
            36 => Some(TermType::Skip),
            37 => Some(TermType::Limit),
            39 => Some(TermType::Contains),
            40 => Some(TermType::GetField),
            41 => Some(TermType::Keys),
            42 => Some(TermType::Values),
            44 => Some(TermType::HasFields),
            46 => Some(TermType::Pluck),
            47 => Some(TermType::Without),
            48 => Some(TermType::Merge),
            49 => Some(TermType::Between),
            50 => Some(TermType::Reduce),
            51 => Some(TermType::Map),
            53 => Some(TermType::Filter),
            54 => Some(TermType::ConcatMap),
            55 => Some(TermType::OrderBy),
            56 => Some(TermType::Distinct),
            57 => Some(TermType::Count),
            60 => Some(TermType::Nth),
            67 => Some(TermType::InsertAt),
            68 => Some(TermType::DeleteAt),
            69 => Some(TermType::ChangeAt),
            70 => Some(TermType::SpliceAt),
            71 => Some(TermType::CoerceTo),
            72 => Some(TermType::TypeOf),
            73 => Some(TermType::Update),
            74 => Some(TermType::Delete),
            75 => Some(TermType::Replace),
            76 => Some(TermType::Insert),
            77 => Some(TermType::DbCreate),
            78 => Some(TermType::DbDrop),
            79 => Some(TermType::DbList),
            80 => Some(TermType::TableCreate),
            81 => Some(TermType::TableDrop),
            82 => Some(TermType::TableList),
            99 => Some(TermType::Branch),
            100 => Some(TermType::Or),
            101 => Some(TermType::And),
            102 => Some(TermType::ForEach),
            103 => Some(TermType::Func),
            152 => Some(TermType::Group),
            153 => Some(TermType::Sum),
            154 => Some(TermType::Avg),
            155 => Some(TermType::Min),
            156 => Some(TermType::Max),
            _ => None,
        }
    }
    
    /// Converts to u64 term type ID (for Cap'n Proto wire protocol).
    ///
    /// # Returns
    ///
    /// The numeric term type ID as specified in the Cap'n Proto schema.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// assert_eq!(TermType::Filter.to_u64(), 53);
    /// ```
    pub fn to_u64(self) -> u64 {
        self as u64
    }
    
    /// Returns the term type name as a string constant.
    ///
    /// This is useful for debugging, logging, and error messages.
    ///
    /// # Returns
    ///
    /// A static string containing the uppercase term name (e.g., "FILTER", "MAP").
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// assert_eq!(TermType::Filter.name(), "FILTER");
    /// ```
    pub fn name(&self) -> &'static str {
        match self {
            TermType::Datum => "DATUM",
            TermType::MakeArray => "MAKE_ARRAY",
            TermType::MakeObj => "MAKE_OBJ",
            TermType::Var => "VAR",
            TermType::Javascript => "JAVASCRIPT",
            TermType::Db => "DB",
            TermType::Table => "TABLE",
            TermType::Get => "GET",
            TermType::GetAll => "GET_ALL",
            TermType::Eq => "EQ",
            TermType::Ne => "NE",
            TermType::Lt => "LT",
            TermType::Le => "LE",
            TermType::Gt => "GT",
            TermType::Ge => "GE",
            TermType::Not => "NOT",
            TermType::Add => "ADD",
            TermType::Sub => "SUB",
            TermType::Mul => "MUL",
            TermType::Div => "DIV",
            TermType::Mod => "MOD",
            TermType::Append => "APPEND",
            TermType::Prepend => "PREPEND",
            TermType::Difference => "DIFFERENCE",
            TermType::SetInsert => "SET_INSERT",
            TermType::SetIntersection => "SET_INTERSECTION",
            TermType::SetUnion => "SET_UNION",
            TermType::SetDifference => "SET_DIFFERENCE",
            TermType::Slice => "SLICE",
            TermType::Skip => "SKIP",
            TermType::Limit => "LIMIT",
            TermType::Contains => "CONTAINS",
            TermType::GetField => "GET_FIELD",
            TermType::Keys => "KEYS",
            TermType::Values => "VALUES",
            TermType::HasFields => "HAS_FIELDS",
            TermType::Pluck => "PLUCK",
            TermType::Without => "WITHOUT",
            TermType::Merge => "MERGE",
            TermType::Between => "BETWEEN",
            TermType::Reduce => "REDUCE",
            TermType::Map => "MAP",
            TermType::Filter => "FILTER",
            TermType::ConcatMap => "CONCAT_MAP",
            TermType::OrderBy => "ORDER_BY",
            TermType::Distinct => "DISTINCT",
            TermType::Count => "COUNT",
            TermType::Nth => "NTH",
            TermType::InsertAt => "INSERT_AT",
            TermType::DeleteAt => "DELETE_AT",
            TermType::ChangeAt => "CHANGE_AT",
            TermType::SpliceAt => "SPLICE_AT",
            TermType::CoerceTo => "COERCE_TO",
            TermType::TypeOf => "TYPE_OF",
            TermType::Update => "UPDATE",
            TermType::Delete => "DELETE",
            TermType::Replace => "REPLACE",
            TermType::Insert => "INSERT",
            TermType::DbCreate => "DB_CREATE",
            TermType::DbDrop => "DB_DROP",
            TermType::DbList => "DB_LIST",
            TermType::TableCreate => "TABLE_CREATE",
            TermType::TableDrop => "TABLE_DROP",
            TermType::TableList => "TABLE_LIST",
            TermType::Branch => "BRANCH",
            TermType::Or => "OR",
            TermType::And => "AND",
            TermType::ForEach => "FOR_EACH",
            TermType::Func => "FUNC",
            TermType::Group => "GROUP",
            TermType::Sum => "SUM",
            TermType::Avg => "AVG",
            TermType::Min => "MIN",
            TermType::Max => "MAX",
        }
    }
}

impl std::fmt::Display for TermType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_term_type_conversion() {
        assert_eq!(TermType::from_u64(0), Some(TermType::Datum));
        assert_eq!(TermType::from_u64(1), Some(TermType::MakeArray));
        assert_eq!(TermType::from_u64(13), Some(TermType::Eq));
        assert_eq!(TermType::from_u64(999), None);
    }

    #[test]
    fn test_term_type_to_u64() {
        assert_eq!(TermType::Datum.to_u64(), 0);
        assert_eq!(TermType::MakeArray.to_u64(), 1);
        assert_eq!(TermType::Eq.to_u64(), 13);
    }

    #[test]
    fn test_term_type_names() {
        assert_eq!(TermType::Datum.name(), "DATUM");
        assert_eq!(TermType::Filter.name(), "FILTER");
        assert_eq!(TermType::Insert.name(), "INSERT");
    }
}
