//! Content-defined chunking using FastCDC.
//!
//! Splits files into variable-size chunks based on content boundaries.
//! This ensures that inserting or modifying data in the middle of a file
//! only affects the chunks around the modification, not the entire file.

use std::io::Read;
use std::path::Path;
use thiserror::Error;

/// Minimum chunk size (4 KB)
pub const MIN_CHUNK_SIZE: u32 = 4 * 1024;

/// Average chunk size (64 KB) - good balance for most file types
pub const AVG_CHUNK_SIZE: u32 = 64 * 1024;

/// Maximum chunk size (256 KB)
pub const MAX_CHUNK_SIZE: u32 = 256 * 1024;

/// Large file threshold — files above this use larger chunks (1 MB)
pub const LARGE_FILE_THRESHOLD: u64 = 100 * 1024 * 1024; // 100 MB

/// Large file average chunk size (1 MB)
pub const LARGE_AVG_CHUNK_SIZE: u32 = 1024 * 1024;

/// Large file max chunk size (4 MB)
pub const LARGE_MAX_CHUNK_SIZE: u32 = 4 * 1024 * 1024;

#[derive(Error, Debug)]
pub enum ChunkError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("File not found: {0}")]
    FileNotFound(String),
}

/// A chunk of data with its BLAKE3 hash
#[derive(Debug, Clone)]
pub struct Chunk {
    /// BLAKE3 hash of the chunk data (hex encoded)
    pub hash: String,
    /// Offset in the original file
    pub offset: u64,
    /// Size of this chunk in bytes
    pub size: u64,
    /// The raw chunk data
    pub data: Vec<u8>,
}

/// Chunking configuration
#[derive(Debug, Clone)]
pub struct ChunkConfig {
    pub min_size: u32,
    pub avg_size: u32,
    pub max_size: u32,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            min_size: MIN_CHUNK_SIZE,
            avg_size: AVG_CHUNK_SIZE,
            max_size: MAX_CHUNK_SIZE,
        }
    }
}

impl ChunkConfig {
    /// Get configuration optimized for large files
    pub fn for_large_files() -> Self {
        Self {
            min_size: MIN_CHUNK_SIZE,
            avg_size: LARGE_AVG_CHUNK_SIZE,
            max_size: LARGE_MAX_CHUNK_SIZE,
        }
    }

    /// Auto-select config based on file size
    pub fn auto(file_size: u64) -> Self {
        if file_size > LARGE_FILE_THRESHOLD {
            Self::for_large_files()
        } else {
            Self::default()
        }
    }
}

/// Chunk a file into content-defined chunks using FastCDC
pub fn chunk_file(path: &Path) -> Result<Vec<Chunk>, ChunkError> {
    if !path.exists() {
        return Err(ChunkError::FileNotFound(path.display().to_string()));
    }

    let metadata = std::fs::metadata(path)?;
    let file_size = metadata.len();

    // For small files, just use the whole file as one chunk
    if file_size <= MIN_CHUNK_SIZE as u64 {
        let data = std::fs::read(path)?;
        let hash = crate::crypto::hash_bytes(&data);
        return Ok(vec![Chunk {
            hash,
            offset: 0,
            size: data.len() as u64,
            data,
        }]);
    }

    // Read the entire file for FastCDC (it needs contiguous data)
    // For very large files, we'll stream in segments
    let config = ChunkConfig::auto(file_size);
    chunk_file_with_config(path, &config)
}

/// Chunk a file with specific configuration
pub fn chunk_file_with_config(path: &Path, config: &ChunkConfig) -> Result<Vec<Chunk>, ChunkError> {
    let data = std::fs::read(path)?;
    Ok(chunk_data(&data, config))
}

/// Chunk raw data into content-defined chunks
pub fn chunk_data(data: &[u8], config: &ChunkConfig) -> Vec<Chunk> {
    if data.is_empty() {
        return Vec::new();
    }

    // For data smaller than min chunk size, return as single chunk
    if data.len() <= config.min_size as usize {
        let hash = crate::crypto::hash_bytes(data);
        return vec![Chunk {
            hash,
            offset: 0,
            size: data.len() as u64,
            data: data.to_vec(),
        }];
    }

    let chunker = fastcdc::v2020::FastCDC::new(
        data,
        config.min_size,
        config.avg_size,
        config.max_size,
    );

    let mut chunks = Vec::new();
    for entry in chunker {
        let chunk_data = &data[entry.offset..entry.offset + entry.length];
        let hash = crate::crypto::hash_bytes(chunk_data);
        chunks.push(Chunk {
            hash,
            offset: entry.offset as u64,
            size: entry.length as u64,
            data: chunk_data.to_vec(),
        });
    }

    chunks
}

