//! InnerFile - a logical file inside a RAR archive.
//!
//! An inner file may span multiple RarFileChunks across multiple volumes.
//! Optimized for streaming and seeking with binary search chunk lookup.

use crate::decompress::rar5::Rar5Decoder;
use crate::decompress::Rar29Decoder;
use crate::error::{RarError, Result};
use crate::file_media::ReadInterval;
use crate::parsing::RarVersion;
use crate::rar_file_chunk::RarFileChunk;
use std::sync::{Arc, Mutex};

/// Mapping of a chunk within the logical file.
/// Stored sorted by start offset for binary search.
#[derive(Clone, Debug)]
pub struct ChunkMapEntry {
    pub index: usize,
    pub start: u64,
    pub end: u64,
}

/// Encryption info for a file.
#[cfg(feature = "crypto")]
#[derive(Debug, Clone)]
pub enum EncryptionInfo {
    /// RAR5 encryption (AES-256-CBC with PBKDF2)
    Rar5 {
        /// 16-byte salt for key derivation
        salt: [u8; 16],
        /// 16-byte initialization vector
        init_v: [u8; 16],
        /// Log2 of PBKDF2 iteration count
        lg2_count: u8,
    },
    /// RAR4 encryption (AES-256-CBC with custom SHA-1 KDF)
    Rar4 {
        /// 8-byte salt for key derivation
        salt: [u8; 8],
    },
}

/// A file inside a RAR archive, potentially spanning multiple chunks/volumes.
/// Optimized for streaming video with fast seeking.
#[derive(Debug)]
pub struct InnerFile {
    pub name: String,
    pub length: u64,
    chunks: Vec<RarFileChunk>,
    /// Sorted by start offset for binary search
    chunk_map: Vec<ChunkMapEntry>,
    /// Compression method (0x30 = store, 0x31-0x35 = LZSS, etc.)
    method: u8,
    /// Packed size (sum of chunk sizes)
    packed_size: u64,
    /// RAR version (4 or 5)
    rar_version: RarVersion,
    /// Cached decompressed data (for compressed files) - Arc to avoid cloning
    decompressed_cache: Mutex<Option<Arc<Vec<u8>>>>,
    /// Encryption info (if encrypted)
    #[cfg(feature = "crypto")]
    encryption: Option<EncryptionInfo>,
    /// Password for decryption
    #[cfg(feature = "crypto")]
    password: Option<String>,
}

impl InnerFile {
    pub fn new(
        name: String,
        chunks: Vec<RarFileChunk>,
        method: u8,
        unpacked_size: u64,
        rar_version: RarVersion,
    ) -> Self {
        let packed_size: u64 = chunks.iter().map(|c| c.length()).sum();
        let chunk_map = Self::calculate_chunk_map(&chunks);

        // For stored files, length = packed_size
        // For compressed files, length = unpacked_size
        let length = if method == 0x30 || method == 0 {
            packed_size
        } else {
            unpacked_size
        };

        Self {
            name,
            length,
            chunks,
            chunk_map,
            method,
            packed_size,
            rar_version,
            decompressed_cache: Mutex::new(None),
            #[cfg(feature = "crypto")]
            encryption: None,
            #[cfg(feature = "crypto")]
            password: None,
        }
    }

    /// Create an InnerFile with encryption info.
    #[cfg(feature = "crypto")]
    pub fn new_encrypted(
        name: String,
        chunks: Vec<RarFileChunk>,
        method: u8,
        unpacked_size: u64,
        rar_version: RarVersion,
        encryption: Option<EncryptionInfo>,
        password: Option<String>,
    ) -> Self {
        let packed_size: u64 = chunks.iter().map(|c| c.length()).sum();
        let chunk_map = Self::calculate_chunk_map(&chunks);

        let length = if method == 0x30 || method == 0 {
            packed_size
        } else {
            unpacked_size
        };

        Self {
            name,
            length,
            chunks,
            chunk_map,
            method,
            packed_size,
            rar_version,
            decompressed_cache: Mutex::new(None),
            encryption,
            password,
        }
    }

