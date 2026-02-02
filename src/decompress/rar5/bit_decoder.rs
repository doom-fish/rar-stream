//! RAR5 Bit-level stream decoder.
//!
//! RAR5 uses a bit decoder (not range coder) for reading Huffman tables
//! and decompressing data. This is aligned byte reading with bit-level access.

/// Padding bytes added to buffer to allow unchecked reads near end
const BUFFER_PADDING: usize = 8;

/// Bit-level stream decoder for RAR5 decompression.
pub struct BitDecoder {
    /// Input buffer (padded with BUFFER_PADDING bytes for safe unchecked reads)
    buf: Vec<u8>,
    /// Current position in buffer
    pos: usize,
    /// Current bit position (0-7)
    bit_pos: usize,
    /// Block end position (byte)
    block_end: usize,
    /// Block end bit position (0-7)
    block_end_bits: usize,
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
            buf,
            pos: 0,
            bit_pos: 0,
            block_end: data_len,
            block_end_bits: 0,
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
        // SAFETY: buffer is padded
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
        // SAFETY: buffer is padded with 8 bytes
        let v = unsafe {
            let b0 = *self.buf.get_unchecked(self.pos) as u32;
            let b1 = *self.buf.get_unchecked(self.pos + 1) as u32;
            (b0 << 8) | b1
        };
        let total_bits = num_bits + self.bit_pos;
        let result = (v >> (16 - total_bits)) & mask;
        self.pos += total_bits >> 3;
        self.bit_pos = total_bits & 7;
        result
    }

    /// Read up to 9 bits.
    #[inline(always)]
    pub fn read_bits_9(&mut self, num_bits: usize) -> u32 {
        if num_bits == 0 {
            return 0;
        }
        // SAFETY: buffer is padded with 8 bytes
        let v = unsafe {
            let b0 = *self.buf.get_unchecked(self.pos) as u32;
            let b1 = *self.buf.get_unchecked(self.pos + 1) as u32;
            (b0 << 8) | b1
        };
        let v = v & (0xFFFF >> self.bit_pos);
        let total_bits = num_bits + self.bit_pos;
        let result = v >> (16 - total_bits);
        self.pos += total_bits >> 3;
        self.bit_pos = total_bits & 7;
        result
    }

    /// Get up to 15 bits for Huffman decoding (without advancing).
    #[inline(always)]
    pub fn get_value_15(&self) -> u32 {
        // SAFETY: buffer is padded with 8 bytes
        unsafe {
            let b0 = *self.buf.get_unchecked(self.pos) as u32;
            let b1 = *self.buf.get_unchecked(self.pos + 1) as u32;
            let b2 = *self.buf.get_unchecked(self.pos + 2) as u32;
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
    }

    /// Get value in high 32 bits for extended reading (matching unrar getbits32).
    /// Returns 32 bits with the first available bit at bit 31.
    #[inline(always)]
    pub fn get_value_high32(&self) -> u32 {
        // SAFETY: buffer is padded with 8 bytes
        unsafe {
            let b0 = *self.buf.get_unchecked(self.pos) as u32;
            let b1 = *self.buf.get_unchecked(self.pos + 1) as u32;
            let b2 = *self.buf.get_unchecked(self.pos + 2) as u32;
            let b3 = *self.buf.get_unchecked(self.pos + 3) as u32;
            let b4 = *self.buf.get_unchecked(self.pos + 4) as u32;
        
            // Build 32-bit value like unrar: RawGetBE4
            let mut v = (b0 << 24) | (b1 << 16) | (b2 << 8) | b3;
            // Left shift by bit_pos to align first available bit to bit 31
            v <<= self.bit_pos;
            // Include bits from 5th byte if needed
            if self.bit_pos > 0 {
                v |= b4 >> (8 - self.bit_pos);
            }
            v
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
    }

    /// Check if reading has exceeded block boundary.
    #[inline(always)]
    pub fn is_block_over_read(&self) -> bool {
        self.pos > self.block_end || (self.pos == self.block_end && self.bit_pos >= self.block_end_bits)
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
        self.pos = pos;
        self.bit_pos = 0;
    }

    /// Check if EOF reached.
    pub fn is_eof(&self) -> bool {
        self.pos >= self.block_end
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
