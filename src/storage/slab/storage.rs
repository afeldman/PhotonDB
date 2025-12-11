//! Complete slab-based storage engine with Phase 4 optimizations
//!
//! Combines SlabAllocator + MetadataStore + Compression + Cache
//! This is the integration layer that provides the high-level API.
//!
//! # Phase 4 Features
//! - Transparent zstd compression
//! - LRU cache for hot data
//! - Cache statistics

use super::allocator::SlabAllocator;
use super::cache::SlabCache;
use super::compression::{compress, decompress, CompressionAlgorithm};
use super::metadata::MetadataStore;
use crate::error::Result;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

/// Complete slab-based storage engine
///
/// Provides key-value storage using:
/// - SlabAllocator for data storage (fixed-size slots)
/// - MetadataStore for key→slot mapping (atomic, no WAL)
/// - Compression for space efficiency
/// - LRU cache for performance
pub struct SlabStorage {
    allocator: Arc<SlabAllocator>,
    metadata: Arc<MetadataStore>,
    cache: SlabCache,
    compression: CompressionAlgorithm,
}

impl SlabStorage {
    /// Create or open a slab storage with compression and caching
    ///
    /// # Arguments
    /// * `base_path` - Directory for storage files
    /// * `min_slot_size` - Minimum slot size (default: 64 bytes)
    /// * `max_slot_size` - Maximum slot size (default: 64 KB)
    pub fn new<P: AsRef<Path>>(
        base_path: P,
        min_slot_size: Option<usize>,
        max_slot_size: Option<usize>,
    ) -> Result<Self> {
        Self::with_options(
            base_path,
            min_slot_size,
            max_slot_size,
            CompressionAlgorithm::Zstd,
            1000,
        )
    }

    /// Create with custom compression and cache settings
    pub fn with_options<P: AsRef<Path>>(
        base_path: P,
        min_slot_size: Option<usize>,
        max_slot_size: Option<usize>,
        compression: CompressionAlgorithm,
        cache_capacity: usize,
    ) -> Result<Self> {
        let base_path = base_path.as_ref();
        
        info!(path = ?base_path, "Opening slab storage with compression");

        // Create subdirectories
        let data_path = base_path.join("data");
        let meta_path = base_path.join("metadata");

        let allocator = Arc::new(SlabAllocator::new(&data_path, min_slot_size, max_slot_size)?);
        let metadata = Arc::new(MetadataStore::new(&meta_path)?);
        let cache = SlabCache::new(cache_capacity);

        Ok(Self {
            allocator,
            metadata,
            cache,
            compression,
        })
    }

    /// Get value for a key (with caching)
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // Check cache first
        if let Some((_slot_id, cached_data)) = self.cache.get(key) {
            return Ok(Some(cached_data));
        }

        // Cache miss: Look up slot in metadata
        let slot_id = match self.metadata.get(key) {
            Some(id) => id,
            None => return Ok(None),
        };

        // Read compressed data from allocator
        let compressed = self.allocator.read(slot_id)?;

        // Decompress
        let data = decompress(&compressed, self.compression)?;

        // Store in cache
        self.cache.put(key.to_vec(), slot_id, data.clone());

        Ok(Some(data))
    }

    /// Set key-value pair (with compression)
    pub fn set(&self, key: &[u8], value: &[u8]) -> Result<()> {
        // Compress value
        let compressed = compress(value, self.compression)?;

        // Check if key already exists
        if let Some(old_slot) = self.metadata.get(key) {
            // Free old slot
            self.allocator.free(old_slot)?;
        }

        // Allocate new slot for compressed data
        let slot_id = self.allocator.allocate(compressed.len())?;

        // Write compressed data
        self.allocator.write(slot_id, &compressed)?;

        // Update metadata atomically
        self.metadata
            .write_batch(vec![(key.to_vec(), slot_id)])?;

        // Invalidate cache
        self.cache.remove(key);

        debug!(key_len = key.len(), value_len = value.len(), compressed_len = compressed.len(), "Set key-value");
        Ok(())
    }

    /// Delete a key
    pub fn delete(&self, key: &[u8]) -> Result<bool> {
        // Look up slot
        let slot_id = match self.metadata.get(key) {
            Some(id) => id,
            None => return Ok(false),
        };

        // Free slot
        self.allocator.free(slot_id)?;

        // Remove from metadata
        self.metadata.remove(key)?;

        // Invalidate cache
        self.cache.remove(key);

        debug!(key_len = key.len(), "Deleted key");
        Ok(true)
    }

    /// List all keys
    pub fn keys(&self) -> Vec<Vec<u8>> {
        self.metadata.keys()
    }

    /// Check if key exists
    pub fn contains_key(&self, key: &[u8]) -> bool {
        self.metadata.get(key).is_some()
    }

    /// Get number of keys
    pub fn len(&self) -> usize {
        self.metadata.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.metadata.is_empty()
    }

    /// Flush all data to disk
    pub fn flush(&self) -> Result<()> {
        self.allocator.flush()?;
        // Metadata is already fsynced on write
        Ok(())
    }

    /// Compact metadata log
    pub fn compact_metadata(&self) -> Result<()> {
        self.metadata.compact()
    }

    /// Get storage statistics including cache metrics
    pub fn stats(&self) -> StorageStats {
        let slab_stats = self.allocator.stats();
        let cache_stats = self.cache.stats();
        StorageStats {
            key_count: self.len(),
            total_allocated: slab_stats.total_allocated,
            size_classes: slab_stats.size_classes.len(),
            cache_hits: cache_stats.hits,
            cache_misses: cache_stats.misses,
            cache_hit_rate: cache_stats.hit_rate,
        }
    }
}

