//! Compression engine supporting LZ4 (fast) and Zstd (balanced).

use crate::config::CompressionAlgorithm;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompressError {
    #[error("Compression failed: {0}")]
    CompressionFailed(String),

    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),
}

/// Header byte indicating compression algorithm
const HEADER_NONE: u8 = 0x00;
const HEADER_LZ4: u8 = 0x01;
const HEADER_ZSTD: u8 = 0x02;

/// Compress data using the specified algorithm.
/// Returns: [1-byte header] [compressed data]
pub fn compress(data: &[u8], algorithm: &CompressionAlgorithm) -> Result<Vec<u8>, CompressError> {
    match algorithm {
        CompressionAlgorithm::None => {
            let mut output = Vec::with_capacity(1 + data.len());
            output.push(HEADER_NONE);
            output.extend_from_slice(data);
            Ok(output)
        }
        CompressionAlgorithm::Lz4 => {
            let compressed = lz4_flex::compress_prepend_size(data);
            let mut output = Vec::with_capacity(1 + compressed.len());
            output.push(HEADER_LZ4);
            output.extend_from_slice(&compressed);
            Ok(output)
        }
        CompressionAlgorithm::Zstd { level } => {
            let compressed = zstd::encode_all(data, *level)
                .map_err(|e| CompressError::CompressionFailed(e.to_string()))?;
            let mut output = Vec::with_capacity(1 + compressed.len());
            output.push(HEADER_ZSTD);
            output.extend_from_slice(&compressed);
            Ok(output)
        }
    }
}

/// Decompress data (auto-detects algorithm from header)
pub fn decompress(data: &[u8]) -> Result<Vec<u8>, CompressError> {
    if data.is_empty() {
        return Err(CompressError::DecompressionFailed("Empty data".into()));
    }

    let header = data[0];
    let payload = &data[1..];

    match header {
        HEADER_NONE => Ok(payload.to_vec()),
        HEADER_LZ4 => {
            lz4_flex::decompress_size_prepended(payload)
                .map_err(|e| CompressError::DecompressionFailed(e.to_string()))
        }
        HEADER_ZSTD => {
            zstd::decode_all(payload)
                .map_err(|e| CompressError::DecompressionFailed(e.to_string()))
        }
        _ => Err(CompressError::DecompressionFailed(
            format!("Unknown compression header: 0x{:02x}", header),
        )),
    }
}

/// Get the compression ratio for a piece of data
pub fn compression_ratio(original_size: u64, compressed_size: u64) -> f64 {
    if compressed_size == 0 {
        return 1.0;
    }
    original_size as f64 / compressed_size as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lz4_round_trip() {
        let data = b"Hello, this is test data that should compress well. ".repeat(100);
        let compressed = compress(&data, &CompressionAlgorithm::Lz4).unwrap();
        assert!(compressed.len() < data.len()); // Should actually compress

        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_zstd_round_trip() {
        let data = b"Zstd compression test data with repetition. ".repeat(100);
        let compressed = compress(&data, &CompressionAlgorithm::Zstd { level: 3 }).unwrap();
        assert!(compressed.len() < data.len());

        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_none_round_trip() {
        let data = b"Uncompressed data";
        let compressed = compress(data, &CompressionAlgorithm::None).unwrap();
        assert_eq!(compressed.len(), data.len() + 1); // +1 for header

        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_empty_data() {
        let compressed = compress(b"", &CompressionAlgorithm::Lz4).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert!(decompressed.is_empty());
    }
}
