//! RAR5 decoder implementation.
//!
//! RAR5 compression is based on LZSS with Huffman coding and optional filters.
//! Dictionary sizes can be up to 4GB.

use super::bit_decoder::BitDecoder;
use super::block_decoder::Rar5BlockDecoder;
use super::filter::{apply_filter, UnpackFilter};
use crate::decompress::DecompressError;

/// RAR5 decoder for decompressing RAR5 archives.
pub struct Rar5Decoder {
    /// Block decoder with sliding window
    block_decoder: Rar5BlockDecoder,
    /// Dictionary size log (power of 2)
    dict_size_log: u8,
    /// Pending filters
    filters: Vec<UnpackFilter>,
    /// Written file size (for filter offset calculation)
    written_file_size: u64,
}

impl Rar5Decoder {
    /// Create a new RAR5 decoder with default dictionary size.
    pub fn new() -> Self {
        Self::with_dict_size(22) // 4MB default
    }

    /// Create a new RAR5 decoder with specified dictionary size.
    /// `dict_size_log` is the power of 2 (e.g., 22 = 4MB).
    pub fn with_dict_size(dict_size_log: u8) -> Self {
        
        Self {
            block_decoder: Rar5BlockDecoder::new(dict_size_log),
            dict_size_log,
            filters: Vec::new(),
            written_file_size: 0,
        }
    }

    /// Reset decoder state for reuse.
    pub fn reset(&mut self) {
        self.block_decoder.reset();
        self.filters.clear();
        self.written_file_size = 0;
    }

    /// Decompress stored (uncompressed) data.
    /// For method 0, the data is simply copied.
    pub fn decompress_stored(
        &mut self,
        input: &[u8],
        unpacked_size: u64,
    ) -> Result<Vec<u8>, DecompressError> {
        if input.len() < unpacked_size as usize {
            return Err(DecompressError::IncompleteData);
        }
        Ok(input[..unpacked_size as usize].to_vec())
    }

    /// Apply pending filters to output data.
    fn apply_filters(&mut self, output: &mut [u8]) {
        // Sort filters by block start position
        self.filters.sort_by_key(|f| f.block_start);

        for filter in &self.filters {
            let start = filter.block_start;
            let end = start + filter.block_length;

            if end <= output.len() {
                // Apply filter - may be in-place or return new buffer
                let block = &mut output[start..end];
                let filtered =
                    apply_filter(block, filter, self.written_file_size + start as u64);

                // If filtered is non-empty, it's a Delta filter result (not in-place)
                if !filtered.is_empty() {
                    output[start..end].copy_from_slice(&filtered);
                }
            }
        }

        self.filters.clear();
    }

    /// Decompress RAR5 compressed data.
    ///
    /// # Arguments
    /// * `input` - Compressed data
    /// * `unpacked_size` - Expected decompressed size
    /// * `method` - Compression method (0 = stored, 1-5 = compression levels)
    /// * `is_solid` - Whether this is part of a solid archive
    pub fn decompress(
        &mut self,
        input: &[u8],
        unpacked_size: u64,
        method: u8,
        is_solid: bool,
    ) -> Result<Vec<u8>, DecompressError> {
        
        if method == 0 {
            return self.decompress_stored(input, unpacked_size);
        }

        // Reset for non-solid archives
        if !is_solid {
            self.block_decoder.reset();
            self.filters.clear();
            self.written_file_size = 0;
        }

        // Initialize bit decoder with compressed data
        let mut bits = BitDecoder::new(input);

        // Decode blocks until we have enough output
        let new_filters = self
            .block_decoder
            .decode_block(&mut bits, unpacked_size as usize)?;

        // Collect any filters returned
        self.filters.extend(new_filters);

        // Get decompressed output (take_output is more efficient)
        let mut output = self.block_decoder.take_output();

        if output.len() != unpacked_size as usize {
            return Err(DecompressError::IncompleteData);
        }

        // Apply any pending filters
        if !self.filters.is_empty() {
            self.apply_filters(&mut output);
        }

        self.written_file_size += output.len() as u64;

        Ok(output)
    }

