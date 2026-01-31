//! LZSS sliding window decoder.
//!
//! Implements the dictionary-based decompression used in RAR.

use super::{DecompressError, Result};

/// Window size for RAR29 (2MB).
pub const WINDOW_SIZE_29: usize = 0x200000;

/// Window size for RAR50 (up to 4GB, but we use 64MB max for memory).
pub const WINDOW_SIZE_50: usize = 0x4000000;

/// LZSS sliding window decoder.
pub struct LzssDecoder {
    /// Sliding window buffer
    window: Vec<u8>,
    /// Window size mask for wrap-around
    mask: usize,
    /// Current write position in window
    pos: usize,
    /// Total bytes written
    total_written: u64,
}

impl LzssDecoder {
    /// Create a new LZSS decoder with the specified window size.
    pub fn new(window_size: usize) -> Self {
        debug_assert!(window_size.is_power_of_two());
        Self {
            window: vec![0; window_size],
            mask: window_size - 1,
            pos: 0,
            total_written: 0,
        }
    }

    /// Create decoder for RAR 2.9 format.
    pub fn rar29() -> Self {
        Self::new(WINDOW_SIZE_29)
    }

    /// Create decoder for RAR 5.0 format.
    pub fn rar50() -> Self {
        Self::new(WINDOW_SIZE_50)
    }

    /// Reset the decoder for reuse, avoiding reallocation.
    /// Note: Window contents are NOT cleared - we only read after writing.
    #[inline]
    pub fn reset(&mut self) {
        self.pos = 0;
        self.total_written = 0;
        // No need to clear window - we validate reads against total_written
    }

    /// Write a literal byte to the output.
    #[inline]
    pub fn write_literal(&mut self, byte: u8) {
        self.window[self.pos] = byte;
        self.pos = (self.pos + 1) & self.mask;
        self.total_written += 1;
    }

    /// Copy bytes from a previous position in the window.
    /// Optimized for both overlapping and non-overlapping copies.
    #[inline]
    pub fn copy_match(&mut self, distance: u32, length: u32) -> Result<()> {
        // Validate distance against bytes actually written, not window size
        if distance == 0 || distance as u64 > self.total_written {
            return Err(DecompressError::InvalidBackReference {
                offset: distance,
                position: self.pos as u32,
            });
        }

        let len = length as usize;
        let dist = distance as usize;
        
        // Fast path: copy doesn't wrap around window boundary and doesn't overlap
        if dist >= len && self.pos + len <= self.window.len() && self.pos >= dist {
            // Non-overlapping, non-wrapping: use copy_within for speed
            let src_start = self.pos - dist;
            self.window.copy_within(src_start..src_start + len, self.pos);
            self.pos += len;
            self.total_written += length as u64;
            return Ok(());
        }

        // Slow path: handle overlapping or wrapping copies byte-by-byte
        let src_pos = (self.pos.wrapping_sub(dist)) & self.mask;

        for i in 0..len {
            let src_idx = (src_pos + i) & self.mask;
            let byte = self.window[src_idx];
            self.window[self.pos] = byte;
            self.pos = (self.pos + 1) & self.mask;
        }

        self.total_written += length as u64;
        Ok(())
    }

    /// Get the current window position.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Get total bytes written.
    pub fn total_written(&self) -> u64 {
        self.total_written
    }

    /// Get a byte at the specified offset from current position (going back).
    #[inline]
    pub fn get_byte_at_offset(&self, offset: usize) -> u8 {
        let idx = (self.pos.wrapping_sub(offset)) & self.mask;
        self.window[idx]
    }

    /// Extract decompressed data from the window.
    /// Call this after decompression to get the output.
    pub fn get_output(&self, start: u64, len: usize) -> Vec<u8> {
        let mut output = Vec::with_capacity(len);
        let window_len = self.window.len();

        // Calculate start position in window
        let start_pos = if self.total_written <= window_len as u64 {
            start as usize
        } else {
            // Window has wrapped
            let _written_in_window = self.total_written as usize % window_len;
            let offset = (self.total_written - start) as usize;
            if offset > window_len {
                return output; // Data no longer in window
            }
            (self.pos.wrapping_sub(offset)) & self.mask
        };

        for i in 0..len {
            let idx = (start_pos + i) & self.mask;
            output.push(self.window[idx]);
        }

        output
    }

    /// Get the most recent `len` bytes from the window.
    pub fn get_recent(&self, len: usize) -> Vec<u8> {
        let actual_len = len.min(self.total_written as usize);
        let mut output = Vec::with_capacity(actual_len);

        let start = (self.pos.wrapping_sub(actual_len)) & self.mask;
        for i in 0..actual_len {
            let idx = (start + i) & self.mask;
            output.push(self.window[idx]);
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_output() {
        let mut decoder = LzssDecoder::new(256);

        decoder.write_literal(b'H');
        decoder.write_literal(b'e');
        decoder.write_literal(b'l');
        decoder.write_literal(b'l');
        decoder.write_literal(b'o');

        assert_eq!(decoder.total_written(), 5);
        assert_eq!(decoder.get_recent(5), b"Hello");
    }

    #[test]
    fn test_copy_match() {
        let mut decoder = LzssDecoder::new(256);

        // Write "abc"
        decoder.write_literal(b'a');
        decoder.write_literal(b'b');
        decoder.write_literal(b'c');

        // Copy from distance 3, length 6 -> "abcabc"
        decoder.copy_match(3, 6).unwrap();

        assert_eq!(decoder.total_written(), 9);
        assert_eq!(decoder.get_recent(9), b"abcabcabc");
    }

    #[test]
    fn test_overlapping_copy() {
        let mut decoder = LzssDecoder::new(256);

        // Write "a"
        decoder.write_literal(b'a');

        // Copy from distance 1, length 5 -> "aaaaa"
        decoder.copy_match(1, 5).unwrap();

        assert_eq!(decoder.get_recent(6), b"aaaaaa");
    }

    #[test]
    fn test_invalid_distance() {
        let mut decoder = LzssDecoder::new(256);
        decoder.write_literal(b'a');

        // Distance 0 is invalid
        assert!(decoder.copy_match(0, 1).is_err());
    }
}
