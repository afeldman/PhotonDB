//! Size class management for slab allocator

use std::collections::BinaryHeap;
use std::cmp::Reverse;

/// A size class manages slots of a specific size
///
/// Uses a min-heap to reuse slots in FIFO order (better cache locality)
#[derive(Debug)]
pub struct SizeClass {
    /// Size of slots in this class (bytes)
    pub slot_size: usize,
    /// Index of this size class
    pub index: u16,
    /// Free slots (stored as offsets)
    free_slots: BinaryHeap<Reverse<u64>>,
    /// Next offset to allocate (if no free slots)
    next_offset: u64,
}

impl SizeClass {
    /// Create a new size class
    pub fn new(index: u16, slot_size: usize) -> Self {
        Self {
            slot_size,
            index,
            free_slots: BinaryHeap::new(),
            next_offset: 0,
        }
    }

    /// Allocate a slot from this size class
    ///
    /// Returns the offset of the allocated slot
    pub fn allocate(&mut self) -> u64 {
        // Try to reuse a free slot first
        if let Some(Reverse(offset)) = self.free_slots.pop() {
            return offset;
        }

        // Otherwise, allocate a new slot at the end
        let offset = self.next_offset;
        self.next_offset += self.slot_size as u64;
        offset
    }

    /// Free a slot by returning it to the free list
    pub fn free(&mut self, offset: u64) {
        self.free_slots.push(Reverse(offset));
    }

    /// Get the number of free slots
    pub fn free_count(&self) -> usize {
        self.free_slots.len()
    }

    /// Get the total number of slots allocated (including free)
    pub fn total_slots(&self) -> u64 {
        self.next_offset / self.slot_size as u64
    }

    /// Check if a given size fits in this size class
    pub fn can_fit(&self, size: usize) -> bool {
        size <= self.slot_size
    }
}

/// Calculate size classes with ~20% growth factor (Sled-style)
///
/// Returns a vector of size classes: [64, 80, 96, 128, 160, ...]
pub fn calculate_size_classes(min_size: usize, max_size: usize) -> Vec<usize> {
    let mut classes = Vec::new();
    let mut current = min_size;

    while current <= max_size {
        classes.push(current);
        // Grow by 20%
        current = (current as f64 * 1.2).ceil() as usize;
    }

    classes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_class_allocation() {
        let mut sc = SizeClass::new(0, 64);

        // First allocation should be at offset 0
        assert_eq!(sc.allocate(), 0);
        // Second allocation at offset 64
        assert_eq!(sc.allocate(), 64);
        // Third at offset 128
        assert_eq!(sc.allocate(), 128);

        assert_eq!(sc.total_slots(), 3);
        assert_eq!(sc.free_count(), 0);
    }

    #[test]
    fn test_size_class_reuse() {
        let mut sc = SizeClass::new(0, 64);

        let offset1 = sc.allocate(); // 0
        let _offset2 = sc.allocate(); // 64
        
        // Free the first slot
        sc.free(offset1);
        assert_eq!(sc.free_count(), 1);

        // Next allocation should reuse the freed slot
        assert_eq!(sc.allocate(), 0);
        assert_eq!(sc.free_count(), 0);

        // New allocation continues from where we left off
        assert_eq!(sc.allocate(), 128);
    }

    #[test]
    fn test_calculate_size_classes() {
        let classes = calculate_size_classes(64, 500);
        
        // Should start with 64
        assert_eq!(classes[0], 64);
        
        // Each class should be ~20% larger than previous
        for i in 1..classes.len() {
            let ratio = classes[i] as f64 / classes[i-1] as f64;
            assert!(ratio >= 1.15 && ratio <= 1.25, "Ratio: {}", ratio);
        }

        // All classes should be <= max_size
        assert!(classes.iter().all(|&s| s <= 500));
    }

    #[test]
    fn test_can_fit() {
        let sc = SizeClass::new(0, 128);
        
        assert!(sc.can_fit(64));
        assert!(sc.can_fit(128));
        assert!(!sc.can_fit(129));
        assert!(!sc.can_fit(256));
    }
}