    /// Check if this file is encrypted.
    #[cfg(feature = "crypto")]
    pub fn is_encrypted(&self) -> bool {
        self.encryption.is_some()
    }

    /// Returns true if this file is compressed.
    pub fn is_compressed(&self) -> bool {
        match self.rar_version {
            RarVersion::Rar4 => self.method != 0x30,
            RarVersion::Rar5 => self.method != 0, // RAR5 uses 0 for stored
        }
    }

    fn calculate_chunk_map(chunks: &[RarFileChunk]) -> Vec<ChunkMapEntry> {
        let mut map = Vec::with_capacity(chunks.len());
        let mut offset = 0u64;

        for (index, chunk) in chunks.iter().enumerate() {
            let start = offset;
            let len = chunk.length();
            let end = if len > 0 { offset + len - 1 } else { offset };
            map.push(ChunkMapEntry { index, start, end });
            offset = end + 1;
        }

        map
    }

    /// Find which chunk contains the given offset using binary search.
    /// O(log n) complexity for fast seeking.
    #[inline]
    pub fn find_chunk_index(&self, offset: u64) -> Option<usize> {
        if offset >= self.length {
            return None;
        }

        // Binary search: find the chunk where start <= offset <= end
        let idx = self.chunk_map.partition_point(|entry| entry.end < offset);

        if idx < self.chunk_map.len() && self.chunk_map[idx].start <= offset {
            Some(idx)
        } else {
            None
        }
    }

    /// Get chunk entry by index.
    #[inline]
    pub fn get_chunk_entry(&self, index: usize) -> Option<&ChunkMapEntry> {
        self.chunk_map.get(index)
    }

    /// Get the underlying chunk by index.
    #[inline]
    pub fn get_chunk(&self, index: usize) -> Option<&RarFileChunk> {
        self.chunks.get(index)
    }

    /// Number of chunks in this file.
    #[inline]
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Read the entire file.
    pub async fn read_to_end(&self) -> Result<Vec<u8>> {
        if self.is_compressed() {
            let data = self.read_decompressed().await?;
            Ok((*data).clone())
        } else {
            self.read_raw_range(0, self.length.saturating_sub(1)).await
        }
    }

    /// Read raw data from all chunks (no decompression).
    async fn read_raw_range(&self, start: u64, end: u64) -> Result<Vec<u8>> {
        if start > end {
            return Ok(Vec::new());
        }

        let packed_len = self.packed_size;
        let actual_end = end.min(packed_len.saturating_sub(1));

        if start >= packed_len {
            return Ok(Vec::new());
        }

        let start_idx = self
            .find_chunk_index(start)
            .ok_or(RarError::InvalidOffset {
                offset: start,
                length: packed_len,
            })?;
        let end_idx = self
            .find_chunk_index(actual_end)
            .ok_or(RarError::InvalidOffset {
                offset: actual_end,
                length: packed_len,
            })?;

        let mut result = Vec::with_capacity((actual_end - start + 1) as usize);

        for i in start_idx..=end_idx {
            let entry = &self.chunk_map[i];
            let chunk = &self.chunks[i];

            let chunk_start = if i == start_idx {
                start - entry.start
            } else {
                0
            };
            let chunk_end = if i == end_idx {
                actual_end - entry.start
            } else {
                chunk.length().saturating_sub(1)
            };

            let data = chunk.read_range(chunk_start, chunk_end).await?;
            result.extend_from_slice(&data);
        }

        Ok(result)
    }

    /// Read all raw packed data from chunks.
    async fn read_all_raw(&self) -> Result<Vec<u8>> {
        let mut result = Vec::with_capacity(self.packed_size as usize);
        for chunk in &self.chunks {
            let data = chunk
                .read_range(0, chunk.length().saturating_sub(1))
                .await?;
            result.extend_from_slice(&data);
        }
        Ok(result)
    }

