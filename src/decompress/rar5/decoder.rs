//! RAR5 decoder implementation.
//!
//! RAR5 compression is based on LZSS with range coding and optional filters.
//! Dictionary sizes can be up to 4GB.

use crate::decompress::DecompressError;

/// RAR5 decoder for decompressing RAR5 archives.
pub struct Rar5Decoder {
    /// Sliding window buffer (dictionary)
    window: Vec<u8>,
    /// Current position in window
    window_pos: usize,
    /// Dictionary size (power of 2)
    dict_size: usize,
}

impl Rar5Decoder {
    /// Create a new RAR5 decoder with default dictionary size.
    pub fn new() -> Self {
        Self::with_dict_size(22) // 4MB default
    }

    /// Create a new RAR5 decoder with specified dictionary size.
    /// `dict_size_log` is the power of 2 (e.g., 22 = 4MB).
    pub fn with_dict_size(dict_size_log: u8) -> Self {
        let dict_size = 1usize << dict_size_log;
        Self {
            window: vec![0u8; dict_size],
            window_pos: 0,
            dict_size,
        }
    }

    /// Reset decoder state for reuse.
    pub fn reset(&mut self) {
        self.window_pos = 0;
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
        _is_solid: bool,
    ) -> Result<Vec<u8>, DecompressError> {
        if method == 0 {
            return self.decompress_stored(input, unpacked_size);
        }

        // TODO: Implement RAR5 LZSS + range coder decompression
        // For now, return an error indicating RAR5 compression is not yet supported
        Err(DecompressError::UnsupportedMethod(method | 0x50)) // 0x50 = RAR5 marker
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
    fn test_unsupported_method() {
        let mut decoder = Rar5Decoder::new();
        let input = b"compressed data";
        let result = decoder.decompress(input, 100, 3, false);
        assert!(result.is_err());
    }
}
