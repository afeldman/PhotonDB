//! Benchmarks for Phase 4 optimizations

#[cfg(test)]
mod bench {
    use crate::storage::slab::{CompressionAlgorithm, SlabStorage};
    use std::time::Instant;

    /// Benchmark compression performance
    #[test]
    fn bench_compression_vs_none() {
        let temp_dir = std::env::temp_dir().join("slab_bench_compression");

        // Test data: 1KB of repetitive text (should compress well)
        let data = "The quick brown fox jumps over the lazy dog. ".repeat(20);
        let test_data = data.as_bytes();

        // Without compression
        {
            let storage = SlabStorage::with_options(
                temp_dir.join("none"),
                Some(64),
                Some(8192),
                CompressionAlgorithm::None,
                1000,
            )
            .unwrap();

            let start = Instant::now();
            for i in 0..1000 {
                let key = format!("key_{}", i);
                storage.set(key.as_bytes(), test_data).unwrap();
            }
            let elapsed = start.elapsed();
            println!("Without compression: {:?} for 1000 writes", elapsed);
        }

        // With compression
        {
            let storage = SlabStorage::with_options(
                temp_dir.join("zstd"),
                Some(64),
                Some(8192),
                CompressionAlgorithm::Zstd,
                1000,
            )
            .unwrap();

            let start = Instant::now();
            for i in 0..1000 {
                let key = format!("key_{}", i);
                storage.set(key.as_bytes(), test_data).unwrap();
            }
            let elapsed = start.elapsed();
            println!("With zstd compression: {:?} for 1000 writes", elapsed);
        }

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    /// Benchmark cache performance
    #[test]
    fn bench_cache_hit_rate() {
        let temp_dir = std::env::temp_dir().join("slab_bench_cache");

        let storage = SlabStorage::with_options(
            &temp_dir,
            Some(64),
            Some(8192),
            CompressionAlgorithm::Zstd,
            100, // Small cache for testing eviction
        )
        .unwrap();

        // Write 1000 keys
        for i in 0..1000 {
            let key = format!("key_{}", i);
            let value = format!("value_{}", i);
            storage.set(key.as_bytes(), value.as_bytes()).unwrap();
        }

        // Read first 100 keys multiple times (should be cached)
        let start = Instant::now();
        for _ in 0..10 {
            for i in 0..100 {
                let key = format!("key_{}", i);
                storage.get(key.as_bytes()).unwrap();
            }
        }
        let elapsed = start.elapsed();

        let stats = storage.stats();
        println!(
            "Cache stats: {} hits, {} misses, {:.2}% hit rate",
            stats.cache_hits,
            stats.cache_misses,
            stats.cache_hit_rate * 100.0
        );
        println!("Read 1000 cached entries: {:?}", elapsed);

        // Verify high hit rate
        assert!(stats.cache_hit_rate > 0.8, "Expected >80% cache hit rate");

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    /// Benchmark parallel metadata batch writes
    #[test]
    fn bench_parallel_batch_writes() {
        use crate::storage::slab::{MetadataStore, SlotId};

        let temp_dir = std::env::temp_dir().join("slab_bench_parallel");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let metadata = MetadataStore::new(&temp_dir).unwrap();

        // Small batch (< 100 entries) - sequential
        let small_batch: Vec<_> = (0..50)
            .map(|i| (format!("key_{}", i).into_bytes(), SlotId::new(0, i as u64)))
            .collect();

        let start = Instant::now();
        metadata.write_batch(small_batch).unwrap();
        let small_elapsed = start.elapsed();
        println!("Small batch (50 entries): {:?}", small_elapsed);

        // Large batch (> 100 entries) - parallel with Rayon
        let large_batch: Vec<_> = (0..500)
            .map(|i| {
                (
                    format!("large_key_{}", i).into_bytes(),
                    SlotId::new(0, i as u64),
                )
            })
            .collect();

        let start = Instant::now();
        metadata.write_batch(large_batch).unwrap();
        let large_elapsed = start.elapsed();
        println!("Large batch (500 entries, parallel): {:?}", large_elapsed);

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }
}
