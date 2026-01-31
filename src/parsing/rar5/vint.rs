//! Variable-length integer (vint) parsing for RAR5.
//!
//! RAR5 uses variable-length integers where each byte contributes 7 bits
//! of data, and the high bit indicates if more bytes follow.
//!
//! Format:
//! - Bits 0-6: Data bits
//! - Bit 7: Continuation flag (1 = more bytes follow)

/// Read a variable-length integer from a byte slice.
/// Returns the value and the number of bytes consumed.
#[inline]
pub fn read_vint(data: &[u8]) -> Option<(u64, usize)> {
    let mut result = 0u64;
    let mut shift = 0;

    for (i, &byte) in data.iter().enumerate() {
        // Prevent overflow - vint can be at most 10 bytes for u64
        if i >= 10 {
            return None;
        }

        result |= u64::from(byte & 0x7F) << shift;

        if byte & 0x80 == 0 {
            return Some((result, i + 1));
        }

        shift += 7;
    }

    // Ran out of bytes without finding end
    None
}

/// Helper for reading multiple vints from a buffer.
pub struct VintReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> VintReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    /// Read the next vint from the buffer.
    #[inline]
    pub fn read(&mut self) -> Option<u64> {
        let (value, consumed) = read_vint(&self.data[self.offset..])?;
        self.offset += consumed;
        Some(value)
    }

    /// Read a fixed number of bytes.
    #[inline]
    pub fn read_bytes(&mut self, count: usize) -> Option<&'a [u8]> {
        if self.offset + count > self.data.len() {
            return None;
        }
        let slice = &self.data[self.offset..self.offset + count];
        self.offset += count;
        Some(slice)
    }

    /// Read a u32 in little-endian format.
    #[inline]
    pub fn read_u32_le(&mut self) -> Option<u32> {
        let bytes = self.read_bytes(4)?;
        Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read a u64 in little-endian format.
    #[inline]
    pub fn read_u64_le(&mut self) -> Option<u64> {
        let bytes = self.read_bytes(8)?;
        Some(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    /// Current position in the buffer.
    pub fn position(&self) -> usize {
        self.offset
    }

    /// Remaining bytes in the buffer.
    pub fn remaining(&self) -> &'a [u8] {
        &self.data[self.offset..]
    }

    /// Skip ahead by a number of bytes.
    pub fn skip(&mut self, count: usize) -> bool {
        if self.offset + count > self.data.len() {
            return false;
        }
        self.offset += count;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_byte_vint() {
        // Values 0-127 fit in one byte
        assert_eq!(read_vint(&[0x00]), Some((0, 1)));
        assert_eq!(read_vint(&[0x7F]), Some((127, 1)));
        assert_eq!(read_vint(&[0x01]), Some((1, 1)));
    }

    #[test]
    fn test_two_byte_vint() {
        // 128 = 0x80 in first byte (continuation) + 0x01 in second
        assert_eq!(read_vint(&[0x80, 0x01]), Some((128, 2)));
        // 255 = 0xFF & 0x7F = 127, then 0x01 << 7 = 128
        assert_eq!(read_vint(&[0xFF, 0x01]), Some((255, 2)));
    }

    #[test]
    fn test_larger_vint() {
        // 16384 = 0x80, 0x80, 0x01
        assert_eq!(read_vint(&[0x80, 0x80, 0x01]), Some((16384, 3)));
    }

    #[test]
    fn test_vint_reader() {
        let data = [0x05, 0x80, 0x01, 0x7F];
        let mut reader = VintReader::new(&data);

        assert_eq!(reader.read(), Some(5));
        assert_eq!(reader.read(), Some(128));
        assert_eq!(reader.read(), Some(127));
        assert_eq!(reader.read(), None);
    }

    #[test]
    fn test_empty_buffer() {
        assert_eq!(read_vint(&[]), None);
    }

    #[test]
    fn test_incomplete_vint() {
        // Continuation bit set but no more bytes
        assert_eq!(read_vint(&[0x80]), None);
    }
}
