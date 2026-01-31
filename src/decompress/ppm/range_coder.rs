//! Carryless range coder for PPMd.
//!
//! Based on Dmitry Subbotin's implementation.

use super::super::BitReader;

/// Range coder constants.
const TOP: u32 = 1 << 24;
const BOT: u32 = 1 << 15;

/// Range coder state.
pub struct RangeCoder {
    low: u32,
    code: u32,
    range: u32,
}

/// Subrange for decoding.
pub struct SubRange {
    pub low_count: u32,
    pub high_count: u32,
    pub scale: u32,
}

impl RangeCoder {
    /// Initialize the range coder from the bitstream.
    pub fn new(reader: &mut BitReader) -> Self {
        let mut code = 0u32;
        for _ in 0..4 {
            code = (code << 8) | reader.read_byte().unwrap_or(0) as u32;
        }
        Self {
            low: 0,
            code,
            range: 0xFFFFFFFF,
        }
    }

    /// Get current count within the scale.
    #[inline]
    pub fn get_current_count(&mut self, scale: u32) -> u32 {
        self.range /= scale;
        // Use wrapping_sub to avoid overflow - this can happen with state corruption

        self.code.wrapping_sub(self.low) / self.range
    }

    /// Debug method to get internal state
    #[cfg(test)]
    pub fn debug_state(&self) -> (u32, u32, u32) {
        (self.code, self.low, self.range)
    }

    /// Get current count with shift.
    #[inline]
    pub fn get_current_shift_count(&mut self, shift: u32) -> u32 {
        self.range >>= shift;
        if self.range == 0 {
            return 0; // Avoid division by zero
        }
        // Use wrapping_sub to avoid overflow - this can happen with state corruption
        self.code.wrapping_sub(self.low) / self.range
    }

    /// Decode with the given subrange (without normalizing).
    #[inline]
    pub fn decode(&mut self, sub: &SubRange) {
        self.low = self
            .low
            .wrapping_add(sub.low_count.wrapping_mul(self.range));
        self.range = self.range.wrapping_mul(sub.high_count - sub.low_count);
    }

    /// Normalize the range coder state.
    #[inline]
    pub fn normalize(&mut self, reader: &mut BitReader) {
        while (self.low ^ (self.low.wrapping_add(self.range))) < TOP
            || self.range < BOT && {
                self.range = (0u32.wrapping_sub(self.low)) & (BOT - 1);
                true
            }
        {
            let byte = reader.read_byte().unwrap_or(0);
            self.code = (self.code << 8) | byte as u32;
            self.range <<= 8;
            self.low <<= 8;
        }
    }
}
