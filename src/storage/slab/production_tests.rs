//! Production integration tests for Slab Storage
//!
//! These tests verify real-world usage scenarios using SlabStorage directly

#[cfg(test)]
mod integration {
    use crate::storage::slab::SlabStorage;

    #[test]
    fn test_production_basic_operations() -> crate::error::Result<()> {
        let temp_dir = std::env::temp_dir().join("slab_prod_basic");

        let storage = SlabStorage::new(&temp_dir, Some(64), Some(8192))?;

        // Write 100 key-value pairs
        for i in 0..100 {
            let key = format!("user:{}", i);
            let value = format!("{{\"id\":{},\"name\":\"User {}\"}}", i, i);
            storage.set(key.as_bytes(), value.as_bytes())?;
        }

        // Read back
        let value = storage.get(b"user:42")?.expect("User 42 should exist");
        let value_str = String::from_utf8(value).unwrap();
        assert!(value_str.contains("\"id\":42"));
        assert!(value_str.contains("User 42"));

        // Update
        storage.set(b"user:42", b"{\"id\":42,\"name\":\"Updated User\"}")?;
        let updated = storage.get(b"user:42")?.expect("Updated user should exist");
        let updated_str = String::from_utf8(updated).unwrap();
        assert!(updated_str.contains("Updated User"));

        // Delete
        storage.delete(b"user:0")?;
        assert!(storage.get(b"user:0")?.is_none());

        // Verify count
        assert_eq!(storage.len(), 99); // 100 - 1 deleted

        // Get stats
        let stats = storage.stats();
        println!("Production basic stats: cache hit rate = {:.2}%", stats.cache_hit_rate * 100.0);

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();

        Ok(())
    }

    #[test]
    fn test_production_persistence() -> crate::error::Result<()> {
        let temp_dir = std::env::temp_dir().join("slab_prod_persist");

        // Phase 1: Write data
        {
            let storage = SlabStorage::new(&temp_dir, Some(64), Some(8192))?;

            for i in 0..50 {
                let key = format!("doc:{}", i);
                let value = format!("{{\"data\":\"Value {}\"}}", i);
                storage.set(key.as_bytes(), value.as_bytes())?;
            }

            storage.flush()?;
        }

        // Phase 2: Reopen and verify
        {
            let storage = SlabStorage::new(&temp_dir, Some(64), Some(8192))?;

            assert_eq!(storage.len(), 50);

            let value = storage.get(b"doc:25")?.expect("Document should persist");
            let value_str = String::from_utf8(value).unwrap();
            assert!(value_str.contains("Value 25"));
        }

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();

        Ok(())
    }

    #[test]
    fn test_production_stress() -> crate::error::Result<()> {
        let temp_dir = std::env::temp_dir().join("slab_prod_stress");

        let storage = SlabStorage::new(&temp_dir, Some(64), Some(16384))?;

        // Write 500 documents with varying sizes
        for i in 0..500 {
            let size = 100 + (i % 5) * 100; // 100B to 500B
            let content = "x".repeat(size);
            let value = format!("{{\"id\":{},\"content\":\"{}\"}}", i, content);
            storage.set(format!("doc:{}", i).as_bytes(), value.as_bytes())?;
        }

        // Verify random reads
        for i in [0, 100, 250, 400, 499] {
            let value = storage
                .get(format!("doc:{}", i).as_bytes())?
                .expect(&format!("Document {} should exist", i));
            let value_str = String::from_utf8(value).unwrap();
            assert!(value_str.contains(&format!("\"id\":{}", i)));
        }

        // Get storage stats
        let stats = storage.stats();
        println!("Stress test completed: {} keys, {:.2}% cache hit rate",
                 stats.key_count, stats.cache_hit_rate * 100.0);

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();

        Ok(())
    }

    #[test]
    fn test_production_large_values() -> crate::error::Result<()> {
        let temp_dir = std::env::temp_dir().join("slab_prod_large");

        let storage = SlabStorage::new(&temp_dir, Some(64), Some(65536))?; // 64KB max

        // Write large values (up to 32KB)
        for i in 0..20 {
            let size = 1024 * (i + 1); // 1KB to 20KB
            let content = "A".repeat(size);
            let value = format!("{{\"data\":\"{}\"}}", content);
            storage.set(format!("large:{}", i).as_bytes(), value.as_bytes())?;
        }

        // Verify large values
        let large = storage.get(b"large:19")?.expect("Large value should exist");
        assert!(large.len() > 19 * 1024);

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();

        Ok(())
    }
}
