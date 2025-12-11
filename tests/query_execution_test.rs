//! Simple end-to-end test for ReQL query execution

use rethinkdb::query::{QueryCompiler, QueryExecutor};
use rethinkdb::reql::{Datum, TermType};
use rethinkdb::storage::btree::SledStorage;
use rethinkdb::storage::Storage;
use std::sync::Arc;

#[tokio::test]
async fn test_query_compilation_and_execution() {
    // Create storage
    let temp_dir = std::env::temp_dir().join(format!("photondb_test_{}", std::process::id()));
    let storage = Arc::new(Storage::new(Box::new(
        SledStorage::new(temp_dir.to_str().unwrap()).expect("Failed to create storage"),
    )));

    // Create executor
    let executor = QueryExecutor::new(storage);

    // Test 1: DB_LIST query
    // JSON: [79] (DB_LIST has term_type 79)
    let query_json = serde_json::json!([79]);
    let term = QueryCompiler::compile(&query_json).expect("Failed to compile DB_LIST");
    assert_eq!(term.term_type, TermType::DbList);

    let result = executor.execute(&term).await.expect("Failed to execute DB_LIST");

    // Should return an array of database names
    match result {
        Datum::Array(dbs) => {
            assert!(!dbs.is_empty(), "Should have at least one database");
            println!("✓ DB_LIST query returned {} databases", dbs.len());
        }
        _ => panic!("Expected array result from DB_LIST, got: {:?}", result),
    }

    // Test 2: Simple number datum
    // Wire protocol format: [TermType, value]
    // For DATUM (type 0): [0, 42]
    let datum_query = serde_json::json!([0, 42]);
    let datum_term = QueryCompiler::compile(&datum_query).expect("Failed to compile DATUM");

    let datum_result = executor
        .execute(&datum_term)
        .await
        .expect("Failed to execute DATUM");

    match datum_result {
        Datum::Number(n) => {
            assert_eq!(n, 42.0, "Expected 42");
            println!("✓ DATUM query returned: {}", n);
        }
        _ => panic!("Expected number from datum, got: {:?}", datum_result),
    }

    // Test 3: Math operation (ADD)
    // Wire format: [TermType, [args...]]
    // ADD is type 20, with DATUM(10) and DATUM(5) as arguments
    let add_query = serde_json::json!([20, [[0, 10], [0, 5]]]);
    let add_term = QueryCompiler::compile(&add_query).expect("Failed to compile ADD");

    let add_result = executor
        .execute(&add_term)
        .await
        .expect("Failed to execute ADD");

    match add_result {
        Datum::Number(n) => {
            assert_eq!(n, 15.0, "10 + 5 should equal 15");
            println!("✓ ADD(10, 5) = {}", n);
        }
        _ => panic!("Expected number from ADD query, got: {:?}", add_result),
    }

    println!("\n✓ All query execution tests passed!");
}

#[tokio::test]
async fn test_datum_to_json_conversion() {
    // Test bidirectional conversion using direct datum creation
    let temp_dir = std::env::temp_dir().join(format!("photondb_test2_{}", std::process::id()));
    let storage = Arc::new(Storage::new(Box::new(
        SledStorage::new(temp_dir.to_str().unwrap()).expect("Failed to create storage"),
    )));
    let executor = QueryExecutor::new(storage);

    // Create a datum directly
    use std::collections::HashMap;
    let test_datum = Datum::Object({
        let mut map = HashMap::new();
        map.insert("name".to_string(), Datum::String("Alice".to_string()));
        map.insert("age".to_string(), Datum::Number(30.0));
        map
    });

    // Convert to JSON
    let json = QueryCompiler::datum_to_json(&test_datum);
    
    // Parse JSON back via compiler (compile a DATUM term)
    // Wire format: [0, value] where value is the JSON object (type 0 = DATUM)
    let wrapped_query = serde_json::json!([0, json]);
    let term = QueryCompiler::compile(&wrapped_query).expect("Failed to compile");
    let datum_from_json = executor.execute(&term).await.expect("Failed to execute");

    match (&test_datum, &datum_from_json) {
        (Datum::Object(orig), Datum::Object(converted)) => {
            assert_eq!(orig.len(), converted.len(), "Object size mismatch");
            assert_eq!(
                orig.get("name"),
                converted.get("name"),
                "Name field mismatch"
            );
            assert_eq!(
                orig.get("age"),
                converted.get("age"),
                "Age field mismatch"
            );
            println!("✓ Round-trip conversion successful!");
        }
        _ => panic!("Round-trip types don't match: {:?} vs {:?}", test_datum, datum_from_json),
    }
}