    /// Read decompressed data (with caching).
    pub async fn read_decompressed(&self) -> Result<Arc<Vec<u8>>> {
        // Check cache first
        {
            let cache = self.decompressed_cache.lock().unwrap();
            if let Some(ref data) = *cache {
                return Ok(Arc::clone(data));
            }
        }

        // Read all packed data
        let mut packed = self.read_all_raw().await?;

        // Decrypt if encrypted
        #[cfg(feature = "crypto")]
        if let Some(ref enc) = self.encryption {
            let password = self.password.as_ref().ok_or(RarError::PasswordRequired)?;

            match enc {
                EncryptionInfo::Rar5 {
                    salt,
                    init_v,
                    lg2_count,
                } => {
                    use crate::crypto::Rar5Crypto;
                    let crypto = Rar5Crypto::derive_key(password, salt, *lg2_count);
                    crypto
                        .decrypt(init_v, &mut packed)
                        .map_err(|e| RarError::DecryptionFailed(e.to_string()))?;
                }
                EncryptionInfo::Rar4 { salt } => {
                    use crate::crypto::Rar4Crypto;
                    let crypto = Rar4Crypto::derive_key(password, salt);
                    crypto
                        .decrypt(&mut packed)
                        .map_err(|e| RarError::DecryptionFailed(e.to_string()))?;
                }
            }
        }

        // Decompress based on RAR version
        let decompressed = match self.rar_version {
            RarVersion::Rar4 => {
                let mut decoder = Rar29Decoder::new();
                decoder.decompress(&packed, self.length)?
            }
            RarVersion::Rar5 => {
                let mut decoder = Rar5Decoder::new();
                // For RAR5: method is stored directly, not offset by 0x30
                // is_solid = false for now (TODO: track from archive flags)
                decoder.decompress(&packed, self.length, self.method, false)?
            }
        };
        let decompressed = Arc::new(decompressed);

        // Cache result
        {
            let mut cache = self.decompressed_cache.lock().unwrap();
            *cache = Some(Arc::clone(&decompressed));
        }

        Ok(decompressed)
    }

    /// Read a range of bytes from the file.
    /// Optimized for sequential reads within chunks.
    pub async fn read_range(&self, interval: ReadInterval) -> Result<Vec<u8>> {
        let start = interval.start;
        let end = interval.end;

        if start > end || end >= self.length {
            return Err(RarError::InvalidOffset {
                offset: end,
                length: self.length,
            });
        }

        if self.is_compressed() {
            // For compressed files, decompress and slice
            let decompressed = self.read_decompressed().await?;
            let start_usize = start as usize;
            let end_usize = (end + 1) as usize;
            if end_usize > decompressed.len() {
                return Err(RarError::InvalidOffset {
                    offset: end,
                    length: self.length,
                });
            }
            return Ok(decompressed[start_usize..end_usize].to_vec());
        }

        let start_idx = self
            .find_chunk_index(start)
            .ok_or(RarError::InvalidOffset {
                offset: start,
                length: self.length,
            })?;
        let end_idx = self.find_chunk_index(end).ok_or(RarError::InvalidOffset {
            offset: end,
            length: self.length,
        })?;

        // Pre-allocate exact size needed
        let mut result = Vec::with_capacity((end - start + 1) as usize);

        for i in start_idx..=end_idx {
            let entry = &self.chunk_map[i];
            let chunk = &self.chunks[i];

            // Calculate the portion of this chunk we need
            let chunk_start = if i == start_idx {
                start - entry.start
            } else {
                0
            };
            let chunk_end = if i == end_idx {
                end - entry.start
            } else {
                chunk.length().saturating_sub(1)
            };

            let data = chunk.read_range(chunk_start, chunk_end).await?;
            result.extend_from_slice(&data);
        }

        Ok(result)
    }

