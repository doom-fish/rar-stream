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
    /// Total bytes written to window
    total_written: u64,
    /// How much has been flushed to output
    flushed_pos: u64,
    /// Output buffer for final result
    output: Vec<u8>,
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
            flushed_pos: 0,
            output: Vec::new(),
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
        self.output.clear();
        // No need to clear window - we validate reads against total_written
    }

    /// Enable output accumulation for extracting files larger than window.
    pub fn enable_output(&mut self, capacity: usize) {
        self.output = Vec::with_capacity(capacity);
    }

    /// Write a literal byte to the window.
    #[inline]
    pub fn write_literal(&mut self, byte: u8) {
        #[cfg(test)]
        {
            if self.total_written == 1498598 {
                eprintln!("!!! WRITE_LITERAL at 1498598: byte=0x{:02x}", byte);
            }
            // Also check if we're writing to window index that corresponds to output 1498598
            if (self.pos as u64) == 1498598 {
                eprintln!("!!! WRITE_LITERAL to window[1498598] at total_written={}: byte=0x{:02x}", self.total_written, byte);
            }
        }
        self.window[self.pos] = byte;
        // Don't write to output during decode - will be flushed later after filters
        self.pos = (self.pos + 1) & self.mask;
        self.total_written += 1;
    }
    
    /// Flush data from window to output, up to the given absolute position.
    /// This is called after filters have been applied.
    pub fn flush_to_output(&mut self, up_to: u64) {
        let current_output_len = self.output.len() as u64;
        if up_to <= current_output_len {
            return; // Already flushed
        }
        
        let flush_start = current_output_len as usize;
        let flush_end = up_to as usize;
        let flush_len = flush_end - flush_start;
        let window_start = flush_start & self.mask;
        
        #[cfg(test)]
        {
            eprintln!("FLUSH: from {} to {} (len {})", flush_start, flush_end, flush_len);
            // Debug: show bytes around mismatch position 1498598
            if flush_start <= 1498598 && flush_end >= 1498598 {
                let mut bytes = Vec::new();
                for pos in 1498590..1498610.min(flush_end) {
                    let window_idx = pos & self.mask;
                    bytes.push(self.window[window_idx]);
                }
                eprintln!("  window bytes around 1498598: {:02x?}", bytes);
                eprintln!("  window_start={}, mask=0x{:x}", window_start, self.mask);
                eprintln!("  total_written at flush time={}", self.total_written);
            }
        }
        
        // Reserve space
        self.output.reserve(flush_len);
        
        // Copy from window to output
        // Note: this assumes flush positions don't span more than window size
        for i in 0..flush_len {
            let window_idx = (window_start + i) & self.mask;
            self.output.push(self.window[window_idx]);
        }
        
        self.flushed_pos = up_to;
    }
    
    /// Get mutable access to the window for filter execution.
    pub fn window_mut(&mut self) -> &mut [u8] {
        &mut self.window
    }
    
    /// Get the window mask (for filter positioning).
    pub fn window_mask(&self) -> u32 {
        self.mask as u32
    }
    
    /// Get how much has been flushed to output.
    pub fn flushed_pos(&self) -> u64 {
        self.flushed_pos
    }
    
    /// Write filtered data directly to output, bypassing the window.
    /// This is used for VM filter output which should NOT modify the window.
    pub fn write_filtered_to_output(&mut self, data: &[u8], position: u64) {
        // Ensure we're at the right position - if not, we might have missed a flush
        let current_len = self.output.len() as u64;
        if current_len < position {
            // Need to flush unfiltered data from window up to this position first
            // This can happen if there's data between the last flush and the filter start
            let window_start = current_len as usize;
            let flush_len = (position - current_len) as usize;
            self.output.reserve(flush_len);
            for i in 0..flush_len {
                let window_idx = (window_start + i) & self.mask;
                self.output.push(self.window[window_idx]);
            }
        }
        self.output.extend_from_slice(data);
        self.flushed_pos = position + data.len() as u64;
    }
    
    /// Get read-only access to the window for filter execution.
    pub fn window(&self) -> &[u8] {
        &self.window
    }

    /// Copy bytes from a previous position in the window.
    /// Optimized for both overlapping and non-overlapping copies.
    #[inline]
    pub fn copy_match(&mut self, distance: u32, length: u32) -> Result<()> {
        #[cfg(test)]
        {
            let pos = self.total_written as usize;
            let end_pos = pos + length as usize;
            // Check if this match covers position 1498598
            if pos <= 1498598 && end_pos > 1498598 {
                let offset_into_match = 1498598 - pos;
                let src_pos = ((pos as u32).wrapping_sub(distance)) as usize & self.mask;
                eprintln!("!!! COPY_MATCH covers 1498598: pos={}, dist={}, len={}, offset_into_match={}", 
                    pos, distance, length, offset_into_match);
                eprintln!("  src bytes to copy: {:02x?}", &self.window[src_pos..(src_pos + length as usize).min(self.window.len())]);
                eprintln!("  byte that will be at 1498598: 0x{:02x} (from src_pos+offset={})", 
                    self.window[(src_pos + offset_into_match) & self.mask], src_pos + offset_into_match);
            }
            // Also check if window index 1498598 is written (might be a second write)
            let win_start = self.pos;
            let win_end = (self.pos + length as usize) & self.mask;
            if (win_start <= 1498598 && 1498598 < win_start + length as usize) || 
               (win_start > 1498598 && win_end > 1498598 && win_end < win_start) {
                eprintln!("!!! COPY_MATCH to window[1498598]: total_written={}, dist={}, len={}", 
                    self.total_written, distance, length);
            }
        }
        
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
            self.window
                .copy_within(src_start..src_start + len, self.pos);
            self.pos += len;
            self.total_written += length as u64;
            return Ok(());
        }

        // Medium path: overlapping but no wrapping - use copy_within in chunks
        // Only worthwhile if we can copy at least 8 bytes at a time
        if self.pos + len <= self.window.len() && self.pos >= dist && dist >= 8 {
            let src_start = self.pos - dist;
            let mut copied = 0;
            while copied < len {
                let chunk = (len - copied).min(dist);
                self.window.copy_within(src_start..src_start + chunk, self.pos + copied);
                copied += chunk;
            }
            self.pos += len;
            self.total_written += length as u64;
            return Ok(());
        }

        // Slow path: handle wrapping or very short distance copies byte-by-byte
        // Use unchecked access since we've already validated distance
        let src_pos = (self.pos.wrapping_sub(dist)) & self.mask;
        let window_ptr = self.window.as_mut_ptr();

        for i in 0..len {
            let src_idx = (src_pos + i) & self.mask;
            let dest_idx = (self.pos + i) & self.mask;
            // SAFETY: src_idx and dest_idx are always < window.len() due to mask
            unsafe {
                let byte = *window_ptr.add(src_idx);
                *window_ptr.add(dest_idx) = byte;
            }
        }
        self.pos = (self.pos + len) & self.mask;

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
    /// Call this after decompression to get the output.
    pub fn get_output(&self, start: u64, len: usize) -> Vec<u8> {
        // If we have accumulated output, use it
        if !self.output.is_empty() {
            let start = start as usize;
            let end = (start + len).min(self.output.len());
            return self.output[start..end].to_vec();
        }
        
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
    
    /// Take ownership of the accumulated output buffer.
    /// More efficient than get_output() when you need all output.
    pub fn take_output(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.output)
    }
    
    /// Get read access to the accumulated output buffer.
    pub fn output(&self) -> &[u8] {
        &self.output
    }

    /// Get mutable access to the output buffer for filter execution.
    pub fn output_mut(&mut self) -> &mut [u8] {
        &mut self.output
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
