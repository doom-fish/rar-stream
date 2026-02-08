//! RAR5 Range Coder implementation (WIP).
//!
//! Range coding is an entropy coding method similar to arithmetic coding.
//! This module provides the infrastructure for future RAR5 PPMd support.
//! Currently unused — RAR5 PPMd blocks are rare in practice.

// WIP: range coder for RAR5 PPMd
/// Range coder state for decoding.
#[allow(dead_code)]
pub struct RangeCoder {
    /// Low bound of current range
    low: u64,
    /// Current range size
    range: u64,
    /// Cached input code value
    code: u64,
    /// Input data
    input: Vec<u8>,
    /// Current position in input
    pos: usize,
}

#[allow(dead_code)]
impl RangeCoder {
    /// Top value for range normalization (2^24)
    const TOP: u64 = 1 << 24;
    /// Bottom value for range (2^15)
    const BOT: u64 = 1 << 15;

    /// Create a new range coder from input data.
    pub fn new(input: &[u8]) -> Self {
        let mut coder = Self {
            low: 0,
            range: 0xFFFF_FFFF,
            code: 0,
            input: input.to_vec(),
            pos: 0,
        };
        // Initialize code from first 4 bytes
        for _ in 0..4 {
            coder.code = (coder.code << 8) | u64::from(coder.get_byte());
        }
        coder
    }

    /// Get next byte from input.
    #[inline]
    fn get_byte(&mut self) -> u8 {
        if self.pos < self.input.len() {
            let byte = self.input[self.pos];
            self.pos += 1;
            byte
        } else {
            0
        }
    }

    /// Normalize the range if needed.
    #[inline]
    fn normalize(&mut self) {
        while self.range < Self::TOP {
            self.code = (self.code << 8) | u64::from(self.get_byte());
            self.range <<= 8;
            self.low <<= 8;
        }
    }

    /// Decode a bit with given probability (0-4096 scale).
    /// Returns true (1) or false (0).
    #[inline]
    pub fn decode_bit(&mut self, prob: u32) -> bool {
        let bound = (self.range >> 12) * u64::from(prob);
        if self.code < bound {
            self.range = bound;
            self.normalize();
            false
        } else {
            self.code -= bound;
            self.range -= bound;
            self.normalize();
            true
        }
    }

    /// Decode a symbol using frequency table.
    /// `freqs` contains cumulative frequencies.
    #[inline]
    pub fn decode_symbol(&mut self, freqs: &[u32], total: u32) -> usize {
        let scale = self.range / u64::from(total);
        let cum = ((self.code - self.low) / scale) as u32;

        // Find symbol by binary search
        let mut lo = 0;
        let mut hi = freqs.len();
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if freqs[mid] <= cum {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }

        let symbol = if lo > 0 { lo - 1 } else { 0 };

        // Update range
        let low_freq = if symbol > 0 { freqs[symbol - 1] } else { 0 };
        let high_freq = freqs[symbol];

        self.low += scale * u64::from(low_freq);
        self.range = scale * u64::from(high_freq - low_freq);
        self.normalize();

        symbol
    }

    /// Decode raw bits without probability model.
    #[inline]
    pub fn decode_bits(&mut self, bits: u32) -> u32 {
        let mut result = 0u32;
        for _ in 0..bits {
            self.range >>= 1;
            if self.code >= self.range {
                self.code -= self.range;
                result = (result << 1) | 1;
            } else {
                result <<= 1;
            }
            self.normalize();
        }
        result
    }

    /// Check if we've reached the end of input.
    pub fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_coder_init() {
        let data = [0x12, 0x34, 0x56, 0x78, 0x9A];
        let coder = RangeCoder::new(&data);
        assert_eq!(coder.code, 0x12345678);
    }

    #[test]
    fn test_decode_bits() {
        let data = [0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00];
        let mut coder = RangeCoder::new(&data);
        // This is a basic test - actual values depend on range state
        let _ = coder.decode_bits(8);
    }
}
