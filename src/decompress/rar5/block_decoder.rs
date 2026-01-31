//! RAR5 LZSS decoder with Huffman tables.
//!
//! RAR5 uses a block-based format where each block contains:
//! 1. Block header with flags
//! 2. Huffman tables (if not using previous tables)
//! 3. Compressed data stream

use super::range_coder::RangeCoder;
use crate::decompress::DecompressError;

/// Maximum number of symbols in main code table (256 literals + 50 length codes)
const MAIN_TABLE_SIZE: usize = 306;
/// Maximum number of symbols in offset table
const OFFSET_TABLE_SIZE: usize = 64;
/// Maximum number of symbols in low offset table
const LOW_OFFSET_TABLE_SIZE: usize = 16;
/// Maximum number of symbols in length table
const LENGTH_TABLE_SIZE: usize = 44;
/// Maximum Huffman code length
const MAX_CODE_LENGTH: usize = 15;
/// Quick table bits for fast lookup
const QUICK_BITS: usize = 10;

/// Huffman decode table
#[derive(Clone)]
pub struct HuffTable {
    /// Code lengths for each symbol
    code_lengths: Vec<u8>,
    /// Quick lookup table: (symbol, length) packed
    quick_table: Vec<u32>,
    /// Decode table for longer codes
    decode_table: Vec<u16>,
    /// Number of symbols
    num_symbols: usize,
    /// Maximum code length in table
    max_length: u8,
}

impl HuffTable {
    /// Create a new empty Huffman table.
    pub fn new(max_symbols: usize) -> Self {
        Self {
            code_lengths: vec![0; max_symbols],
            quick_table: vec![0; 1 << QUICK_BITS],
            decode_table: Vec::new(),
            num_symbols: max_symbols,
            max_length: 0,
        }
    }

    /// Build table from code lengths.
    pub fn build(&mut self, lengths: &[u8]) -> Result<(), DecompressError> {
        let num_symbols = lengths.len().min(self.num_symbols);
        self.code_lengths[..num_symbols].copy_from_slice(&lengths[..num_symbols]);
        
        // Find max length
        self.max_length = 0;
        for &len in &self.code_lengths[..num_symbols] {
            if len > self.max_length {
                self.max_length = len;
            }
        }

        if self.max_length == 0 {
            return Ok(()); // Empty table
        }

        // Count codes of each length
        let mut count = [0u32; MAX_CODE_LENGTH + 1];
        for &len in &self.code_lengths[..num_symbols] {
            if len > 0 {
                count[len as usize] += 1;
            }
        }

        // Calculate starting codes for each length
        let mut next_code = [0u32; MAX_CODE_LENGTH + 1];
        let mut code = 0u32;
        for bits in 1..=MAX_CODE_LENGTH {
            code = (code + count[bits - 1]) << 1;
            next_code[bits] = code;
        }

        // Build quick table for codes <= QUICK_BITS
        self.quick_table.fill(0);
        
        for (symbol, &len) in self.code_lengths[..num_symbols].iter().enumerate() {
            if len > 0 && len as usize <= QUICK_BITS {
                let code = next_code[len as usize];
                next_code[len as usize] += 1;
                
                // Fill all entries that start with this code
                let fill_bits = QUICK_BITS - len as usize;
                let base_idx = (code as usize) << fill_bits;
                let fill_count = 1 << fill_bits;
                
                let entry = ((symbol as u32) << 8) | (len as u32);
                for i in 0..fill_count {
                    if base_idx + i < self.quick_table.len() {
                        self.quick_table[base_idx + i] = entry;
                    }
                }
            }
        }

        Ok(())
    }

    /// Decode a symbol using the range coder.
    #[inline]
    pub fn decode(&self, coder: &mut RangeCoder) -> u16 {
        // Peek bits for quick table lookup
        let bits = coder.decode_bits(QUICK_BITS as u32);
        let entry = self.quick_table[bits as usize];
        
        if entry != 0 {
            let len = (entry & 0xFF) as u32;
            // Put back unused bits (we took QUICK_BITS but only needed 'len')
            // Note: This is simplified - real impl needs bit buffer management
            let symbol = (entry >> 8) as u16;
            return symbol;
        }

        // Slow path for longer codes or empty entries
        0
    }
}

