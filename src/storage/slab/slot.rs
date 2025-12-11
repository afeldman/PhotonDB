//! Slot management for slab allocator

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a slot in the heap
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SlotId {
    /// Size class index (0 = smallest)
    pub size_class: u16,
    /// Slot offset within the size class file
    pub offset: u64,
}

impl SlotId {
    /// Create a new slot ID
    pub fn new(size_class: u16, offset: u64) -> Self {
        Self { size_class, offset }
    }

    /// Get the file index for this slot
    pub fn file_index(&self) -> usize {
        self.size_class as usize
    }
}

impl fmt::Display for SlotId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Slot(class={}, offset={})", self.size_class, self.offset)
    }
}

/// A slot in the slab allocator
#[derive(Debug, Clone)]
pub struct Slot {
    /// Unique ID
    pub id: SlotId,
    /// Size of this slot in bytes
    pub size: usize,
    /// Whether this slot is currently allocated
    pub allocated: bool,
}

impl Slot {
    /// Create a new free slot
    pub fn new(id: SlotId, size: usize) -> Self {
        Self {
            id,
            size,
            allocated: false,
        }
    }

    /// Allocate this slot
    pub fn allocate(&mut self) {
        self.allocated = true;
    }

    /// Free this slot
    pub fn free(&mut self) {
        self.allocated = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_id_creation() {
        let id = SlotId::new(5, 1024);
        assert_eq!(id.size_class, 5);
        assert_eq!(id.offset, 1024);
        assert_eq!(id.file_index(), 5);
    }

    #[test]
    fn test_slot_lifecycle() {
        let id = SlotId::new(0, 0);
        let mut slot = Slot::new(id, 64);
        
        assert!(!slot.allocated);
        
        slot.allocate();
        assert!(slot.allocated);
        
        slot.free();
        assert!(!slot.allocated);
    }
}
