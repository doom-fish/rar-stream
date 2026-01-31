//! Bit reader for compressed data streams.
//!
//! Reads bits from a byte stream, LSB first (RAR convention).

use super::{DecompressError, Result};

/// Bit reader that reads from a byte slice.
pub struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    bit_pos: u32,
    /// Current bit buffer (up to 32 bits)
    buffer: u32,
    /// Bits available in buffer
    bits_in_buffer: u32,
}

impl<'a> BitReader<'a> {
    /// Create a new bit reader from a byte slice.
    pub fn new(data: &'a [u8]) -> Self {
        let mut reader = Self {
            data,
            pos: 0,
            bit_pos: 0,
            buffer: 0,
            bits_in_buffer: 0,
        };
        reader.fill_buffer();
        reader
    }

    /// Fill the buffer with more bytes.
    fn fill_buffer(&mut self) {
        while self.bits_in_buffer <= 24 && self.pos < self.data.len() {
            self.buffer |= (self.data[self.pos] as u32) << (24 - self.bits_in_buffer);
            self.bits_in_buffer += 8;
            self.pos += 1;
        }
    }

    /// Peek at the next n bits without consuming them.
    #[inline]
    pub fn peek_bits(&self, n: u32) -> u32 {
        debug_assert!(n <= 16);
        self.buffer >> (32 - n)
    }

    /// Read n bits and advance the position.
    #[inline]
    pub fn read_bits(&mut self, n: u32) -> Result<u32> {
        debug_assert!(n <= 16);

        if n > self.bits_in_buffer && self.pos >= self.data.len() {
            return Err(DecompressError::UnexpectedEof);
        }

        let value = self.peek_bits(n);
        self.advance_bits(n);
        Ok(value)
    }

    /// Advance by n bits.
    #[inline]
    pub fn advance_bits(&mut self, n: u32) {
        self.buffer <<= n;
        self.bits_in_buffer = self.bits_in_buffer.saturating_sub(n);
        self.bit_pos += n;
        self.fill_buffer();
    }

    /// Read a single bit.
    #[inline]
    pub fn read_bit(&mut self) -> Result<bool> {
        Ok(self.read_bits(1)? != 0)
    }

    /// Read a single byte (8 bits).
    #[inline]
    pub fn read_byte(&mut self) -> Option<u8> {
        self.read_bits(8).ok().map(|v| v as u8)
    }

    /// Align to byte boundary by skipping remaining bits in current byte.
    #[inline]
    pub fn align_to_byte(&mut self) {
        let bits_used_in_byte = self.bit_pos % 8;
        if bits_used_in_byte > 0 {
            let skip = 8 - bits_used_in_byte;
            self.advance_bits(skip);
        }
    }

    /// Get the current bit position.
    pub fn bit_position(&self) -> u64 {
        self.bit_pos as u64
    }

    /// Get the current byte position (bytes consumed from stream).
    pub fn byte_position(&self) -> usize {
        self.pos
    }

    /// Check if at end of data.
    pub fn is_eof(&self) -> bool {
        self.bits_in_buffer == 0 && self.pos >= self.data.len()
    }

    /// Remaining bits available.
    pub fn remaining_bits(&self) -> u64 {
        self.bits_in_buffer as u64 + ((self.data.len() - self.pos) as u64 * 8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_bits() {
        let data = [0b10110100, 0b11001010];
        let mut reader = BitReader::new(&data);

        assert_eq!(reader.read_bits(4).unwrap(), 0b1011);
        assert_eq!(reader.read_bits(4).unwrap(), 0b0100);
        assert_eq!(reader.read_bits(8).unwrap(), 0b11001010);
    }

    #[test]
    fn test_peek_bits() {
        let data = [0b10110100];
        let reader = BitReader::new(&data);

        assert_eq!(reader.peek_bits(4), 0b1011);
        assert_eq!(reader.peek_bits(8), 0b10110100);
    }

    #[test]
    fn test_eof() {
        let data = [0xFF];
        let mut reader = BitReader::new(&data);

        assert!(!reader.is_eof());
        reader.read_bits(8).unwrap();
        assert!(reader.is_eof());
    }
}
