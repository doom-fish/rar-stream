//! RAR5 Bit-level stream decoder.
//!
//! RAR5 uses a bit decoder (not range coder) for reading Huffman tables
//! and decompressing data. This is aligned byte reading with bit-level access.

#[cfg(feature = "parallel")]
use std::sync::Arc;

/// Padding bytes added to buffer to allow unchecked reads near end.
/// Must accommodate the largest unchecked read (8 bytes for
/// get_value_high32). Since move_pos clamps pos to data_len,
/// reads at data_len + 8 are always within bounds.
const BUFFER_PADDING: usize = 16;

/// Bit-level stream decoder for RAR5 decompression.
pub struct BitDecoder {
    /// Input buffer (padded with BUFFER_PADDING bytes for safe unchecked reads)
    #[cfg(feature = "parallel")]
    buf: Arc<Vec<u8>>,
    #[cfg(not(feature = "parallel"))]
    buf: Vec<u8>,
    /// Original data length (before padding)
    data_len: usize,
    /// Current position in buffer
    pos: usize,
    /// Current bit position (0-7)
    bit_pos: usize,
    /// Block end position (byte)
    block_end: usize,
    /// Block end bit position (0-7)
    block_end_bits: usize,
    /// Pre-computed block end in total bits for fast comparison
    block_end_total_bits: usize,
}

impl BitDecoder {
    /// Create a new bit decoder from input data.
    /// Adds padding to allow unchecked reads near buffer end.
    pub fn new(input: &[u8]) -> Self {
        let data_len = input.len();
        let mut buf = Vec::with_capacity(data_len + BUFFER_PADDING);
        buf.extend_from_slice(input);
        buf.resize(data_len + BUFFER_PADDING, 0xFF);
        Self {
            #[cfg(feature = "parallel")]
            buf: Arc::new(buf),
            #[cfg(not(feature = "parallel"))]
            buf,
            data_len,
            pos: 0,
            bit_pos: 0,
            block_end: data_len,
            block_end_bits: 0,
            block_end_total_bits: data_len * 8,
        }
    }

    /// Create a new decoder sharing the same buffer.
    /// This is O(1) - just increments reference count.
    #[cfg(feature = "parallel")]
    #[inline]
    pub fn clone_view(&self) -> Self {
        Self {
            buf: Arc::clone(&self.buf),
            data_len: self.data_len,
            pos: 0,
            bit_pos: 0,
            block_end: self.data_len,
            block_end_bits: 0,
            block_end_total_bits: self.data_len * 8,
        }
    }

    /// Align to byte boundary.
    pub fn align_to_byte(&mut self) {
        if self.bit_pos != 0 {
            self.pos += 1;
            self.bit_pos = 0;
        }
    }

    /// Read a byte when already aligned.
    #[inline]
    pub fn read_byte_aligned(&mut self) -> u8 {
        if self.pos >= self.data_len {
            return 0;
        }
        // SAFETY: pos < data_len, which is within allocated buffer
        let b = unsafe { *self.buf.get_unchecked(self.pos) };
        self.pos += 1;
        b
    }

    /// Read up to 9 bits with fixed mask.
    #[inline(always)]
    pub fn read_bits_9fix(&mut self, num_bits: usize) -> u32 {
        if num_bits == 0 {
            return 0;
        }
        let mask = (1u32 << num_bits) - 1;
        // SAFETY: pos is clamped to data_len, buffer has BUFFER_PADDING bytes
        // after data_len, so pos+1 is always within bounds.
        let v = unsafe {
            let p = self.pos.min(self.data_len);
            let b0 = *self.buf.get_unchecked(p) as u32;
            let b1 = *self.buf.get_unchecked(p + 1) as u32;
            (b0 << 8) | b1
        };
        let total_bits = num_bits + self.bit_pos;
        let result = (v >> (16 - total_bits)) & mask;
        self.pos += total_bits >> 3;
        self.bit_pos = total_bits & 7;
        if self.pos > self.data_len {
            self.pos = self.data_len;
        }
        result
    }

    /// Read up to 9 bits.
    #[inline(always)]
    pub fn read_bits_9(&mut self, num_bits: usize) -> u32 {
        if num_bits == 0 {
            return 0;
        }
        // SAFETY: pos is clamped to data_len, buffer has BUFFER_PADDING bytes
        // after data_len, so pos+1 is always within bounds.
        let v = unsafe {
            let p = self.pos.min(self.data_len);
            let b0 = *self.buf.get_unchecked(p) as u32;
            let b1 = *self.buf.get_unchecked(p + 1) as u32;
            (b0 << 8) | b1
        };
        let v = v & (0xFFFF >> self.bit_pos);
        let total_bits = num_bits + self.bit_pos;
        let result = v >> (16 - total_bits);
        self.pos += total_bits >> 3;
        self.bit_pos = total_bits & 7;
        if self.pos > self.data_len {
            self.pos = self.data_len;
        }
        result
    }