/// Stream-chunk a large file without loading it entirely into memory
/// Reads the file in segments and applies FastCDC to each segment
pub fn chunk_file_streaming(path: &Path, config: &ChunkConfig) -> Result<Vec<Chunk>, ChunkError> {
    let metadata = std::fs::metadata(path)?;
    let file_size = metadata.len();

    // For files that fit in memory reasonably (~512 MB), just read all at once
    if file_size < 512 * 1024 * 1024 {
        return chunk_file_with_config(path, config);
    }

    // For very large files, read in segments
    // Each segment is large enough to contain multiple chunks
    let segment_size = config.max_size as usize * 16; // ~4-64 MB segments
    let mut file = std::io::BufReader::with_capacity(segment_size, std::fs::File::open(path)?);
    let mut chunks = Vec::new();
    let mut global_offset: u64 = 0;
    let mut buffer = vec![0u8; segment_size + config.max_size as usize]; // overlap for boundary
    let mut carry_over = Vec::new();

    loop {
        // Prepend any carry-over from previous segment
        let carry_len = carry_over.len();
        buffer[..carry_len].copy_from_slice(&carry_over);

        let n = read_fill(&mut file, &mut buffer[carry_len..])?;
        if n == 0 && carry_len == 0 {
            break;
        }

        let total = carry_len + n;
        let is_last = n == 0 || total < segment_size;

        let segment = &buffer[..total];

        let chunker = fastcdc::v2020::FastCDC::new(
            segment,
            config.min_size,
            config.avg_size,
            config.max_size,
        );

        let mut last_end = 0;
        for entry in chunker {
            let end = entry.offset + entry.length;

            // If this is not the last read and the chunk ends near the boundary,
            // carry it over to the next segment for proper boundary detection
            if !is_last && end > total - config.max_size as usize {
                break;
            }

            let chunk_data = &segment[entry.offset..end];
            let hash = crate::crypto::hash_bytes(chunk_data);
            chunks.push(Chunk {
                hash,
                offset: global_offset + entry.offset as u64,
                size: entry.length as u64,
                data: chunk_data.to_vec(),
            });
            last_end = end;
        }

        // Carry over unprocessed data
        carry_over = segment[last_end..total].to_vec();
        global_offset += last_end as u64;

        if is_last {
            // Process remaining carry-over as a final chunk
            if !carry_over.is_empty() {
                let hash = crate::crypto::hash_bytes(&carry_over);
                chunks.push(Chunk {
                    hash,
                    offset: global_offset,
                    size: carry_over.len() as u64,
                    data: carry_over,
                });
            }
            break;
        }
    }

    Ok(chunks)
}

/// Helper to read as much as possible into buffer
fn read_fill<R: Read>(reader: &mut R, buf: &mut [u8]) -> Result<usize, std::io::Error> {
    let mut total = 0;
    while total < buf.len() {
        match reader.read(&mut buf[total..])? {
            0 => break,
            n => total += n,
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_small_data() {
        let data = b"Hello, Ferrum!";
        let chunks = chunk_data(data, &ChunkConfig::default());
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].data, data);
        assert_eq!(chunks[0].offset, 0);
    }

    #[test]
    fn test_chunk_large_data() {
        // Create 1 MB of random-ish data
        let data: Vec<u8> = (0..1024 * 1024).map(|i| (i % 256) as u8).collect();
        let chunks = chunk_data(&data, &ChunkConfig::default());
        assert!(chunks.len() > 1, "Expected multiple chunks, got {}", chunks.len());

        // Verify all data is accounted for
        let total_size: u64 = chunks.iter().map(|c| c.size).sum();
        assert_eq!(total_size, data.len() as u64);

        // Verify chunks are contiguous
        let mut expected_offset = 0u64;
        for chunk in &chunks {
            assert_eq!(chunk.offset, expected_offset);
            expected_offset += chunk.size;
        }
    }

    #[test]
    fn test_chunk_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_data.bin");

        // Write 512 KB of data
        let data: Vec<u8> = (0..512 * 1024).map(|i| (i % 251) as u8).collect();
        std::fs::write(&file_path, &data).unwrap();

        let chunks = chunk_file(&file_path).unwrap();
        assert!(chunks.len() >= 1);

        let total: u64 = chunks.iter().map(|c| c.size).sum();
        assert_eq!(total, data.len() as u64);
    }

    #[test]
    fn test_deterministic_chunking() {
        let data: Vec<u8> = (0..256 * 1024).map(|i| (i % 256) as u8).collect();
        let chunks1 = chunk_data(&data, &ChunkConfig::default());
        let chunks2 = chunk_data(&data, &ChunkConfig::default());

        assert_eq!(chunks1.len(), chunks2.len());
        for (c1, c2) in chunks1.iter().zip(chunks2.iter()) {
            assert_eq!(c1.hash, c2.hash);
            assert_eq!(c1.offset, c2.offset);
            assert_eq!(c1.size, c2.size);
        }
    }

    #[test]
    fn test_empty_data() {
        let chunks = chunk_data(&[], &ChunkConfig::default());
        assert!(chunks.is_empty());
    }
}