/// Storage statistics with cache metrics
#[derive(Debug)]
pub struct StorageStats {
    pub key_count: usize,
    pub total_allocated: u64,
    pub size_classes: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slab_storage_basic() -> Result<()> {
        let temp_dir = std::env::temp_dir().join(format!("slab_storage_{}", std::process::id()));
        let storage = SlabStorage::new(&temp_dir, Some(64), Some(512))?;

        // Set some values
        storage.set(b"key1", b"value1")?;
        storage.set(b"key2", b"value2")?;

        // Get them back
        assert_eq!(storage.get(b"key1")?, Some(b"value1".to_vec()));
        assert_eq!(storage.get(b"key2")?, Some(b"value2".to_vec()));
        assert_eq!(storage.get(b"key3")?, None);

        // Check existence
        assert!(storage.contains_key(b"key1"));
        assert!(!storage.contains_key(b"key3"));

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
        Ok(())
    }

    #[test]
    fn test_slab_storage_update() -> Result<()> {
        let temp_dir =
            std::env::temp_dir().join(format!("slab_storage_update_{}", std::process::id()));
        let storage = SlabStorage::new(&temp_dir, Some(64), Some(512))?;

        // Set initial value
        storage.set(b"key1", b"initial")?;
        assert_eq!(storage.get(b"key1")?, Some(b"initial".to_vec()));

        // Update
        storage.set(b"key1", b"updated")?;
        assert_eq!(storage.get(b"key1")?, Some(b"updated".to_vec()));

        // Still only 1 key
        assert_eq!(storage.len(), 1);

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
        Ok(())
    }

    #[test]
    fn test_slab_storage_delete() -> Result<()> {
        let temp_dir =
            std::env::temp_dir().join(format!("slab_storage_delete_{}", std::process::id()));
        let storage = SlabStorage::new(&temp_dir, Some(64), Some(512))?;

        storage.set(b"key1", b"value1")?;
        storage.set(b"key2", b"value2")?;
        assert_eq!(storage.len(), 2);

        // Delete key1
        assert!(storage.delete(b"key1")?);
        assert_eq!(storage.get(b"key1")?, None);
        assert_eq!(storage.len(), 1);

        // Delete non-existent key
        assert!(!storage.delete(b"key3")?);

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
        Ok(())
    }

    #[test]
    fn test_slab_storage_persistence() -> Result<()> {
        let temp_dir =
            std::env::temp_dir().join(format!("slab_storage_persist_{}", std::process::id()));

        // Write data
        {
            let storage = SlabStorage::new(&temp_dir, Some(64), Some(512))?;
            storage.set(b"key1", b"value1")?;
            storage.set(b"key2", b"value2")?;
            storage.flush()?;
        }

        // Reopen and verify
        {
            let storage = SlabStorage::new(&temp_dir, Some(64), Some(512))?;
            assert_eq!(storage.get(b"key1")?, Some(b"value1".to_vec()));
            assert_eq!(storage.get(b"key2")?, Some(b"value2".to_vec()));
            assert_eq!(storage.len(), 2);
        }

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
        Ok(())
    }

    #[test]
    fn test_slab_storage_keys() -> Result<()> {
        let temp_dir =
            std::env::temp_dir().join(format!("slab_storage_keys_{}", std::process::id()));
        let storage = SlabStorage::new(&temp_dir, Some(64), Some(512))?;

        storage.set(b"key1", b"value1")?;
        storage.set(b"key2", b"value2")?;
        storage.set(b"key3", b"value3")?;

        let keys = storage.keys();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&b"key1".to_vec()));
        assert!(keys.contains(&b"key2".to_vec()));
        assert!(keys.contains(&b"key3".to_vec()));

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
        Ok(())
    }

    #[test]
    fn test_slab_storage_stats() -> Result<()> {
        let temp_dir =
            std::env::temp_dir().join(format!("slab_storage_stats_{}", std::process::id()));
        let storage = SlabStorage::new(&temp_dir, Some(64), Some(512))?;

        storage.set(b"key1", b"value1")?;
        storage.set(b"key2", b"value2")?;

        let stats = storage.stats();
        assert_eq!(stats.key_count, 2);
        assert!(stats.total_allocated > 0);
        assert!(stats.size_classes > 0);

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
        Ok(())
    }
}