    /// Create a streaming reader for a byte range.
    /// Yields chunks incrementally for backpressure-aware streaming.
    pub fn stream_range(&self, start: u64, end: u64) -> InnerFileStream<'_> {
        InnerFileStream::new(self, start, end)
    }

    /// Get chunk boundaries for a range (useful for prioritizing torrent pieces).
    /// Returns (chunk_index, chunk_start_offset, chunk_end_offset) for each chunk.
    pub fn get_chunk_ranges(&self, start: u64, end: u64) -> Vec<(usize, u64, u64)> {
        let start_idx = match self.find_chunk_index(start) {
            Some(i) => i,
            None => return vec![],
        };
        let end_idx = match self.find_chunk_index(end) {
            Some(i) => i,
            None => return vec![],
        };

        let mut ranges = Vec::with_capacity(end_idx - start_idx + 1);

        for i in start_idx..=end_idx {
            let entry = &self.chunk_map[i];
            let chunk = &self.chunks[i];

            let chunk_start = if i == start_idx {
                start - entry.start
            } else {
                0
            };
            let chunk_end = if i == end_idx {
                end - entry.start
            } else {
                chunk.length().saturating_sub(1)
            };

            // Convert to absolute offsets within the RAR volume
            let abs_start = chunk.start_offset + chunk_start;
            let abs_end = chunk.start_offset + chunk_end;

            ranges.push((i, abs_start, abs_end));
        }

        ranges
    }

    /// Translate a logical offset to (volume_index, volume_offset).
    /// Useful for mapping seek positions to torrent pieces.
    pub fn translate_offset(&self, offset: u64) -> Option<(usize, u64)> {
        let idx = self.find_chunk_index(offset)?;
        let entry = &self.chunk_map[idx];
        let chunk = &self.chunks[idx];

        let offset_in_chunk = offset - entry.start;
        let volume_offset = chunk.start_offset + offset_in_chunk;

        Some((idx, volume_offset))
    }
}

/// Streaming reader for InnerFile ranges.
/// Implements Stream for async iteration over chunks.
pub struct InnerFileStream<'a> {
    inner_file: &'a InnerFile,
    current_offset: u64,
    end_offset: u64,
    current_chunk_idx: Option<usize>,
    done: bool,
}

impl<'a> InnerFileStream<'a> {
    pub fn new(inner_file: &'a InnerFile, start: u64, end: u64) -> Self {
        let current_chunk_idx = inner_file.find_chunk_index(start);
        Self {
            inner_file,
            current_offset: start,
            end_offset: end.min(inner_file.length.saturating_sub(1)),
            current_chunk_idx,
            done: start > end || start >= inner_file.length,
        }
    }

    /// Get the next chunk of data (for manual iteration).
    /// Returns None when done.
    pub async fn next_chunk(&mut self) -> Option<Result<Vec<u8>>> {
        if self.done {
            return None;
        }

        let chunk_idx = self.current_chunk_idx?;
        let entry = self.inner_file.get_chunk_entry(chunk_idx)?;
        let chunk = self.inner_file.get_chunk(chunk_idx)?;

        // Calculate range within this chunk
        let chunk_start = self.current_offset - entry.start;
        let chunk_end = if self.end_offset <= entry.end {
            self.end_offset - entry.start
        } else {
            chunk.length().saturating_sub(1)
        };

        // Read the chunk data
        let result = chunk.read_range(chunk_start, chunk_end).await;

        match &result {
            Ok(data) => {
                self.current_offset += data.len() as u64;

                if self.current_offset > self.end_offset {
                    self.done = true;
                } else {
                    // Move to next chunk
                    self.current_chunk_idx = Some(chunk_idx + 1);
                    if chunk_idx + 1 >= self.inner_file.chunk_count() {
                        self.done = true;
                    }
                }
            }
            Err(_) => {
                self.done = true;
            }
        }

        Some(result)
    }

    /// Remaining bytes to read.
    pub fn remaining(&self) -> u64 {
        if self.done {
            0
        } else {
            self.end_offset.saturating_sub(self.current_offset) + 1
        }
    }

    /// Current read position.
    pub fn position(&self) -> u64 {
        self.current_offset
    }
}

/// Chunk info for streaming prioritization.
#[derive(Debug, Clone)]
pub struct StreamChunkInfo {
    pub chunk_index: usize,
    pub logical_start: u64,
    pub logical_end: u64,
    pub volume_start: u64,
    pub volume_end: u64,
    pub size: u64,
}