/// Block types in RAR5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    /// LZ block with Huffman tables
    Lz,
    /// LZ block reusing previous tables
    LzContinue,
    /// PPM block (not implemented)
    Ppm,
}

/// RAR5 block decoder state
pub struct Rar5BlockDecoder {
    /// Main symbol table (literals + lengths)
    main_table: HuffTable,
    /// Offset table
    offset_table: HuffTable,
    /// Low offset table (for nearby matches)
    low_offset_table: HuffTable,
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
    /// Tables have been initialized
    tables_valid: bool,
}

impl Rar5BlockDecoder {
    /// Create a new block decoder with given dictionary size.
    pub fn new(dict_size_log: u8) -> Self {
        let dict_size = 1usize << dict_size_log;
        Self {
            main_table: HuffTable::new(MAIN_TABLE_SIZE),
            offset_table: HuffTable::new(OFFSET_TABLE_SIZE),
            low_offset_table: HuffTable::new(LOW_OFFSET_TABLE_SIZE),
            length_table: HuffTable::new(LENGTH_TABLE_SIZE),
            recent_offsets: [0; 4],
            last_length: 0,
            window: vec![0u8; dict_size],
            window_mask: dict_size - 1,
            window_pos: 0,
            dict_size,
            tables_valid: false,
        }
    }