    /// Decompress RAR5 data using parallel decompression.
    ///
    /// # Arguments
    /// * `input` - Compressed data
    /// * `unpacked_size` - Expected decompressed size
    #[cfg(feature = "parallel")]
    pub fn decompress_parallel(
        &mut self,
        input: &[u8],
        unpacked_size: u64,
    ) -> Result<Vec<u8>, DecompressError> {
        use super::block_decoder::ParallelConfig;
        
        // Reset state
        self.block_decoder.reset();
        self.filters.clear();
        self.written_file_size = 0;
        
        // Use parallel decode with default config
        let config = ParallelConfig::default();
        let output = self.block_decoder.decode_parallel_with_config(
            input,
            unpacked_size as usize,
            &config,
        )?;
        
        if output.len() != unpacked_size as usize {
            return Err(DecompressError::IncompleteData);
        }
        
        self.written_file_size += output.len() as u64;
        
        Ok(output)
    }
}

impl Default for Rar5Decoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress_stored() {
        let mut decoder = Rar5Decoder::new();
        let input = b"Hello, World!";
        let result = decoder.decompress_stored(input, 13).unwrap();
        assert_eq!(result, b"Hello, World!");
    }

    #[test]
    fn test_decompress_stored_with_extra_data() {
        let mut decoder = Rar5Decoder::new();
        let input = b"Hello, World! Extra data here";
        let result = decoder.decompress_stored(input, 13).unwrap();
        assert_eq!(result, b"Hello, World!");
    }

    #[test]
    fn test_decompress_compressed() {
        // Test that compressed decompression runs without panicking
        // (actual correctness requires proper RAR5 test data)
        let mut decoder = Rar5Decoder::new();
        // Fake compressed data - won't produce correct output but tests the path
        let input = vec![0u8; 100];
        let result = decoder.decompress(&input, 10, 3, false);
        // Should fail with error (invalid block header)
        assert!(result.is_err());
    }

    #[test]
    fn test_decompress_real_file() {
        // Test with actual RAR5 compressed data from fixtures
        let data = std::fs::read("__fixtures__/rar5/compressed.rar").unwrap();

        // Compressed data starts at byte 75, 104 bytes
        let compressed = &data[75..179];
        let unpacked_size = 152;
        let dict_bits = 17; // 128KB dictionary

        let mut decoder = Rar5Decoder::new();
        let result = decoder.decompress(compressed, unpacked_size, dict_bits, false);

        match result {
            Ok(output) => {
                assert_eq!(output.len(), 152, "output size should match unpacked size");
                let text = std::str::from_utf8(&output).expect("output should be valid UTF-8");
                assert!(
                    text.starts_with("This is a test file"),
                    "should start with expected text"
                );
                assert!(text.contains("hello hello"), "should contain repeated text");
            }
            Err(e) => {
                panic!("Decompression failed: {:?}", e);
            }
        }
    }
}

    #[test]
    fn test_decompress_alpine_rar5() {
        // Parse alpine_rar5.rar and decompress
        let data = std::fs::read("__fixtures__/large/alpine_rar5.rar").unwrap();
        
        // Data starts at byte 76, 2939435 bytes, unpacked 8130560 bytes
        let compressed = &data[76..76+2939435];
        let unpacked_size = 8130560u64;
        let dict_log = 23u8;
        let method = 3u8;
        
        let mut decoder = Rar5Decoder::with_dict_size(dict_log);
        let result = decoder.decompress(compressed, unpacked_size, method, false);
        
        match result {
            Ok(output) => {
                assert_eq!(output.len(), unpacked_size as usize);
                
                // Compare with original
                let original = std::fs::read("__fixtures__/large/alpine.tar").unwrap();
                
                // Find first difference
                for (i, (a, b)) in output.iter().zip(original.iter()).enumerate() {
                    if a != b {
                        panic!("Mismatch at byte {}: got {:02x}, expected {:02x}", i, a, b);
                    }
                }
                
                assert_eq!(output, original, "Output should match original");
            }
            Err(e) => panic!("Decompression failed: {:?}", e),
        }
    }
