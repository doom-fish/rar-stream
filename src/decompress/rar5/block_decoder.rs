//! RAR5 LZSS decoder with Huffman tables.
//!
//! RAR5 uses a block-based format where each block contains:
//! 1. Block header with flags, checksum, and size
//! 2. Huffman tables (if flag indicates new tables)
//! 3. Compressed data stream
//!
//! Multi-threaded mode (with `parallel` feature):
//! - Phase 1: Decode Huffman symbols to DecodedItem buffer (parallelizable)
//! - Phase 2: Apply decoded items to sliding window (sequential)

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

/// Decoded item from Huffman decoding phase.
/// Used for multi-threaded decompression where decoding and output are separate.
#[derive(Debug, Clone)]
pub enum DecodedItem {
    /// Literal bytes (up to 8 bytes packed for efficiency)
    Literal { bytes: [u8; 8], len: u8 },
    /// Match with explicit offset
    Match { length: u32, offset: usize },
    /// Repeat with recent offset index (0-3)
    Rep { length: u32, rep_idx: u8 },
    /// Full repeat (reuse last length and offset[0])
    FullRep,
    /// Filter command
    Filter { filter_type: u8, block_start: usize, block_length: usize, channels: u8 },
}

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
    /// For slow decode: decode_len[i] = first code of length i+1 (left-aligned to 16 bits)
    decode_len: [u32; 16],
    /// For slow decode: first_symbol[i] = first symbol index for length i
    first_symbol: [u16; 16],
    /// Symbol permutation (sorted by code)
    symbols: Vec<u16>,
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
            decode_len: [0; 16],
            first_symbol: [0; 16],
            symbols: vec![0; max_symbols],
        }
    }

    /// Build table from code lengths. Returns false if table is empty.
    /// Based on unrar's MakeDecodeTables.
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

        // Count codes of each length (unrar: LengthCount)
        let mut length_count = [0u32; 16];
        for &len in &self.code_lengths[..num_symbols] {
            if len > 0 && len < 16 {
                length_count[len as usize] += 1;
            }
        }

        // Build decode_len (left-aligned upper limit for each bit length)
        // and decode_pos (start position in symbols array for each length)
        let mut decode_pos = [0u32; 16];
        let mut upper_limit = 0u32;
        for i in 1..16 {
            upper_limit += length_count[i];
            // Left-aligned upper limit
            self.decode_len[i] = upper_limit << (16 - i);
            upper_limit *= 2;
            // Start position for this length
            decode_pos[i] = decode_pos[i - 1] + length_count[i - 1];
        }
        self.first_symbol = [
            decode_pos[0] as u16, decode_pos[1] as u16, decode_pos[2] as u16, decode_pos[3] as u16,
            decode_pos[4] as u16, decode_pos[5] as u16, decode_pos[6] as u16, decode_pos[7] as u16,
            decode_pos[8] as u16, decode_pos[9] as u16, decode_pos[10] as u16, decode_pos[11] as u16,
            decode_pos[12] as u16, decode_pos[13] as u16, decode_pos[14] as u16, decode_pos[15] as u16,
        ];

        // Build symbols array (unrar: DecodeNum)
        self.symbols.fill(0);
        let mut copy_pos = decode_pos;
        for (symbol, &len) in self.code_lengths[..num_symbols].iter().enumerate() {
            if len > 0 && len < 16 {
                let pos = copy_pos[len as usize] as usize;
                if pos < self.symbols.len() {
                    self.symbols[pos] = symbol as u16;
                }
                copy_pos[len as usize] += 1;
            }
        }

        // Build quick table for fast decode
        self.quick_table.fill(0);
        let mut cur_bit_length = 1usize;
        let quick_size = 1 << self.quick_bits;
        
        for code in 0..quick_size {
            // Left-align the code
            let bit_field = (code << (16 - self.quick_bits)) as u32;
            
            // Find the bit length for this code
            while cur_bit_length < self.quick_bits && bit_field >= self.decode_len[cur_bit_length] {
                cur_bit_length += 1;
            }
            
            if bit_field < self.decode_len[cur_bit_length] {
                // Calculate position in symbols array
                let dist = if cur_bit_length > 0 {
                    bit_field.wrapping_sub(self.decode_len[cur_bit_length - 1])
                } else {
                    bit_field
                };
                let dist_shifted = dist >> (16 - cur_bit_length);
                let pos = decode_pos[cur_bit_length] + dist_shifted;
                
                if (pos as usize) < self.symbols.len() {
                    let symbol = self.symbols[pos as usize];
                    // Pack: symbol in high bits, length in low 8 bits
                    self.quick_table[code] = ((symbol as u32) << 8) | (cur_bit_length as u32);
                }
            }
        }

        true
    }

    /// Decode a symbol using the bit decoder.
    /// Based on unrar's DecodeNumber.
    #[inline]
    pub fn decode(&self, bits: &mut BitDecoder) -> u16 {
        // Get 15 bits left-aligned (unrar uses 16-bit with mask 0xfffe)
        let bit_field = (bits.get_value_15() << 1) as u32;

        // Quick decode path
        if bit_field < self.decode_len[self.quick_bits] {
            let code = (bit_field >> (16 - self.quick_bits)) as usize;
            let entry = self.quick_table.get(code).copied().unwrap_or(0);
            if entry != 0 {
                let len = (entry & 0xFF) as usize;
                let symbol = (entry >> 8) as u16;
                bits.move_pos(len);
                return symbol;
            }
        }

        // Slow path: find the matching bit length
        let mut bit_len = 15usize;
        for i in (self.quick_bits + 1)..15 {
            if bit_field < self.decode_len[i] {
                bit_len = i;
                break;
            }
        }

        bits.move_pos(bit_len);

        // Calculate distance from start code for this bit length
        let dist = bit_field.wrapping_sub(self.decode_len[bit_len - 1]);
        let dist_shifted = dist >> (16 - bit_len);
        let pos = (self.first_symbol[bit_len] as u32) + dist_shifted;

        // Safety check
        if (pos as usize) >= self.symbols.len() {
            return 0;
        }

        self.symbols[pos as usize]
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

/// Block header metadata for multi-threaded decoding.
#[cfg(feature = "parallel")]
#[derive(Debug, Clone)]
pub struct BlockHeader {
    /// Byte offset where block data starts (after header)
    pub block_start: usize,
    /// Size of block data in bytes
    pub block_size: usize,
    /// Number of valid bits in last byte (1-8)
    pub block_bit_size: usize,
    /// Whether this block has new Huffman tables
    pub table_present: bool,
    /// Whether this is the last block in the file
    pub last_block: bool,
}

/// Per-block Huffman tables for multi-threaded decoding.
#[cfg(feature = "parallel")]
#[derive(Clone)]
pub struct BlockTables {
    pub main_table: HuffTable,
    pub dist_table: HuffTable,
    pub align_table: HuffTable,
    pub len_table: HuffTable,
    pub use_align_bits: bool,
}

#[cfg(feature = "parallel")]
impl BlockTables {
    pub fn new() -> Self {
        Self {
            main_table: HuffTable::new(MAIN_TABLE_SIZE, QUICK_BITS_MAIN),
            dist_table: HuffTable::new(DIST_TABLE_SIZE, QUICK_BITS_DIST),
            align_table: HuffTable::new(ALIGN_TABLE_SIZE, QUICK_BITS_ALIGN),
            len_table: HuffTable::new(LEN_TABLE_SIZE, QUICK_BITS_LEN),
            use_align_bits: false,
        }
    }
}

/// Thread-local data for parallel block decoding.
#[cfg(feature = "parallel")]
pub struct UnpackThreadData {
    /// Block header info
    pub header: BlockHeader,
    /// Huffman tables for this block
    pub tables: BlockTables,
    /// Decoded items from this block
    pub decoded: Vec<DecodedItem>,
    /// Estimated output size from this block
    pub output_size: usize,
    /// Whether decoding was incomplete (needs more data)
    pub incomplete: bool,
    /// Whether this is a large block (use single-threaded path)
    pub large_block: bool,
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

        // BlockBitSize is the number of valid bits in the last byte (1-8)
        let block_bit_size = ((flags & 7) + 1) as usize;
        
        // block_size is bytes, block_end is position of last byte
        // unrar: ReadBorder = BlockStart + BlockSize - 1
        let block_start = bits.position();
        let block_end = block_start + block_size - 1;
        bits.set_block_end(block_end, block_bit_size);

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

        // Decode symbols until we have enough output
        while self.window_pos - start_pos < output_size {
            // Check if we need to read a new block header
            if bits.is_block_over_read() || !self.tables_valid {
                // If this was the last block, we're done
                if self.is_last_block {
                    break;
                }
                
                if bits.is_eof() {
                    break;
                }
                
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
            }

            if bits.is_eof() {
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

    /// Decode symbols from a block into a buffer without touching the window.
    /// This is the first phase of two-phase decoding for multi-threaded support.
    /// Returns (decoded_items, estimated_output_size, is_last_block).
    #[cfg(feature = "parallel")]
    pub fn decode_symbols(
        &mut self,
        bits: &mut BitDecoder,
        max_symbols: usize,
    ) -> Result<(Vec<DecodedItem>, usize, bool), DecompressError> {
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

        let mut items = Vec::with_capacity(max_symbols.min(0x4000));
        let mut output_size = 0usize;
        let mut literal_buf = [0u8; 8];
        let mut literal_count = 0u8;

        // Flush pending literals as a DecodedItem
        let flush_literals = |items: &mut Vec<DecodedItem>, buf: &mut [u8; 8], count: &mut u8| {
            if *count > 0 {
                items.push(DecodedItem::Literal { bytes: *buf, len: *count - 1 });
                *count = 0;
            }
        };

        while items.len() < max_symbols {
            if bits.is_eof() || bits.is_block_over_read() {
                break;
            }

            let sym = self.main_table.decode(bits) as usize;

            if sym < 256 {
                // Literal byte - batch up to 8
                literal_buf[literal_count as usize] = sym as u8;
                literal_count += 1;
                output_size += 1;
                if literal_count == 8 {
                    flush_literals(&mut items, &mut literal_buf, &mut literal_count);
                }
            } else {
                // Non-literal - flush pending literals first
                flush_literals(&mut items, &mut literal_buf, &mut literal_count);

                if sym == 256 {
                    // Filter command
                    let block_start = self.read_filter_data(bits) as usize;
                    let block_length = self.read_filter_data(bits) as usize;
                    let v = bits.get_value_high32();
                    let filter_type = bits.read_bits_big(3, v) as u8;
                    let channels = if filter_type == 3 {
                        // Delta filter
                        let v = bits.get_value_high32();
                        (bits.read_bits_big(5, v) + 1) as u8
                    } else {
                        0
                    };
                    items.push(DecodedItem::Filter { filter_type, block_start, block_length, channels });
                } else if sym == 257 {
                    // Full repeat
                    if self.last_length != 0 {
                        output_size += self.last_length as usize;
                    }
                    items.push(DecodedItem::FullRep);
                } else if sym < 262 {
                    // Use recent offset
                    let rep_idx = (sym - 258) as u8;
                    let length = self.decode_length(bits)? as u32;
                    output_size += length as usize;
                    // Update state for FullRep
                    self.last_length = length;
                    items.push(DecodedItem::Rep { length, rep_idx });
                } else {
                    // New match with offset
                    let len_slot = sym - 262;
                    let length = self.slot_to_length(len_slot as u32, bits)? as u32;
                    let offset = self.decode_offset(bits)?;
                    output_size += length as usize;
                    // Update state
                    self.last_length = length;
                    items.push(DecodedItem::Match { length, offset });
                }
            }
        }

        // Flush remaining literals
        flush_literals(&mut items, &mut literal_buf, &mut literal_count);

        Ok((items, output_size, self.is_last_block))
    }

    /// Apply decoded items to the sliding window (phase 2 of two-phase decoding).
    /// Must be called sequentially on the main thread.
    #[cfg(feature = "parallel")]
    pub fn apply_decoded(&mut self, items: &[DecodedItem]) -> Result<Vec<super::filter::UnpackFilter>, DecompressError> {
        use super::filter::{FilterType, UnpackFilter};
        let mut filters = Vec::new();

        for item in items {
            match item {
                DecodedItem::Literal { bytes, len } => {
                    for i in 0..=*len as usize {
                        self.write_byte(bytes[i]);
                    }
                }
                DecodedItem::Match { length, offset } => {
                    // Update recent offsets
                    for j in (1..NUM_REPS).rev() {
                        self.recent_offsets[j] = self.recent_offsets[j - 1];
                    }
                    self.recent_offsets[0] = *offset as u64;
                    self.last_length = *length;
                    self.copy_bytes(*offset, *length as usize);
                }
                DecodedItem::Rep { length, rep_idx } => {
                    let rep_idx = *rep_idx as usize;
                    let offset = self.recent_offsets[rep_idx] as usize;
                    if offset == 0 {
                        return Err(DecompressError::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid zero offset in Rep",
                        )));
                    }
                    // Rotate recent offsets
                    if rep_idx > 0 {
                        let off = self.recent_offsets[rep_idx];
                        for j in (1..=rep_idx).rev() {
                            self.recent_offsets[j] = self.recent_offsets[j - 1];
                        }
                        self.recent_offsets[0] = off;
                    }
                    self.last_length = *length;
                    self.copy_bytes(offset, *length as usize);
                }
                DecodedItem::FullRep => {
                    if self.last_length != 0 && self.recent_offsets[0] != 0 {
                        let length = self.last_length as usize;
                        let offset = self.recent_offsets[0] as usize;
                        self.copy_bytes(offset, length);
                    }
                }
                DecodedItem::Filter { filter_type, block_start, block_length, channels } => {
                    if let Some(ft) = FilterType::from_bits(*filter_type) {
                        let actual_start = (*block_start + self.window_pos) % self.dict_size;
                        filters.push(UnpackFilter::new(ft, actual_start, *block_length, *channels));
                    }
                }
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
            // num_bits = slot / 2 - 1
            // base = (2 | (slot & 1)) << num_bits
            let num_bits = (slot / 2 - 1) as usize;
            let base = (2 | (slot & 1)) << num_bits;

            if num_bits < NUM_ALIGN_BITS {
                // Few extra bits - read directly
                let v = bits.get_value_high32();
                let extra = bits.read_bits_big(num_bits, v);
                Ok((base + extra + 1) as usize)
            } else {
                // More bits - use alignment table
                // Only read high bits if num_bits > 4
                let high = if num_bits > NUM_ALIGN_BITS {
                    let high_bits_count = num_bits - NUM_ALIGN_BITS;
                    let v = bits.get_value_high32();
                    bits.read_bits_big(high_bits_count, v)
                } else {
                    0
                };

                let low = if self.use_align_bits {
                    self.align_table.decode(bits) as u32
                } else {
                    bits.read_bits_9fix(NUM_ALIGN_BITS)
                };

                let offset = base + (high << NUM_ALIGN_BITS) + low + 1;
                Ok(offset as usize)
            }
        }
    }
}

// ============================================================================
// Multi-threaded decoding support
// ============================================================================

#[cfg(feature = "parallel")]
impl Rar5BlockDecoder {
    /// Scan compressed data to find block boundaries.
    /// Returns a list of block headers that can be decoded in parallel.
    /// 
    /// # Arguments
    /// * `data` - The compressed data buffer
    /// * `max_blocks` - Maximum number of blocks to scan
    /// 
    /// # Returns
    /// Vector of (block_start, BlockHeader) pairs
    pub fn scan_blocks(
        &mut self,
        data: &[u8],
        max_blocks: usize,
    ) -> Result<Vec<(usize, BlockHeader)>, DecompressError> {
        use super::bit_decoder::BitDecoder;
        
        let mut bits = BitDecoder::new(data);
        let mut blocks = Vec::with_capacity(max_blocks);
        let large_block_size = 0x20000; // 128KB threshold
        
        while blocks.len() < max_blocks && !bits.is_eof() {
            let header_start = bits.position();
            
            // Read block header (same as read_block_header but returns metadata)
            bits.align_to_byte();
            
            if bits.position() + 3 > data.len() {
                break; // Not enough data for header
            }
            
            let flags = bits.read_byte_aligned();
            let checksum = bits.read_byte_aligned();
            let mut check = flags ^ checksum;
            
            let num = ((flags >> 3) & 3) as usize;
            if num >= 3 {
                break; // Invalid header
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
                break; // Checksum mismatch
            }
            
            let block_bit_size = ((flags & 7) + 1) as usize;
            let block_start = bits.position();
            let table_present = (flags & 0x80) != 0;
            let last_block = (flags & 0x40) != 0;
            
            // Check if block fits in buffer
            if block_start + block_size > data.len() {
                break; // Block extends beyond buffer
            }
            
            let header = BlockHeader {
                block_start,
                block_size,
                block_bit_size,
                table_present,
                last_block,
            };
            
            // Skip large blocks - they'll use single-threaded path
            if block_size > large_block_size {
                blocks.push((header_start, header));
                break;
            }
            
            blocks.push((header_start, header));
            
            if last_block {
                break;
            }
            
            // Move to next block
            bits.set_position(block_start + block_size);
        }
        
        Ok(blocks)
    }
    
    /// Read Huffman tables from a block into a BlockTables structure.
    /// This is used to prepare tables for parallel decoding.
    pub fn read_tables_into(
        &self,
        bits: &mut BitDecoder,
        tables: &mut BlockTables,
    ) -> Result<(), DecompressError> {
        // Read level table (pre-code) as 4-bit values
        let mut level_lens = [0u8; LEVEL_TABLE_SIZE];
        let mut level_table = HuffTable::new(LEVEL_TABLE_SIZE, QUICK_BITS_LEVEL);
        
        let mut i = 0;
        while i < LEVEL_TABLE_SIZE {
            let v = bits.get_value_high32();
            let len = bits.read_bits_big(4, v) as u8;
            if len == 15 {
                // Zero run
                let v = bits.get_value_high32();
                let count = bits.read_bits_big(4, v) as usize;
                if count == 0 {
                    level_lens[i] = 15;
                    i += 1;
                } else {
                    let fill_count = (count + 2).min(LEVEL_TABLE_SIZE - i);
                    for j in 0..fill_count {
                        level_lens[i + j] = 0;
                    }
                    i += fill_count;
                }
            } else {
                level_lens[i] = len;
                i += 1;
            }
        }
        
        level_table.build(&level_lens);
        
        // Calculate total symbols needed
        let total_symbols = MAIN_TABLE_SIZE + DIST_TABLE_SIZE + ALIGN_TABLE_SIZE + LEN_TABLE_SIZE;
        let mut lens = vec![0u8; total_symbols];
        
        // Decode all code lengths using level table
        let mut idx = 0;
        while idx < total_symbols {
            let sym = level_table.decode(bits) as usize;
            
            if sym < 16 {
                lens[idx] = sym as u8;
                idx += 1;
            } else if sym == 16 {
                // Copy previous 3-6 times
                let v = bits.get_value_high32();
                let count = 3 + bits.read_bits_big(2, v) as usize;
                let prev = if idx > 0 { lens[idx - 1] } else { 0 };
                let fill_count = count.min(total_symbols - idx);
                for j in 0..fill_count {
                    lens[idx + j] = prev;
                }
                idx += fill_count;
            } else if sym == 17 {
                // 3-10 zeros
                let v = bits.get_value_high32();
                let count = 3 + bits.read_bits_big(3, v) as usize;
                let fill_count = count.min(total_symbols - idx);
                for j in 0..fill_count {
                    lens[idx + j] = 0;
                }
                idx += fill_count;
            } else if sym == 18 {
                // 11-138 zeros
                let v = bits.get_value_high32();
                let count = 11 + bits.read_bits_big(7, v) as usize;
                let fill_count = count.min(total_symbols - idx);
                for j in 0..fill_count {
                    lens[idx + j] = 0;
                }
                idx += fill_count;
            }
        }
        
        // Build the four tables
        let mut offset = 0;
        tables.main_table.build(&lens[offset..offset + MAIN_TABLE_SIZE]);
        offset += MAIN_TABLE_SIZE;
        
        tables.dist_table.build(&lens[offset..offset + DIST_TABLE_SIZE]);
        offset += DIST_TABLE_SIZE;
        
        // Check if align table has non-default values
        tables.use_align_bits = false;
        for k in 0..ALIGN_TABLE_SIZE {
            if lens[offset + k] != NUM_ALIGN_BITS as u8 {
                tables.align_table.build(&lens[offset..offset + ALIGN_TABLE_SIZE]);
                tables.use_align_bits = true;
                break;
            }
        }
        offset += ALIGN_TABLE_SIZE;
        
        tables.len_table.build(&lens[offset..offset + LEN_TABLE_SIZE]);
        
        Ok(())
    }
    
    /// Decode a single block using provided tables, outputting to DecodedItem buffer.
    /// This is stateless except for the tables, suitable for parallel execution.
    pub fn decode_block_stateless(
        bits: &mut BitDecoder,
        tables: &BlockTables,
        header: &BlockHeader,
        max_items: usize,
    ) -> Result<(Vec<DecodedItem>, usize), DecompressError> {
        let mut items = Vec::with_capacity(max_items.min(0x4100));
        let mut output_size = 0usize;
        let mut literal_buf = [0u8; 8];
        let mut literal_count = 0u8;
        let mut last_length = 0u32;
        
        // Set block boundary
        let block_end = header.block_start + header.block_size - 1;
        bits.set_block_end(block_end, header.block_bit_size);
        
        let flush_literals = |items: &mut Vec<DecodedItem>, buf: &mut [u8; 8], count: &mut u8| {
            if *count > 0 {
                items.push(DecodedItem::Literal { bytes: *buf, len: *count - 1 });
                *count = 0;
            }
        };
        
        while items.len() < max_items {
            if bits.is_eof() || bits.is_block_over_read() {
                break;
            }
            
            let sym = tables.main_table.decode(bits) as usize;
            
            if sym < 256 {
                // Literal byte
                literal_buf[literal_count as usize] = sym as u8;
                literal_count += 1;
                output_size += 1;
                if literal_count == 8 {
                    flush_literals(&mut items, &mut literal_buf, &mut literal_count);
                }
            } else {
                flush_literals(&mut items, &mut literal_buf, &mut literal_count);
                
                if sym == 256 {
                    // Filter command - read inline since we don't have self
                    let read_filter_data = |bits: &mut BitDecoder| -> u32 {
                        let v = bits.get_value_high32();
                        let byte_count = ((v >> 30) + 1) as usize;
                        bits.read_bits_big(2, v);
                        let mut data: u32 = 0;
                        for i in 0..byte_count {
                            let v = bits.get_value_high32();
                            let byte_val = bits.read_bits_big(8, v);
                            data |= byte_val << (i * 8);
                        }
                        data
                    };
                    
                    let block_start = read_filter_data(bits) as usize;
                    let block_length = read_filter_data(bits) as usize;
                    let v = bits.get_value_high32();
                    let filter_type = bits.read_bits_big(3, v) as u8;
                    let channels = if filter_type == 3 {
                        let v = bits.get_value_high32();
                        (bits.read_bits_big(5, v) + 1) as u8
                    } else {
                        0
                    };
                    items.push(DecodedItem::Filter { filter_type, block_start, block_length, channels });
                } else if sym == 257 {
                    // Full repeat
                    if last_length != 0 {
                        output_size += last_length as usize;
                    }
                    items.push(DecodedItem::FullRep);
                } else if sym < 262 {
                    // Rep with index
                    let rep_idx = (sym - 258) as u8;
                    
                    // Decode length
                    let length_slot = tables.len_table.decode(bits) as u32;
                    let length = Self::slot_to_length_static(length_slot, bits);
                    
                    output_size += length as usize;
                    last_length = length;
                    items.push(DecodedItem::Rep { length, rep_idx });
                } else {
                    // Match with distance
                    let len_slot = (sym - 262) as u32;
                    let length = Self::slot_to_length_static(len_slot, bits);
                    
                    // Decode distance
                    let dist_slot = tables.dist_table.decode(bits) as u32;
                    let offset = Self::decode_offset_static(dist_slot, tables, bits)?;
                    
                    output_size += length as usize;
                    last_length = length;
                    items.push(DecodedItem::Match { length, offset });
                }
            }
        }
        
        flush_literals(&mut items, &mut literal_buf, &mut literal_count);
        
        Ok((items, output_size))
    }
    
    /// Static version of slot_to_length for stateless decoding.
    fn slot_to_length_static(slot: u32, bits: &mut BitDecoder) -> u32 {
        // Length = 2 + slot for slots 0-7
        // Length = 2 + 8 + (bits read) for higher slots
        if slot < 8 {
            2 + slot
        } else {
            let num_bits = (slot / 4) as usize;
            let base = 2 + ((2 + (slot & 3)) << num_bits);
            let v = bits.get_value_high32();
            let extra = bits.read_bits_big(num_bits, v);
            base + extra
        }
    }
    
    /// Static version of decode_offset for stateless decoding.
    fn decode_offset_static(
        slot: u32,
        tables: &BlockTables,
        bits: &mut BitDecoder,
    ) -> Result<usize, DecompressError> {
        if slot < 4 {
            Ok((slot + 1) as usize)
        } else {
            let num_bits = (slot / 2 - 1) as usize;
            let base = (2 | (slot & 1)) << num_bits;
            
            if num_bits < NUM_ALIGN_BITS {
                let v = bits.get_value_high32();
                let extra = bits.read_bits_big(num_bits, v);
                Ok((base + extra + 1) as usize)
            } else {
                let high = if num_bits > NUM_ALIGN_BITS {
                    let high_bits_count = num_bits - NUM_ALIGN_BITS;
                    let v = bits.get_value_high32();
                    bits.read_bits_big(high_bits_count, v)
                } else {
                    0
                };
                
                let low = if tables.use_align_bits {
                    tables.align_table.decode(bits) as u32
                } else {
                    bits.read_bits_9fix(NUM_ALIGN_BITS)
                };
                
                let offset = base + (high << NUM_ALIGN_BITS) + low + 1;
                Ok(offset as usize)
            }
        }
    }
    
    /// Decode multiple blocks in parallel using rayon.
    /// Returns the combined output as a byte vector.
    pub fn decode_parallel(
        &mut self,
        data: &[u8],
        output_size: usize,
    ) -> Result<Vec<u8>, DecompressError> {
        use rayon::prelude::*;
        use super::bit_decoder::BitDecoder;
        
        // Scan for block boundaries
        let max_blocks = rayon::current_num_threads() * 2;
        let block_infos = self.scan_blocks(data, max_blocks)?;
        
        if block_infos.is_empty() {
            return Err(DecompressError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "No valid blocks found",
            )));
        }
        
        // Prepare tables for each block
        // For blocks with table_present=true, we read their tables
        // For blocks without, they inherit from previous block
        let mut block_tables: Vec<BlockTables> = Vec::with_capacity(block_infos.len());
        let mut current_tables = BlockTables::new();
        
        for (header_start, header) in &block_infos {
            if header.table_present {
                let mut bits = BitDecoder::new(data);
                bits.set_position(header.block_start);
                self.read_tables_into(&mut bits, &mut current_tables)?;
            }
            block_tables.push(current_tables.clone());
        }
        
        // Parallel decode each block
        let decoded_blocks: Vec<Result<(Vec<DecodedItem>, usize), DecompressError>> = block_infos
            .par_iter()
            .zip(block_tables.par_iter())
            .map(|((_, header), tables)| {
                let mut bits = BitDecoder::new(data);
                bits.set_position(header.block_start);
                
                // Skip tables if they were present (already parsed)
                if header.table_present {
                    // Need to skip past tables - for now just decode from block start
                    // The tables reading will advance the bit position
                }
                
                Self::decode_block_stateless(&mut bits, tables, header, 0x4100)
            })
            .collect();
        
        // Apply decoded items sequentially
        let mut total_decoded = Vec::new();
        for result in decoded_blocks {
            let (items, _) = result?;
            total_decoded.push(items);
        }
        
        // Apply all items to window
        for items in &total_decoded {
            self.apply_decoded(items)?;
        }
        
        // Get output
        Ok(self.get_output(0, output_size.min(self.window_pos)))
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

    #[cfg(feature = "parallel")]
    #[test]
    fn test_block_tables_clone() {
        let tables = super::BlockTables::new();
        let cloned = tables.clone();
        assert_eq!(cloned.use_align_bits, tables.use_align_bits);
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn test_scan_blocks_empty() {
        let mut decoder = Rar5BlockDecoder::new(20);
        let result = decoder.scan_blocks(&[], 10);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
