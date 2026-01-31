//! RAR5 decoder implementation.
//!
//! RAR5 compression is based on LZSS with Huffman coding and optional filters.
//! Dictionary sizes can be up to 4GB.

use super::bit_decoder::BitDecoder;
use super::block_decoder::Rar5BlockDecoder;
use crate::decompress::DecompressError;

/// RAR5 decoder for decompressing RAR5 archives.
pub struct Rar5Decoder {
    /// Block decoder with sliding window
    block_decoder: Rar5BlockDecoder,
    /// Dictionary size log (power of 2)
    dict_size_log: u8,
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
        }
    }

    /// Reset decoder state for reuse.
    pub fn reset(&mut self) {
        self.block_decoder.reset();
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
        }

        // Initialize bit decoder with compressed data
        let mut bits = BitDecoder::new(input);

        // Decode blocks until we have enough output
        let start_pos = 0;
        self.block_decoder
            .decode_block(&mut bits, unpacked_size as usize)?;

        // Get decompressed output
        let output = self.block_decoder.get_output(start_pos, unpacked_size as usize);

        if output.len() != unpacked_size as usize {
            return Err(DecompressError::IncompleteData);
        }

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
                assert!(text.starts_with("This is a test file"), "should start with expected text");
                assert!(text.contains("hello hello"), "should contain repeated text");
            }
            Err(e) => {
                panic!("Decompression failed: {:?}", e);
            }
        }
    }
}
