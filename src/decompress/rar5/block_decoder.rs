//! RAR5 LZSS decoder with Huffman tables.
//!
//! RAR5 uses a block-based format where each block contains:
//! 1. Block header with flags
//! 2. Huffman tables (if not using previous tables)
//! 3. Compressed data stream

use super::range_coder::RangeCoder;
use crate::decompress::DecompressError;

/// Maximum number of symbols in main code table
const MAIN_TABLE_SIZE: usize = 306;
/// Maximum number of symbols in offset table
const OFFSET_TABLE_SIZE: usize = 64;
/// Maximum number of symbols in small offset table  
const SMALL_OFFSET_TABLE_SIZE: usize = 16;
/// Maximum number of symbols in length table
const LENGTH_TABLE_SIZE: usize = 44;

/// Huffman decode table entry
#[derive(Clone, Copy, Default)]
struct HuffEntry {
    /// Symbol value
    symbol: u16,
    /// Code length in bits
    length: u8,
}

/// Huffman decode table
struct HuffTable {
    /// Entries indexed by code
    entries: Vec<HuffEntry>,
    /// Quick lookup table for short codes
    quick_table: Vec<u16>,
    /// Maximum code length
    max_length: u8,
}

impl HuffTable {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            quick_table: vec![0; 1024],
            max_length: 0,
        }
    }

    /// Build table from code lengths.
    fn build(&mut self, lengths: &[u8], max_symbol: usize) -> Result<(), DecompressError> {
        self.entries.clear();
        self.entries.resize(max_symbol, HuffEntry::default());

        // Count codes of each length
        let mut count = [0u32; 16];
        for &len in lengths.iter().take(max_symbol) {
            if len > 0 && (len as usize) < count.len() {
                count[len as usize] += 1;
            }
        }

        // Calculate starting codes for each length
        let mut next_code = [0u32; 16];
        let mut code = 0u32;
        for bits in 1..16 {
            code = (code + count[bits - 1]) << 1;
            next_code[bits] = code;
        }

        // Assign codes to symbols
        for (symbol, &len) in lengths.iter().enumerate().take(max_symbol) {
            if len > 0 {
                let len_idx = len as usize;
                self.entries[symbol] = HuffEntry {
                    symbol: symbol as u16,
                    length: len,
                };
                next_code[len_idx] += 1;
                self.max_length = self.max_length.max(len);
            }
        }

        Ok(())
    }
}

/// RAR5 block decoder state
pub struct Rar5BlockDecoder {
    /// Main symbol table (literals + lengths)
    main_table: HuffTable,
    /// Offset table
    offset_table: HuffTable,
    /// Small offset table (for recent offsets)
    small_offset_table: HuffTable,
    /// Length table
    length_table: HuffTable,
    /// Recent offsets for back-references
    recent_offsets: [u64; 4],
    /// Last used length
    last_length: u32,
    /// Dictionary/window buffer
    window: Vec<u8>,
    /// Window mask (size - 1)
    window_mask: usize,
    /// Current position in window
    window_pos: usize,
    /// Dictionary size
    dict_size: usize,
}

impl Rar5BlockDecoder {
    /// Create a new block decoder with given dictionary size.
    pub fn new(dict_size_log: u8) -> Self {
        let dict_size = 1usize << dict_size_log;
        Self {
            main_table: HuffTable::new(),
            offset_table: HuffTable::new(),
            small_offset_table: HuffTable::new(),
            length_table: HuffTable::new(),
            recent_offsets: [0; 4],
            last_length: 0,
            window: vec![0u8; dict_size],
            window_mask: dict_size - 1,
            window_pos: 0,
            dict_size,
        }
    }

    /// Reset decoder state (keep tables).
    pub fn reset(&mut self) {
        self.recent_offsets = [0; 4];
        self.last_length = 0;
        self.window_pos = 0;
    }

    /// Write a byte to the window.
    #[inline]
    fn write_byte(&mut self, byte: u8) {
        self.window[self.window_pos & self.window_mask] = byte;
        self.window_pos += 1;
    }

    /// Copy bytes from earlier position in window.
    #[inline]
    fn copy_bytes(&mut self, offset: usize, length: usize) {
        let src_start = self.window_pos.wrapping_sub(offset);
        for i in 0..length {
            let src_pos = (src_start + i) & self.window_mask;
            let byte = self.window[src_pos];
            self.write_byte(byte);
        }
    }

    /// Get output from window.
    pub fn get_output(&self, start: usize, length: usize) -> Vec<u8> {
        let mut output = Vec::with_capacity(length);
        for i in 0..length {
            let pos = (start + i) & self.window_mask;
            output.push(self.window[pos]);
        }
        output
    }

    /// Decode a block of compressed data.
    pub fn decode_block(
        &mut self,
        coder: &mut RangeCoder,
        output_size: usize,
    ) -> Result<(), DecompressError> {
        let start_pos = self.window_pos;

        while self.window_pos - start_pos < output_size {
            // Decode main symbol
            // For now, simplified - just decode literals
            // Full implementation would decode from Huffman tables

            if coder.is_eof() {
                break;
            }

            // Placeholder: Read raw bytes (for testing)
            // Real implementation decodes from Huffman + range coder
            let byte = coder.decode_bits(8) as u8;
            self.write_byte(byte);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_decoder_new() {
        let decoder = Rar5BlockDecoder::new(20); // 1MB dictionary
        assert_eq!(decoder.dict_size, 1 << 20);
        assert_eq!(decoder.window.len(), 1 << 20);
    }

    #[test]
    fn test_write_and_copy() {
        let mut decoder = Rar5BlockDecoder::new(10); // 1KB dictionary
        
        // Write some bytes
        decoder.write_byte(b'H');
        decoder.write_byte(b'e');
        decoder.write_byte(b'l');
        decoder.write_byte(b'l');
        decoder.write_byte(b'o');

        assert_eq!(decoder.window_pos, 5);
        
        // Copy from offset 5, length 5 (repeat "Hello")
        decoder.copy_bytes(5, 5);
        assert_eq!(decoder.window_pos, 10);
        
        let output = decoder.get_output(0, 10);
        assert_eq!(&output, b"HelloHello");
    }
}
