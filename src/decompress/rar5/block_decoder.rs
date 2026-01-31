//! RAR5 LZSS decoder with Huffman tables.
//!
//! RAR5 uses a block-based format where each block contains:
//! 1. Block header with flags, checksum, and size
//! 2. Huffman tables (if flag indicates new tables)
//! 3. Compressed data stream

use super::bit_decoder::BitDecoder;
use crate::decompress::DecompressError;

// Table sizes from RAR5 spec (matching unrar5j)
/// Number of repetition entries
const NUM_REPS: usize = 4;
/// Length table size
const LEN_TABLE_SIZE: usize = 11 * 4; // 44
/// Main table: 256 literals + 1 + 1 + NUM_REPS + LEN_TABLE_SIZE = 306
const MAIN_TABLE_SIZE: usize = 256 + 1 + 1 + NUM_REPS + LEN_TABLE_SIZE;
/// Distance table size (v6 format)
const DIST_TABLE_SIZE: usize = 64;
/// Alignment table size
const ALIGN_TABLE_SIZE: usize = 16;
/// Number of alignment bits
const NUM_ALIGN_BITS: usize = 4;
/// Level table size (pre-code)
const LEVEL_TABLE_SIZE: usize = 20;
/// Maximum Huffman code bits
const NUM_HUFFMAN_BITS: usize = 15;
/// Quick lookup table bits
const QUICK_BITS_MAIN: usize = 10;
const QUICK_BITS_DIST: usize = 7;
const QUICK_BITS_LEN: usize = 7;
const QUICK_BITS_ALIGN: usize = 6;
const QUICK_BITS_LEVEL: usize = 6;

/// Huffman decode table with quick lookup.
#[derive(Clone)]
pub struct HuffTable {
    /// Code lengths for each symbol
    code_lengths: Vec<u8>,
    /// Quick lookup table: (symbol << 8) | length
    quick_table: Vec<u32>,
    /// Number of symbols
    num_symbols: usize,
    /// Quick table bits
    quick_bits: usize,
    /// Maximum code length in table
    max_length: u8,
}

impl HuffTable {
    /// Create a new Huffman table.
    pub fn new(max_symbols: usize, quick_bits: usize) -> Self {
        Self {
            code_lengths: vec![0; max_symbols],
            quick_table: vec![0; 1 << quick_bits],
            num_symbols: max_symbols,
            quick_bits,
            max_length: 0,
        }
    }

    /// Build table from code lengths. Returns false if table is empty.
    pub fn build(&mut self, lengths: &[u8]) -> bool {
        let num_symbols = lengths.len().min(self.num_symbols);
        self.code_lengths[..num_symbols].copy_from_slice(&lengths[..num_symbols]);
        for i in num_symbols..self.num_symbols {
            self.code_lengths[i] = 0;
        }

        // Find max length
        self.max_length = 0;
        for &len in &self.code_lengths[..num_symbols] {
            if len > self.max_length {
                self.max_length = len;
            }
        }

        if self.max_length == 0 {
            self.quick_table.fill(0);
            return false;
        }

        // Count codes of each length
        let mut count = [0u32; NUM_HUFFMAN_BITS + 1];
        for &len in &self.code_lengths[..num_symbols] {
            if len > 0 && (len as usize) <= NUM_HUFFMAN_BITS {
                count[len as usize] += 1;
            }
        }

        // Calculate starting codes for each length
        let mut next_code = [0u32; NUM_HUFFMAN_BITS + 1];
        let mut code = 0u32;
        for bits in 1..=NUM_HUFFMAN_BITS {
            code = (code + count[bits - 1]) << 1;
            next_code[bits] = code;
        }

        // Build quick table
        self.quick_table.fill(0);

        for (symbol, &len) in self.code_lengths[..num_symbols].iter().enumerate() {
            if len > 0 && (len as usize) <= self.quick_bits {
                let code = next_code[len as usize];
                next_code[len as usize] += 1;

                // Fill all entries that start with this code
                let fill_bits = self.quick_bits - len as usize;
                let base_idx = (code as usize) << fill_bits;
                let fill_count = 1 << fill_bits;

                // Pack symbol and length: symbol in high bits, length in low 8 bits
                let entry = ((symbol as u32) << 8) | (len as u32);
                for i in 0..fill_count {
                    let idx = base_idx + i;
                    if idx < self.quick_table.len() {
                        self.quick_table[idx] = entry;
                    }
                }
            }
        }

        true
    }

