//! Compression support for slab storage

use crate::error::{Error, Result};
use std::io::Write;

/// Compression algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    None,
    Zstd,
}

/// Compress data using specified algorithm
pub fn compress(data: &[u8], algorithm: CompressionAlgorithm) -> Result<Vec<u8>> {
    match algorithm {
        CompressionAlgorithm::None => Ok(data.to_vec()),
        CompressionAlgorithm::Zstd => {
            let mut encoder = zstd::Encoder::new(Vec::new(), 3)
                .map_err(|e| Error::Storage(format!("Failed to create zstd encoder: {}", e)))?;
            encoder
                .write_all(data)
                .map_err(|e| Error::Storage(format!("Failed to compress: {}", e)))?;
            encoder
                .finish()
                .map_err(|e| Error::Storage(format!("Failed to finish compression: {}", e)))
        }
    }
}

/// Decompress data using specified algorithm
pub fn decompress(data: &[u8], algorithm: CompressionAlgorithm) -> Result<Vec<u8>> {
    match algorithm {
        CompressionAlgorithm::None => Ok(data.to_vec()),
        CompressionAlgorithm::Zstd => zstd::decode_all(data)
            .map_err(|e| Error::Storage(format!("Failed to decompress: {}", e))),
    }
}

/// Compression statistics
#[derive(Debug, Clone)]
pub struct CompressionStats {
    pub original_size: usize,
    pub compressed_size: usize,
    pub ratio: f64,
}

impl CompressionStats {
    pub fn new(original_size: usize, compressed_size: usize) -> Self {
        let ratio = if original_size > 0 {
            compressed_size as f64 / original_size as f64
        } else {
            1.0
        };
        Self {
            original_size,
            compressed_size,
            ratio,
        }
    }

    /// Calculate space saved (percentage)
    pub fn space_saved_percent(&self) -> f64 {
        (1.0 - self.ratio) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_none() -> Result<()> {
        let data = b"Hello, World!";
        let compressed = compress(data, CompressionAlgorithm::None)?;
        assert_eq!(compressed, data);

        let decompressed = decompress(&compressed, CompressionAlgorithm::None)?;
        assert_eq!(decompressed, data);
        Ok(())
    }

    #[test]
    fn test_compression_zstd() -> Result<()> {
        let data = b"Hello, World! This is a test of zstd compression. ".repeat(10);
        let compressed = compress(&data, CompressionAlgorithm::Zstd)?;
        
        // Compression should reduce size for repetitive data
        assert!(compressed.len() < data.len());

        let decompressed = decompress(&compressed, CompressionAlgorithm::Zstd)?;
        assert_eq!(decompressed, data);
        Ok(())
    }

    #[test]
    fn test_compression_stats() {
        let stats = CompressionStats::new(1000, 250);
        assert_eq!(stats.ratio, 0.25);
        assert_eq!(stats.space_saved_percent(), 75.0);
    }
}