    /// Reset decoder state (keep tables).
    pub fn reset(&mut self) {
        self.recent_offsets = [0; 4];
        self.last_length = 0;
        self.window_pos = 0;
        self.tables_valid = false;
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

    /// Read and build Huffman tables from compressed stream.
    fn read_tables(&mut self, coder: &mut RangeCoder) -> Result<(), DecompressError> {
        // Read code lengths using a pre-code
        // First, read the pre-code (code lengths for the code length alphabet)
        let mut precode_lengths = [0u8; 20];
        
        // Pre-code is transmitted as 4-bit values for 20 symbols
        for len in &mut precode_lengths {
            *len = coder.decode_bits(4) as u8;
        }

        // Build pre-code table
        let mut precode = HuffTable::new(20);
        precode.build(&precode_lengths)?;

        // Now decode the actual code lengths for main table
        let mut main_lengths = vec![0u8; MAIN_TABLE_SIZE];
        self.decode_code_lengths(coder, &precode, &mut main_lengths)?;
        self.main_table.build(&main_lengths)?;

        // Decode offset table lengths
        let mut offset_lengths = vec![0u8; OFFSET_TABLE_SIZE];
        self.decode_code_lengths(coder, &precode, &mut offset_lengths)?;
        self.offset_table.build(&offset_lengths)?;

        // Decode low offset table lengths
        let mut low_offset_lengths = vec![0u8; LOW_OFFSET_TABLE_SIZE];
        self.decode_code_lengths(coder, &precode, &mut low_offset_lengths)?;
        self.low_offset_table.build(&low_offset_lengths)?;

        // Decode length table lengths
        let mut length_lengths = vec![0u8; LENGTH_TABLE_SIZE];
        self.decode_code_lengths(coder, &precode, &mut length_lengths)?;
        self.length_table.build(&length_lengths)?;

        self.tables_valid = true;
        Ok(())
    }

    /// Decode code lengths using pre-code.
    fn decode_code_lengths(
        &self,
        coder: &mut RangeCoder,
        precode: &HuffTable,
        lengths: &mut [u8],
    ) -> Result<(), DecompressError> {
        let mut i = 0;
        while i < lengths.len() {
            let sym = precode.decode(coder);
            
            match sym {
                0..=15 => {
                    // Direct code length
                    lengths[i] = sym as u8;
                    i += 1;
                }
                16 => {
                    // Repeat previous length 3-6 times
                    let count = 3 + coder.decode_bits(2) as usize;
                    let prev = if i > 0 { lengths[i - 1] } else { 0 };
                    for _ in 0..count {
                        if i < lengths.len() {
                            lengths[i] = prev;
                            i += 1;
                        }
                    }
                }
                17 => {
                    // Repeat zero 3-10 times
                    let count = 3 + coder.decode_bits(3) as usize;
                    for _ in 0..count {
                        if i < lengths.len() {
                            lengths[i] = 0;
                            i += 1;
                        }
                    }
                }
                18 => {
                    // Repeat zero 11-138 times
                    let count = 11 + coder.decode_bits(7) as usize;
                    for _ in 0..count {
                        if i < lengths.len() {
                            lengths[i] = 0;
                            i += 1;
                        }
                    }
                }
                _ => {
                    // Invalid symbol
                    return Err(DecompressError::InvalidHuffmanCode);
                }
            }
        }
        Ok(())
    }

    /// Decode a block of compressed data.
    pub fn decode_block(
        &mut self,
        coder: &mut RangeCoder,
        output_size: usize,
    ) -> Result<(), DecompressError> {
        let start_pos = self.window_pos;

        // Read block header (simplified)
        let block_type = coder.decode_bits(1);
        
        if block_type == 0 {
            // New tables
            self.read_tables(coder)?;
        } else if !self.tables_valid {
            // Continue block but no previous tables
            return Err(DecompressError::InvalidHuffmanCode);
        }

        // Decode symbols until we have enough output
        while self.window_pos - start_pos < output_size {
            if coder.is_eof() {
                break;
            }

            let sym = self.main_table.decode(coder);

            if sym < 256 {
                // Literal byte
                self.write_byte(sym as u8);
            } else if sym < 262 {
                // Use recent offset (sym - 256 selects which one)
                let offset_idx = (sym - 256) as usize;
                let offset = self.recent_offsets[offset_idx.min(3)] as usize;
                let length = self.decode_length(coder)?;
                
                // Update recent offsets
                if offset_idx > 0 {
                    let off = self.recent_offsets[offset_idx];
                    for j in (1..=offset_idx).rev() {
                        self.recent_offsets[j] = self.recent_offsets[j - 1];
                    }
                    self.recent_offsets[0] = off;
                }

                self.copy_bytes(offset, length);
            } else {
                // New offset + length
                let length_sym = sym - 262;
                let length = self.slot_to_length(length_sym as u32, coder)?;
                let offset = self.decode_offset(coder)?;

                // Update recent offsets
                for j in (1..4).rev() {
                    self.recent_offsets[j] = self.recent_offsets[j - 1];
                }
                self.recent_offsets[0] = offset as u64;

                self.copy_bytes(offset, length);
            }
        }

        Ok(())
    }

    /// Decode a match length.
    fn decode_length(&mut self, coder: &mut RangeCoder) -> Result<usize, DecompressError> {
        let sym = self.length_table.decode(coder);
        self.slot_to_length(sym as u32, coder)
    }

    /// Convert length slot to actual length.
    fn slot_to_length(&self, slot: u32, coder: &mut RangeCoder) -> Result<usize, DecompressError> {
        // Slot 0-7: lengths 2-9
        // Higher slots have extra bits
        if slot < 8 {
            Ok((slot + 2) as usize)
        } else {
            let extra_bits = (slot - 4) / 4;
            let base = ((4 + (slot & 3)) << extra_bits) + 2;
            let extra = coder.decode_bits(extra_bits);
            Ok((base + extra) as usize)
        }
    }

    /// Decode a match offset.
    fn decode_offset(&mut self, coder: &mut RangeCoder) -> Result<usize, DecompressError> {
        let slot = self.offset_table.decode(coder) as u32;
        
        if slot < 4 {
            // Small offsets 0-3
            let low = self.low_offset_table.decode(coder) as u32;
            Ok(((slot << 4) | low) as usize + 1)
        } else {
            // Larger offsets with extra bits
            let extra_bits = (slot / 2) - 1;
            let base = (2 + (slot & 1)) << extra_bits;
            let extra = coder.decode_bits(extra_bits);
            let low = self.low_offset_table.decode(coder) as u32;
            Ok((((base + extra) << 4) | low) as usize + 1)
        }
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

    #[test]
    fn test_huff_table_build() {
        let mut table = HuffTable::new(8);
        // Simple code lengths: symbols 0-7 with lengths 3,3,3,3,3,3,3,3
        let lengths = [3u8, 3, 3, 3, 3, 3, 3, 3];
        table.build(&lengths).unwrap();
        assert_eq!(table.max_length, 3);
    }
}