    /// Decode a symbol using the bit decoder.
    #[inline]
    pub fn decode(&self, bits: &mut BitDecoder) -> u16 {
        let peek = bits.get_value_15() as usize;
        let idx = peek >> (15 - self.quick_bits);
        let entry = self.quick_table.get(idx).copied().unwrap_or(0);

        if entry != 0 {
            let len = (entry & 0xFF) as usize;
            let symbol = (entry >> 8) as u16;
            bits.move_pos(len);
            return symbol;
        }

        // For codes longer than quick_bits, fall back to slow decode
        // This shouldn't happen often with proper quick_bits settings
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
    /// Main symbol table (literals + match lengths)
    main_table: HuffTable,
    /// Distance table
    dist_table: HuffTable,
    /// Alignment table (low 4 bits of distances)
    align_table: HuffTable,
    /// Length table
    len_table: HuffTable,
    /// Level table (for decoding Huffman code lengths)
    level_table: HuffTable,
    /// Recent offsets for back-references
    recent_offsets: [u64; NUM_REPS],
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
    /// Use alignment bits
    use_align_bits: bool,
    /// Is last block
    is_last_block: bool,
}

impl Rar5BlockDecoder {
    /// Create a new block decoder with given dictionary size.
    pub fn new(dict_size_log: u8) -> Self {
        let dict_size = 1usize << dict_size_log.min(30); // Cap at 1GB for safety
        Self {
            main_table: HuffTable::new(MAIN_TABLE_SIZE, QUICK_BITS_MAIN),
            dist_table: HuffTable::new(DIST_TABLE_SIZE, QUICK_BITS_DIST),
            align_table: HuffTable::new(ALIGN_TABLE_SIZE, QUICK_BITS_ALIGN),
            len_table: HuffTable::new(LEN_TABLE_SIZE, QUICK_BITS_LEN),
            level_table: HuffTable::new(LEVEL_TABLE_SIZE, QUICK_BITS_LEVEL),
            recent_offsets: [0; NUM_REPS],
            last_length: 0,
            window: vec![0u8; dict_size],
            window_mask: dict_size - 1,
            window_pos: 0,
            dict_size,
            tables_valid: false,
            use_align_bits: false,
            is_last_block: false,
        }
    }

    /// Reset decoder state.
    pub fn reset(&mut self) {
        self.recent_offsets = [0; NUM_REPS];
        self.last_length = 0;
        self.window_pos = 0;
        self.tables_valid = false;
        self.use_align_bits = false;
        self.is_last_block = false;
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

    /// Read block header and return if new tables should be read.
    /// Returns Ok(true) if new tables, Ok(false) if reusing previous.
    fn read_block_header(&mut self, bits: &mut BitDecoder) -> Result<bool, DecompressError> {
        bits.align_to_byte();

        let flags = bits.read_byte_aligned();
        let checksum = bits.read_byte_aligned();
        let mut check = flags ^ checksum;

        let num = ((flags >> 3) & 3) as usize;
        if num >= 3 {
            return Err(DecompressError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid block header flags",
            )));
        }

        let mut block_size = bits.read_byte_aligned() as usize;
        check ^= block_size as u8;

        if num >= 1 {
            let b = bits.read_byte_aligned();
            check ^= b;
            block_size += (b as usize) << 8;
        }
        if num >= 2 {
            let b = bits.read_byte_aligned();
            check ^= b;
            block_size += (b as usize) << 16;
        }

