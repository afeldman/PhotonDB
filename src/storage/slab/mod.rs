//! Slab Allocator
//!
//! Inspired by Sled's heap allocator design. Manages on-disk storage
//! using fixed-size slots organized by size classes.
//!
//! # Architecture
//!
//! ```text
//! SlabAllocator
//!   ├─→ SizeClass(64B)   → Free: [3, 7, 12]
//!   ├─→ SizeClass(128B)  → Free: [1, 4]
//!   ├─→ SizeClass(256B)  → Free: [2, 9]
//!   └─→ SizeClass(512B)  → Free: []
//!
//! MetadataStore (Atomic, No WAL)
//!   └─→ key1 → SlotId(class=3, offset=128)
//!   └─→ key2 → SlotId(class=1, offset=64)
//!
//! Performance (Phase 4):
//!   ├─→ Parallel Writes (Rayon)
//!   ├─→ Compression (Zstd)
//!   └─→ LRU Cache (Scan-resistant)
//! ```
//!
//! Each size class maintains a heap of free slots.
//! Metadata store provides atomic key→slot mapping without WAL.

pub mod allocator;
pub mod bench;
pub mod cache;
pub mod compression;
pub mod engine;
pub mod metadata;
pub mod production_tests;
pub mod size_class;
pub mod slot;
pub mod storage;

pub use allocator::SlabAllocator;
pub use cache::{CacheStats, SlabCache};
pub use compression::{compress, decompress, CompressionAlgorithm, CompressionStats};
pub use engine::SlabStorageEngine;
pub use metadata::{MetadataBatch, MetadataStore};
pub use size_class::SizeClass;
pub use slot::{Slot, SlotId};
pub use storage::{SlabStorage, StorageStats};