    /// Get 16 bits for Huffman decoding (without advancing).
    /// Returns bits in the same format as unrar's getbits(): 16-bit value with
    /// bit at (InAddr,InBit) at the highest position.
    #[inline(always)]
    pub fn getbits(&self) -> u32 {
        // SAFETY: pos clamped to data_len, buffer has BUFFER_PADDING bytes
        // after data_len, so 4-byte read is always within bounds.
        unsafe {
            let p = self.pos.min(self.data_len);
            let ptr = self.buf.as_ptr().add(p) as *const u32;
            let v = u32::from_be(ptr.read_unaligned());
            (v >> (16 - self.bit_pos)) & 0xFFFF
        }
    }

    /// Get up to 15 bits for Huffman decoding (without advancing).
    #[inline(always)]
    pub fn get_value_15(&self) -> u32 {
        // SAFETY: pos clamped to data_len, buffer has BUFFER_PADDING bytes
        unsafe {
            let p = self.pos.min(self.data_len);
            let b0 = *self.buf.get_unchecked(p) as u32;
            let b1 = *self.buf.get_unchecked(p + 1) as u32;
            let b2 = *self.buf.get_unchecked(p + 2) as u32;
            let v = (b0 << 16) | (b1 << 8) | b2;
            (v >> (9 - self.bit_pos)) & 0x7FFF
        }
    }

    /// Move position by num_bits.
    #[inline(always)]
    pub fn move_pos(&mut self, num_bits: usize) {
        let total = num_bits + self.bit_pos;
        self.pos += total >> 3;
        self.bit_pos = total & 7;
        // Clamp to data_len so subsequent reads stay within the padded buffer.
        // Callers check is_eof() to detect end-of-data.
        if self.pos > self.data_len {
            self.pos = self.data_len;
        }
    }

    /// Get value in high 32 bits for extended reading (matching unrar getbits32).
    /// Returns 32 bits with the first available bit at bit 31.
    #[inline(always)]
    pub fn get_value_high32(&self) -> u32 {
        // SAFETY: pos clamped to data_len, buffer has BUFFER_PADDING (>=16)
        // bytes after data_len, so 8-byte read is always within bounds.
        unsafe {
            let p = self.pos.min(self.data_len);
            let ptr = self.buf.as_ptr().add(p);
            let v = u64::from_be((ptr as *const u64).read_unaligned());
            ((v << self.bit_pos) >> 32) as u32
        }
    }

    /// Read N bits from pre-read high value.
    /// Assumes v was obtained from get_value_high32().
    #[inline(always)]
    pub fn read_bits_big(&mut self, num_bits: usize, v: u32) -> u32 {
        if num_bits == 0 {
            return 0;
        }
        let result = v >> (32 - num_bits);
        self.move_pos(num_bits);
        result
    }

    /// Set block end position.
    /// `end` is the byte position of the last byte in the block.
    /// `bit_size` is the number of valid bits in that last byte (1-8).
    pub fn set_block_end(&mut self, end: usize, bit_size: usize) {
        self.block_end = end;
        self.block_end_bits = bit_size;
        // Pre-compute total bits for fast comparison
        self.block_end_total_bits = end * 8 + bit_size;
    }

    /// Check if reading has exceeded block boundary.
    /// Uses pre-computed total bits for a single comparison.
    #[inline(always)]
    pub fn is_block_over_read(&self) -> bool {
        // Current position in total bits
        self.pos * 8 + self.bit_pos >= self.block_end_total_bits
    }

    /// Current byte position.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Current bit position within byte (0-7).
    pub fn bit_pos(&self) -> usize {
        self.bit_pos
    }

    /// Set byte position (resets bit position to 0).
    pub fn set_position(&mut self, pos: usize) {
        self.pos = pos.min(self.data_len);
        self.bit_pos = 0;
    }

    /// Set byte position and bit position within that byte.
    pub fn set_position_with_bit(&mut self, pos: usize, bit_pos: usize) {
        self.pos = pos.min(self.data_len);
        self.bit_pos = bit_pos;
    }

    /// Check if EOF reached (past all input data, not just current block).
    pub fn is_eof(&self) -> bool {
        self.pos >= self.data_len
    }

    /// Raw pointer to the underlying buffer data.
    pub fn buf_ptr(&self) -> *const u8 {
        self.buf.as_ptr()
    }

    /// Total length of the underlying buffer (including padding).
    pub fn buf_len(&self) -> usize {
        self.buf.len()
    }

    /// Pre-computed block end in total bits.
    pub fn block_end_total_bits(&self) -> usize {
        self.block_end_total_bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_bits() {
        let data = [0xC6, 0xF9, 0x65, 0x30];
        let mut decoder = BitDecoder::new(&data);

        // Read 4 bits: should be 0xC = 12
        let v = decoder.read_bits_9fix(4);
        assert_eq!(v, 0xC);

        // Read 4 more bits: should be 0x6 = 6
        let v = decoder.read_bits_9fix(4);
        assert_eq!(v, 0x6);
    }

    #[test]
    fn test_read_byte_aligned() {
        let data = [0xAB, 0xCD, 0xEF];
        let mut decoder = BitDecoder::new(&data);

        assert_eq!(decoder.read_byte_aligned(), 0xAB);
        assert_eq!(decoder.read_byte_aligned(), 0xCD);
        assert_eq!(decoder.read_byte_aligned(), 0xEF);
    }
}