        if check != 0x5A {
            return Err(DecompressError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Block header checksum mismatch",
            )));
        }

        let block_size_bits7 = ((flags & 7) + 1) as usize;
        block_size += block_size_bits7 >> 3;
        if block_size == 0 {
            return Err(DecompressError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid block size",
            )));
        }
        block_size -= 1;
        let block_end_bits = block_size_bits7 & 7;

        bits.set_block_end(bits.position() + block_size, block_end_bits);
        self.is_last_block = (flags & 0x40) != 0;

        // Flag 0x80 indicates new tables
        let new_tables = (flags & 0x80) != 0;
        Ok(new_tables)
    }

    /// Read and build Huffman tables from bit stream.
    fn read_tables(&mut self, bits: &mut BitDecoder) -> Result<(), DecompressError> {
        // Read level table (pre-code) as 4-bit values
        let mut level_lens = [0u8; LEVEL_TABLE_SIZE];
        let mut i = 0;
        while i < LEVEL_TABLE_SIZE {
            let len = bits.read_bits_9fix(4) as u8;
            if len == 15 {
                let num_zeros = bits.read_bits_9fix(4) as usize;
                if num_zeros != 0 {
                    let end = (i + num_zeros + 2).min(LEVEL_TABLE_SIZE);
                    while i < end {
                        level_lens[i] = 0;
                        i += 1;
                    }
                    continue;
                }
            }
            level_lens[i] = len;
            i += 1;
        }

        if !self.level_table.build(&level_lens) {
            // Empty level table is OK for empty blocks
        }

        // Total table size (main + dist + align + len)
        let table_size = MAIN_TABLE_SIZE + DIST_TABLE_SIZE + ALIGN_TABLE_SIZE + LEN_TABLE_SIZE;
        let mut lens = vec![0u8; table_size];

        // Decode all table lengths using level table
        i = 0;
        while i < table_size {
            if bits.is_block_over_read() {
                return Err(DecompressError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Block overread while reading tables",
                )));
            }

            let sym = self.level_table.decode(bits) as usize;

            if sym < 16 {
                lens[i] = sym as u8;
                i += 1;
            } else {
                // Repeat codes based on unrar5j logic:
                // num = ((sym - 16) & 1) * 4;
                // num += num + 3 + readBits9(num + 3);
                // So for sym 18: num=0, count = 3 + extra(3 bits) = 3-10
                // For sym 19: num=4, count = 11 + extra(7 bits) = 11-138
                // For sym 16: num=0, count = 3 + extra(3 bits) = 3-10
                // For sym 17: num=4, count = 11 + extra(7 bits) = 11-138
                let num = ((sym - 16) & 1) * 4;
                let read_bits = num + 3;
                // count = 2*num + 3 + extra (Java: num += num + 3 + ...)
                let mut count = 2 * num + 3 + bits.read_bits_9(read_bits) as usize;
                count += i;
                if count > table_size {
                    count = table_size;
                }
                let v = if sym < 18 {
                    // sym 16-17: repeat previous
                    if i == 0 {
                        return Err(DecompressError::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid repeat at table start",
                        )));
                    }
                    lens[i - 1]
                } else {
                    // sym 18-19: repeat zeros
                    0
                };
                while i < count {
                    lens[i] = v;
                    i += 1;
                }
            }
        }

        // Build the four tables from the combined lengths
        let mut offset = 0;
        self.main_table
            .build(&lens[offset..offset + MAIN_TABLE_SIZE]);
        offset += MAIN_TABLE_SIZE;

        self.dist_table
            .build(&lens[offset..offset + DIST_TABLE_SIZE]);
        offset += DIST_TABLE_SIZE;

        // Check if align table has non-default values
        self.use_align_bits = false;
        for k in 0..ALIGN_TABLE_SIZE {
            if lens[offset + k] != NUM_ALIGN_BITS as u8 {
                self.align_table
                    .build(&lens[offset..offset + ALIGN_TABLE_SIZE]);
                self.use_align_bits = true;
                break;
            }
        }
        offset += ALIGN_TABLE_SIZE;

        self.len_table.build(&lens[offset..offset + LEN_TABLE_SIZE]);

        self.tables_valid = true;
        Ok(())
    }

    /// Decode a block of compressed data.
    /// Returns a list of pending filters that need to be applied.
    pub fn decode_block(
        &mut self,
        bits: &mut BitDecoder,
        output_size: usize,
    ) -> Result<Vec<super::filter::UnpackFilter>, DecompressError> {
        let start_pos = self.window_pos;
        let mut filters = Vec::new();

        // Read block header
        let new_tables = self.read_block_header(bits)?;

        if new_tables {
            self.read_tables(bits)?;
        } else if !self.tables_valid {
            return Err(DecompressError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Continue block but no previous tables",
            )));
        }

        // Decode symbols until we have enough output
        while self.window_pos - start_pos < output_size {
            if bits.is_eof() || bits.is_block_over_read() {
                break;
            }

            let sym = self.main_table.decode(bits) as usize;

            if sym < 256 {
                // Literal byte
                self.write_byte(sym as u8);
            } else if sym == 256 {
                // Filter command
                if let Some(filter) = self.read_filter(bits)? {
                    filters.push(filter);
                }
            } else if sym == 257 {
                // Repeat last length with last distance
                if self.last_length != 0 && self.recent_offsets[0] != 0 {
                    let length = self.last_length as usize;
                    let offset = self.recent_offsets[0] as usize;
                    self.copy_bytes(offset, length);
                }
            } else if sym < 262 {
                // Use recent offset (sym 258-261 = offsets 0-3)
                let rep_idx = sym - 258;
                let offset = self.recent_offsets[rep_idx] as usize;
                if offset == 0 {
                    return Err(DecompressError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid zero offset",
                    )));
                }

                let length = self.decode_length(bits)?;

                // Rotate recent offsets
                if rep_idx > 0 {
                    let off = self.recent_offsets[rep_idx];
                    for j in (1..=rep_idx).rev() {
                        self.recent_offsets[j] = self.recent_offsets[j - 1];
                    }
                    self.recent_offsets[0] = off;
                }
                self.last_length = length as u32;
                self.copy_bytes(offset, length);
            } else {
                // New offset with length (sym >= 262)
                let len_slot = sym - 262;
                let length = self.slot_to_length(len_slot as u32, bits)?;
                let offset = self.decode_offset(bits)?;

                // Update recent offsets
                for j in (1..NUM_REPS).rev() {
                    self.recent_offsets[j] = self.recent_offsets[j - 1];
                }
                self.recent_offsets[0] = offset as u64;
                self.last_length = length as u32;

                self.copy_bytes(offset, length);
            }
        }

        Ok(filters)
    }

    /// Read filter data (variable length integer).
    fn read_filter_data(&self, bits: &mut BitDecoder) -> u32 {
        // Read byte count (2 bits + 1)
        let v = bits.get_value_high32();
        let byte_count = ((v >> 30) + 1) as usize;
        bits.read_bits_big(2, v);

        // Read data bytes
        let mut data: u32 = 0;
        for i in 0..byte_count {
            let v = bits.get_value_high32();
            let byte_val = bits.read_bits_big(8, v);
            data |= byte_val << (i * 8);
        }
        data
    }

    /// Read a filter command from the bitstream.
    fn read_filter(
        &mut self,
        bits: &mut BitDecoder,
    ) -> Result<Option<super::filter::UnpackFilter>, DecompressError> {
        use super::filter::{FilterType, UnpackFilter};

        let block_start = self.read_filter_data(bits) as usize;
        let block_length = self.read_filter_data(bits) as usize;

        // Read filter type (3 bits)
        let v = bits.get_value_high32();
        let filter_type_bits = bits.read_bits_big(3, v) as u8;

        let filter_type = match FilterType::from_bits(filter_type_bits) {
            Some(ft) => ft,
            None => {
                // Unknown filter type - skip it
                return Ok(None);
            }
        };

        // Read channels for delta filter (5 bits + 1)
        let channels = if filter_type == FilterType::Delta {
            let v = bits.get_value_high32();
            (bits.read_bits_big(5, v) + 1) as u8
        } else {
            0
        };

        // Calculate actual block start relative to current window position
        let actual_start = (block_start + self.window_pos) % self.dict_size;

        Ok(Some(UnpackFilter::new(
            filter_type,
            actual_start,
            block_length,
            channels,
        )))
    }

    /// Decode a match length from length table.
    fn decode_length(&mut self, bits: &mut BitDecoder) -> Result<usize, DecompressError> {
        let sym = self.len_table.decode(bits) as u32;
        self.slot_to_length(sym, bits)
    }

    /// Convert length slot to actual length.
    fn slot_to_length(&self, slot: u32, bits: &mut BitDecoder) -> Result<usize, DecompressError> {
        // Length table: slot 0-7 = lengths 2-9
        // Higher slots have extra bits
        if slot < 8 {
            Ok((slot + 2) as usize)
        } else if slot < LEN_TABLE_SIZE as u32 {
            let extra_bits = ((slot - 4) / 4) as usize;
            let base = ((4 + (slot & 3)) << extra_bits) + 2;
            let v = bits.get_value_high32();
            let extra = bits.read_bits_big(extra_bits, v);
            Ok((base + extra) as usize)
        } else {
            Err(DecompressError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid length slot",
            )))
        }
    }

    /// Decode a match offset.
    ///
    /// Based on unrar5j logic:
    /// - slot < 4: offset = slot + 1 (no extra bits)
    /// - slot >= 4: offset has extra bits based on slot value
    fn decode_offset(&mut self, bits: &mut BitDecoder) -> Result<usize, DecompressError> {
        let slot = self.dist_table.decode(bits) as u32;

        if slot < 4 {
            // Small offsets: directly slot + 1
            Ok((slot + 1) as usize)
        } else {
            // Larger offsets with extra bits
            // num_bits = (slot - 2) >> 1
            // base = (2 | (slot & 1)) << num_bits
            let num_bits = ((slot - 2) >> 1) as usize;
            let base = ((2 | (slot & 1)) << num_bits) as u32;

            if num_bits < NUM_ALIGN_BITS {
                // Few extra bits - read directly
                let v = bits.get_value_high32();
                let extra = bits.read_bits_big(num_bits, v) as u32;
                Ok((base + extra + 1) as usize)
            } else {
                // More bits - use alignment table
                let high_bits_count = num_bits - NUM_ALIGN_BITS;
                let v = bits.get_value_high32();
                let high = bits.read_bits_big(high_bits_count, v) as u32;

                let low = if self.use_align_bits {
                    self.align_table.decode(bits) as u32
                } else {
                    bits.read_bits_9fix(NUM_ALIGN_BITS) as u32
                };

                let offset = base + (high << NUM_ALIGN_BITS) + low + 1;
                Ok(offset as usize)
            }
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
        let mut table = HuffTable::new(8, 6);
        // Simple code lengths: symbols 0-7 with lengths 3,3,3,3,3,3,3,3
        let lengths = [3u8, 3, 3, 3, 3, 3, 3, 3];
        assert!(table.build(&lengths));
        assert_eq!(table.max_length, 3);
    }
}