impl InnerFile {
    /// Get detailed chunk info for streaming prioritization.
    /// Useful for telling the torrent engine which pieces to prioritize.
    pub fn get_stream_chunks(&self, start: u64, end: u64) -> Vec<StreamChunkInfo> {
        let start_idx = match self.find_chunk_index(start) {
            Some(i) => i,
            None => return vec![],
        };
        let end_idx = match self.find_chunk_index(end) {
            Some(i) => i,
            None => return vec![],
        };

        let mut infos = Vec::with_capacity(end_idx - start_idx + 1);

        for i in start_idx..=end_idx {
            let entry = &self.chunk_map[i];
            let chunk = &self.chunks[i];

            let logical_start = if i == start_idx { start } else { entry.start };
            let logical_end = if i == end_idx { end } else { entry.end };

            let offset_in_chunk_start = logical_start - entry.start;
            let offset_in_chunk_end = logical_end - entry.start;

            infos.push(StreamChunkInfo {
                chunk_index: i,
                logical_start,
                logical_end,
                volume_start: chunk.start_offset + offset_in_chunk_start,
                volume_end: chunk.start_offset + offset_in_chunk_end,
                size: logical_end - logical_start + 1,
            });
        }

        infos
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_media::{FileMedia, ReadInterval};
    use std::sync::Arc;

    /// Mock FileMedia for testing
    struct MockFileMedia {
        name: String,
        length: u64,
        data: Vec<u8>,
    }

    impl MockFileMedia {
        fn new(name: &str, data: Vec<u8>) -> Self {
            Self {
                name: name.to_string(),
                length: data.len() as u64,
                data,
            }
        }
    }

    impl FileMedia for MockFileMedia {
        fn length(&self) -> u64 {
            self.length
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn read_range(
            &self,
            interval: ReadInterval,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = crate::error::Result<Vec<u8>>> + Send + '_>,
        > {
            let start = interval.start as usize;
            let end = (interval.end + 1) as usize;
            let data = self.data[start..end.min(self.data.len())].to_vec();
            Box::pin(async move { Ok(data) })
        }
    }

    fn create_test_chunks(chunk_sizes: &[u64]) -> Vec<RarFileChunk> {
        let mut chunks = Vec::new();
        for (i, &size) in chunk_sizes.iter().enumerate() {
            let data = vec![i as u8; size as usize];
            let media = Arc::new(MockFileMedia::new(&format!("chunk{}.rar", i), data));
            // Each chunk starts at offset 0 in its own file
            chunks.push(RarFileChunk::new(media, 0, size - 1));
        }
        chunks
    }

    #[test]
    fn test_binary_search_single_chunk() {
        let chunks = create_test_chunks(&[1000]);
        let file = InnerFile::new("test.mkv".to_string(), chunks, 0x30, 0, RarVersion::Rar4);

        assert_eq!(file.length, 1000);
        assert_eq!(file.find_chunk_index(0), Some(0));
        assert_eq!(file.find_chunk_index(500), Some(0));
        assert_eq!(file.find_chunk_index(999), Some(0));
        assert_eq!(file.find_chunk_index(1000), None); // Out of bounds
    }

    #[test]
    fn test_binary_search_multiple_chunks() {
        // 3 chunks: 0-99, 100-199, 200-299
        let chunks = create_test_chunks(&[100, 100, 100]);
        let file = InnerFile::new("test.mkv".to_string(), chunks, 0x30, 0, RarVersion::Rar4);

        assert_eq!(file.length, 300);

        // First chunk
        assert_eq!(file.find_chunk_index(0), Some(0));
        assert_eq!(file.find_chunk_index(50), Some(0));
        assert_eq!(file.find_chunk_index(99), Some(0));

        // Second chunk
        assert_eq!(file.find_chunk_index(100), Some(1));
        assert_eq!(file.find_chunk_index(150), Some(1));
        assert_eq!(file.find_chunk_index(199), Some(1));

        // Third chunk
        assert_eq!(file.find_chunk_index(200), Some(2));
        assert_eq!(file.find_chunk_index(250), Some(2));
        assert_eq!(file.find_chunk_index(299), Some(2));

        // Out of bounds
        assert_eq!(file.find_chunk_index(300), None);
    }

    #[test]
    fn test_binary_search_many_chunks() {
        // 100 chunks of 1000 bytes each = 100KB file
        let chunk_sizes: Vec<u64> = vec![1000; 100];
        let chunks = create_test_chunks(&chunk_sizes);
        let file = InnerFile::new("test.mkv".to_string(), chunks, 0x30, 0, RarVersion::Rar4);

        assert_eq!(file.length, 100_000);

        // Test seeking to various positions
        for i in 0..100 {
            let offset = i * 1000;
            assert_eq!(file.find_chunk_index(offset), Some(i as usize));
            assert_eq!(file.find_chunk_index(offset + 500), Some(i as usize));
            assert_eq!(file.find_chunk_index(offset + 999), Some(i as usize));
        }
    }

    #[test]
    fn test_translate_offset() {
        let chunks = create_test_chunks(&[100, 100, 100]);
        let file = InnerFile::new("test.mkv".to_string(), chunks, 0x30, 0, RarVersion::Rar4);

        // Each mock chunk starts at 0 in its volume
        let (idx, vol_offset) = file.translate_offset(0).unwrap();
        assert_eq!(idx, 0);
        assert_eq!(vol_offset, 0);

        let (idx, vol_offset) = file.translate_offset(150).unwrap();
        assert_eq!(idx, 1);
        assert_eq!(vol_offset, 50); // 150 - 100 = 50

        let (idx, vol_offset) = file.translate_offset(250).unwrap();
        assert_eq!(idx, 2);
        assert_eq!(vol_offset, 50); // 250 - 200 = 50
    }

    #[test]
    fn test_get_stream_chunks() {
        let chunks = create_test_chunks(&[100, 100, 100]);
        let file = InnerFile::new("test.mkv".to_string(), chunks, 0x30, 0, RarVersion::Rar4);

        // Request spanning all chunks
        let infos = file.get_stream_chunks(50, 250);
        assert_eq!(infos.len(), 3);

        assert_eq!(infos[0].chunk_index, 0);
        assert_eq!(infos[0].logical_start, 50);
        assert_eq!(infos[0].logical_end, 99);
        assert_eq!(infos[0].size, 50);

        assert_eq!(infos[1].chunk_index, 1);
        assert_eq!(infos[1].logical_start, 100);
        assert_eq!(infos[1].logical_end, 199);
        assert_eq!(infos[1].size, 100);

        assert_eq!(infos[2].chunk_index, 2);
        assert_eq!(infos[2].logical_start, 200);
        assert_eq!(infos[2].logical_end, 250);
        assert_eq!(infos[2].size, 51);
    }

    #[tokio::test]
    async fn test_read_range() {
        let chunks = create_test_chunks(&[100, 100, 100]);
        let file = InnerFile::new("test.mkv".to_string(), chunks, 0x30, 0, RarVersion::Rar4);

        // Read from middle chunk
        let data = file
            .read_range(ReadInterval {
                start: 150,
                end: 160,
            })
            .await
            .unwrap();
        assert_eq!(data.len(), 11);
        // All bytes should be 1 (from chunk 1)
        assert!(data.iter().all(|&b| b == 1));
    }

    #[tokio::test]
    async fn test_read_range_spanning_chunks() {
        let chunks = create_test_chunks(&[100, 100, 100]);
        let file = InnerFile::new("test.mkv".to_string(), chunks, 0x30, 0, RarVersion::Rar4);

        // Read spanning chunk 0 and 1
        let data = file
            .read_range(ReadInterval {
                start: 90,
                end: 110,
            })
            .await
            .unwrap();
        assert_eq!(data.len(), 21);

        // First 10 bytes from chunk 0
        assert!(data[..10].iter().all(|&b| b == 0));
        // Next 11 bytes from chunk 1
        assert!(data[10..].iter().all(|&b| b == 1));
    }
}
