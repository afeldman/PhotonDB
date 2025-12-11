//! Slab allocator implementation

use super::size_class::{calculate_size_classes, SizeClass};
use super::slot::SlotId;
use crate::error::{Error, Result};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

/// Slab allocator for on-disk storage
///
/// Manages multiple size classes, each with its own file.
/// Similar to Sled's heap allocator but simplified.
pub struct SlabAllocator {
    /// Base directory for slab files
    base_path: PathBuf,
    /// Size classes (sorted by size)
    size_classes: Vec<Arc<RwLock<SizeClass>>>,
    /// File handles for each size class
    files: Vec<Arc<RwLock<File>>>,
}

impl SlabAllocator {
    /// Create a new slab allocator
    ///
    /// # Arguments
    /// * `base_path` - Directory to store slab files
    /// * `min_size` - Minimum slot size (default: 64 bytes)
    /// * `max_size` - Maximum slot size (default: 64 KB)
    pub fn new<P: AsRef<Path>>(
        base_path: P,
        min_size: Option<usize>,
        max_size: Option<usize>,
    ) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        std::fs::create_dir_all(&base_path)
            .map_err(|e| Error::Storage(format!("Failed to create slab directory: {}", e)))?;

        let min = min_size.unwrap_or(64);
        let max = max_size.unwrap_or(65536);

        let sizes = calculate_size_classes(min, max);
        info!(
            "Initializing slab allocator with {} size classes: {:?}",
            sizes.len(),
            sizes
        );

        let mut size_classes = Vec::new();
        let mut files = Vec::new();

        for (index, &size) in sizes.iter().enumerate() {
            // Create size class
            let sc = SizeClass::new(index as u16, size);
            size_classes.push(Arc::new(RwLock::new(sc)));

            // Open/create file for this size class
            let file_path = base_path.join(format!("slab_{:04}_{}.bin", index, size));
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&file_path)
                .map_err(|e| Error::Storage(format!("Failed to open slab file: {}", e)))?;

            files.push(Arc::new(RwLock::new(file)));
            debug!("Opened slab file: {:?}", file_path);
        }

        Ok(Self {
            base_path,
            size_classes,
            files,
        })
    }

    /// Allocate space for data of the given size
    ///
    /// Returns a SlotId that can be used to read/write the data.
    /// Note: Size includes 4-byte length prefix overhead.
    pub fn allocate(&self, size: usize) -> Result<SlotId> {
        let total_size = size + 4; // Account for 4-byte length prefix
        
        // Find the smallest size class that can fit this data
        let size_class_idx = self
            .size_classes
            .iter()
            .position(|sc| sc.read().unwrap().can_fit(total_size))
            .ok_or_else(|| {
                Error::Storage(format!(
                    "Data size {} (+4 byte prefix = {}) exceeds maximum slab size",
                    size,
                    total_size
                ))
            })?;

        // Allocate from that size class
        let mut sc = self.size_classes[size_class_idx].write().unwrap();
        let offset = sc.allocate();

        let slot_id = SlotId::new(size_class_idx as u16, offset);
        debug!("Allocated {} bytes at {}", size, slot_id);

        Ok(slot_id)
    }

    /// Free a previously allocated slot
    pub fn free(&self, slot_id: SlotId) -> Result<()> {
        let size_class_idx = slot_id.file_index();
        if size_class_idx >= self.size_classes.len() {
            return Err(Error::Storage(format!(
                "Invalid size class index: {}",
                size_class_idx
            )));
        }

        let mut sc = self.size_classes[size_class_idx].write().unwrap();
        sc.free(slot_id.offset);

        debug!("Freed slot {}", slot_id);
        Ok(())
    }

    /// Write data to a slot
    pub fn write(&self, slot_id: SlotId, data: &[u8]) -> Result<()> {
        let size_class_idx = slot_id.file_index();
        if size_class_idx >= self.files.len() {
            return Err(Error::Storage(format!(
                "Invalid size class index: {}",
                size_class_idx
            )));
        }

        // Check size (data + 4-byte length prefix must fit in slot)
        let sc = self.size_classes[size_class_idx].read().unwrap();
        let total_size = data.len() + 4; // 4 bytes for length prefix
        if total_size > sc.slot_size {
            return Err(Error::Storage(format!(
                "Data size {} (+4 byte prefix = {}) exceeds slot size {}",
                data.len(),
                total_size,
                sc.slot_size
            )));
        }
        drop(sc);

        // Write to file
        let mut file = self.files[size_class_idx].write().unwrap();
        file.seek(SeekFrom::Start(slot_id.offset))
            .map_err(|e| Error::Storage(format!("Seek failed: {}", e)))?;

        // Write length prefix (4 bytes) + data
        let len_bytes = (data.len() as u32).to_le_bytes();
        file.write_all(&len_bytes)
            .map_err(|e| Error::Storage(format!("Write failed: {}", e)))?;
        file.write_all(data)
            .map_err(|e| Error::Storage(format!("Write failed: {}", e)))?;

        debug!("Wrote {} bytes to {}", data.len(), slot_id);
        Ok(())
    }

    /// Read data from a slot
    pub fn read(&self, slot_id: SlotId) -> Result<Vec<u8>> {
        let size_class_idx = slot_id.file_index();
        if size_class_idx >= self.files.len() {
            return Err(Error::Storage(format!(
                "Invalid size class index: {}",
                size_class_idx
            )));
        }

        let mut file = self.files[size_class_idx].write().unwrap();
        file.seek(SeekFrom::Start(slot_id.offset))
            .map_err(|e| Error::Storage(format!("Seek failed: {}", e)))?;

        // Read length prefix
        let mut len_bytes = [0u8; 4];
        file.read_exact(&mut len_bytes)
            .map_err(|e| Error::Storage(format!("Read failed: {}", e)))?;
        let len = u32::from_le_bytes(len_bytes) as usize;

        // Read data
        let mut data = vec![0u8; len];
        file.read_exact(&mut data)
            .map_err(|e| Error::Storage(format!("Read failed: {}", e)))?;

        debug!("Read {} bytes from {}", data.len(), slot_id);
        Ok(data)
    }

    /// Get statistics about the allocator
    pub fn stats(&self) -> SlabStats {
        let mut stats = SlabStats::default();

        for (i, sc) in self.size_classes.iter().enumerate() {
            let sc = sc.read().unwrap();
            let class_stats = SizeClassStats {
                index: i,
                slot_size: sc.slot_size,
                total_slots: sc.total_slots(),
                free_slots: sc.free_count() as u64,
                allocated_slots: sc.total_slots() - sc.free_count() as u64,
            };
            stats.size_classes.push(class_stats);
            stats.total_allocated += class_stats.allocated_slots * sc.slot_size as u64;
        }

        stats
    }

    /// Flush all files to disk
    pub fn flush(&self) -> Result<()> {
        for file in &self.files {
            let file = file.write().unwrap();
            file.sync_all()
                .map_err(|e| Error::Storage(format!("Flush failed: {}", e)))?;
        }
        Ok(())
    }
}

