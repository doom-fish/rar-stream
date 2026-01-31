//! RAR5 Bit-level stream decoder.
//!
//! RAR5 uses a bit decoder (not range coder) for reading Huffman tables
//! and decompressing data. This is aligned byte reading with bit-level access.

/// Bit-level stream decoder for RAR5 decompression.
pub struct BitDecoder {
    /// Input buffer
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
    pub fn new(input: &[u8]) -> Self {
        Self {
            buf: input.to_vec(),
            pos: 0,
            bit_pos: 0,
            block_end: input.len(),
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
        if self.pos < self.buf.len() {
            let b = self.buf[self.pos];
            self.pos += 1;
            b
        } else {
            0xFF
        }
    }

    /// Read up to 9 bits with fixed mask.
    #[inline]
    pub fn read_bits_9fix(&mut self, num_bits: usize) -> u32 {
        if num_bits == 0 {
            return 0;
        }
        let mask = (1u32 << num_bits) - 1;
        let b0 = self.buf.get(self.pos).copied().unwrap_or(0xFF) as u32;
        let b1 = self.buf.get(self.pos + 1).copied().unwrap_or(0xFF) as u32;
        let v = (b0 << 8) | b1;
        let total_bits = num_bits + self.bit_pos;
        let result = (v >> (16 - total_bits)) & mask;
        self.pos += total_bits >> 3;
        self.bit_pos = total_bits & 7;
        result
    }

    /// Read up to 9 bits.
    #[inline]
    pub fn read_bits_9(&mut self, num_bits: usize) -> u32 {
        if num_bits == 0 {
            return 0;
        }
        let b0 = self.buf.get(self.pos).copied().unwrap_or(0xFF) as u32;
        let b1 = self.buf.get(self.pos + 1).copied().unwrap_or(0xFF) as u32;
        let v = (b0 << 8) | b1;
        let v = v & (0xFFFF >> self.bit_pos);
        let total_bits = num_bits + self.bit_pos;
        let result = v >> (16 - total_bits);
        self.pos += total_bits >> 3;
        self.bit_pos = total_bits & 7;
        result
    }

    /// Get up to 15 bits for Huffman decoding (without advancing).
    #[inline]
    pub fn get_value_15(&self) -> u32 {
        let b0 = self.buf.get(self.pos).copied().unwrap_or(0xFF) as u32;
        let b1 = self.buf.get(self.pos + 1).copied().unwrap_or(0xFF) as u32;
        let b2 = self.buf.get(self.pos + 2).copied().unwrap_or(0xFF) as u32;
        let v = (b0 << 16) | (b1 << 8) | b2;
        (v >> (9 - self.bit_pos)) & 0x7FFF
    }

    /// Move position by num_bits.
    #[inline]
    pub fn move_pos(&mut self, num_bits: usize) {
        let total = num_bits + self.bit_pos;
        self.pos += total >> 3;
        self.bit_pos = total & 7;
    }

    /// Get value in high 32 bits for extended reading.
    #[inline]
    pub fn get_value_high32(&self) -> u32 {
        let b0 = self.buf.get(self.pos).copied().unwrap_or(0xFF) as u32;
        let b1 = self.buf.get(self.pos + 1).copied().unwrap_or(0xFF) as u32;
        let b2 = self.buf.get(self.pos + 2).copied().unwrap_or(0xFF) as u32;
        let v = (b0 << 16) | (b1 << 8) | b2;
        v << (8 + self.bit_pos)
    }

    /// Read extended bits from pre-read high value.
    #[inline]
    pub fn read_bits_big(&mut self, num_bits: usize, mut v: u32) -> u32 {
        if num_bits == 0 {
            return 0;
        }
        // Include 4th byte for extended reading when needed
        if self.bit_pos > 0 {
            if let Some(&b3) = self.buf.get(self.pos + 3) {
                v |= (b3 as u32) << self.bit_pos;
            }
        }
        let result = v >> (32 - num_bits);
        self.move_pos(num_bits);
        result
    }

    /// Set block end position.
    pub fn set_block_end(&mut self, end: usize, end_bits: usize) {
        self.block_end = end;
        self.block_end_bits = end_bits;
    }

    /// Check if reading has exceeded block boundary.
    pub fn is_block_over_read(&self) -> bool {
        if self.pos < self.block_end {
            false
        } else if self.pos > self.block_end {
            true
        } else {
            self.bit_pos > self.block_end_bits
        }
    }

    /// Current byte position.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Check if EOF reached.
    pub fn is_eof(&self) -> bool {
        self.pos >= self.buf.len()
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