/// Statistics for the slab allocator
#[derive(Debug, Default)]
pub struct SlabStats {
    pub size_classes: Vec<SizeClassStats>,
    pub total_allocated: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct SizeClassStats {
    pub index: usize,
    pub slot_size: usize,
    pub total_slots: u64,
    pub free_slots: u64,
    pub allocated_slots: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocator_basic() -> Result<()> {
        let temp_dir = std::env::temp_dir().join(format!("slab_test_{}", std::process::id()));
        let allocator = SlabAllocator::new(&temp_dir, Some(64), Some(512))?;

        // Allocate a slot (100 bytes fits in 112B class, which is index 3)
        // Size classes: [64, 77, 93, 112, 135, 162, 195, 234, 281, 338, 406, 488]
        let slot = allocator.allocate(100)?;
        assert_eq!(slot.size_class, 3); // 112B class

        // Write data
        let data = b"Hello, Slab!";
        allocator.write(slot, data)?;

        // Read it back
        let read_data = allocator.read(slot)?;
        assert_eq!(&read_data, data);

        // Free the slot
        allocator.free(slot)?;

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
        Ok(())
    }

    #[test]
    fn test_allocator_reuse() -> Result<()> {
        let temp_dir = std::env::temp_dir().join(format!("slab_test_reuse_{}", std::process::id()));
        let allocator = SlabAllocator::new(&temp_dir, Some(64), Some(256))?;

        // Allocate two slots
        let slot1 = allocator.allocate(50)?;
        let slot2 = allocator.allocate(50)?;

        assert_ne!(slot1.offset, slot2.offset);

        // Free first slot
        allocator.free(slot1)?;

        // Next allocation should reuse first slot
        let slot3 = allocator.allocate(50)?;
        assert_eq!(slot3.offset, slot1.offset);

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
        Ok(())
    }

    #[test]
    fn test_allocator_stats() -> Result<()> {
        let temp_dir = std::env::temp_dir().join(format!("slab_test_stats_{}", std::process::id()));
        let allocator = SlabAllocator::new(&temp_dir, Some(64), Some(256))?;

        // Allocate some slots
        allocator.allocate(50)?;
        allocator.allocate(100)?;
        allocator.allocate(200)?;

        let stats = allocator.stats();
        assert!(!stats.size_classes.is_empty());
        assert!(stats.total_allocated > 0);

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
        Ok(())
    }
}
