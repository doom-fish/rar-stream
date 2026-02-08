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

#[cfg(feature = "parallel")]
use std::sync::Arc;

/// Cold error for invalid back-references — keeps hot path code small.
#[cold]
#[inline(never)]
fn backref_error() -> DecompressError {
    DecompressError::Io(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Invalid back-reference: offset or length out of bounds",
    ))
}

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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
    Filter {
        filter_type: u8,
        block_start: usize,
        block_length: usize,
        channels: u8,
    },
}

/// Decoded command for split-buffer parallel decode.
/// Literals are stored in a separate byte buffer; this enum only contains
/// non-literal commands + markers for literal runs. Without the Literal variant,
/// the enum is only 12 bytes (vs 24 for DecodedItem).
#[cfg(feature = "parallel")]
#[derive(Debug, Clone, Copy)]
pub enum DecodeCmd {
    /// Run of literal bytes (read `count` bytes from literal buffer)
    LiteralRun { count: u32 },
    /// Match with explicit offset
    Match { length: u32, offset: u32 },
    /// Repeat with recent offset index (0-3)
    Rep { length: u32, rep_idx: u8 },
    /// Full repeat (reuse last length and offset[0])
    FullRep,
    /// Filter command
    Filter {
        filter_type: u8,
        block_start: u32,
        block_length: u32,
        channels: u8,
    },
}

/// Output of split-buffer decode: literals in a flat byte buffer,
/// commands (matches, reps, filters) in a separate Vec.
#[cfg(feature = "parallel")]
pub struct SplitDecoded {
    pub literals: Vec<u8>,
    pub commands: Vec<DecodeCmd>,
    pub output_size: usize,
}

/// Huffman decode table with quick lookup.
#[derive(Clone)]
pub struct HuffTable {
    /// Code lengths for each symbol
    code_lengths: Vec<u8>,
    /// Quick lookup table: packed as (symbol << 4) | length in u16.
    /// Symbol max 306 (12 bits), length max 15 (4 bits).
    quick_table: Vec<u16>,
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
        // Zero remaining lengths
        if num_symbols < self.num_symbols {
            // SAFETY: filling a u8 slice with zeros
            unsafe {
                std::ptr::write_bytes(
                    self.code_lengths.as_mut_ptr().add(num_symbols),
                    0,
                    self.num_symbols - num_symbols,
                );
            }
        }

        // Find max length
        self.max_length = 0;
        for &len in &self.code_lengths[..num_symbols] {
            if len > self.max_length {
                self.max_length = len;
            }
        }

        if self.max_length == 0 {
            // SAFETY: filling a u16 slice with zeros
            unsafe {
                std::ptr::write_bytes(self.quick_table.as_mut_ptr(), 0, self.quick_table.len());
            }
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
            decode_pos[0] as u16,
            decode_pos[1] as u16,
            decode_pos[2] as u16,
            decode_pos[3] as u16,
            decode_pos[4] as u16,
            decode_pos[5] as u16,
            decode_pos[6] as u16,
            decode_pos[7] as u16,
            decode_pos[8] as u16,
            decode_pos[9] as u16,
            decode_pos[10] as u16,
            decode_pos[11] as u16,
            decode_pos[12] as u16,
            decode_pos[13] as u16,
            decode_pos[14] as u16,
            decode_pos[15] as u16,
        ];

        // Build symbols array (unrar: DecodeNum)
        // SAFETY: filling a u16 slice with zeros via memset
        unsafe {
            std::ptr::write_bytes(self.symbols.as_mut_ptr(), 0, self.symbols.len());
        }
        let mut copy_pos = decode_pos;
        for (symbol, &len) in self.code_lengths[..num_symbols].iter().enumerate() {
            if len > 0 && len < 16 {
                let pos = copy_pos[len as usize] as usize;
                if pos < self.symbols.len() {
                    // SAFETY: pos bounds-checked above
                    unsafe {
                        *self.symbols.get_unchecked_mut(pos) = symbol as u16;
                    }
                }
                copy_pos[len as usize] += 1;
            }
        }

        // Build quick table for fast decode
        // SAFETY: filling a u16 slice with zeros via memset
        unsafe {
            std::ptr::write_bytes(self.quick_table.as_mut_ptr(), 0, self.quick_table.len());
        }
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
                    // SAFETY: pos bounds-checked, code < quick_size = quick_table.len()
                    unsafe {
                        let symbol = *self.symbols.get_unchecked(pos as usize);
                        *self.quick_table.get_unchecked_mut(code) =
                            ((symbol as u16) << 4) | (cur_bit_length as u16);
                    }
                }
            }
        }

        true
    }

    /// Decode a symbol using the bit decoder.
    /// Based on unrar's DecodeNumber.
    #[inline(always)]
    pub fn decode(&self, bits: &mut BitDecoder) -> u16 {
        // Get 16 bits for lookup (single 4-byte read + bswap)
        let bit_field = bits.getbits();

        // Quick decode path
        if bit_field < self.decode_len[self.quick_bits] {
            let code = (bit_field >> (16 - self.quick_bits)) as usize;
            // SAFETY: code is derived from bit_field >> (16 - quick_bits),
            // which is bounded by quick_bits (10), so code < 1024 = quick_table.len()
            let entry = unsafe { *self.quick_table.get_unchecked(code) };
            if entry != 0 {
                let len = (entry & 0xF) as usize;
                let symbol = (entry >> 4) as u16;
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

        // SAFETY: pos is bounded by symbol table size
        if (pos as usize) >= self.symbols.len() {
            return 0;
        }

        unsafe { *self.symbols.get_unchecked(pos as usize) }
    }
}

/// Block types in RAR5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
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
    /// Byte position where actual data starts (after tables if present)
    pub data_start: usize,
    /// Bit position within data_start byte
    pub data_start_bit: usize,
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

/// Configuration for parallel decompression.
#[cfg(feature = "parallel")]
pub struct ParallelConfig {
    /// Number of threads (0 = auto-detect)
    pub num_threads: usize,
    /// Blocks per batch (0 = auto: num_threads * 2)
    pub blocks_per_batch: usize,
    /// Large block threshold in bytes (blocks larger use single-thread)
    pub large_block_size: usize,
    /// Max decoded items per block
    pub max_items_per_block: usize,
}

#[cfg(feature = "parallel")]
impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            num_threads: 0,              // auto-detect
            blocks_per_batch: 24,        // Optimal batch size for parallelism
            large_block_size: 0x20000,   // 128KB like unrar
            max_items_per_block: 0x4100, // ~16K items like unrar
        }
    }
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
    recent_offsets: [u32; NUM_REPS],
    /// Last used length
    last_length: usize,
    /// Dictionary/window buffer (for backreferences)
    window: Vec<u8>,
    /// Window mask (size - 1)
    window_mask: usize,
    /// Current position in window
    window_pos: usize,
    /// Dictionary size
    #[allow(dead_code)]
    dict_size: usize,
    /// Output buffer (accumulates all output)
    output: Vec<u8>,
    /// Output start offset mapped into window (for direct output writes)
    output_start: usize,
    /// Whether output buffer is mapped to window (direct write mode)
    output_mapped: bool,
    /// Tables have been initialized
    tables_valid: bool,
    /// Use alignment bits
    use_align_bits: bool,
    /// Is last block
    is_last_block: bool,
    /// Reusable buffer for read_tables (avoids per-block allocation)
    table_lens: Vec<u8>,
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
            output: Vec::new(),
            output_start: 0,
            output_mapped: false,
            tables_valid: false,
            use_align_bits: false,
            is_last_block: false,
            table_lens: vec![
                0u8;
                MAIN_TABLE_SIZE + DIST_TABLE_SIZE + ALIGN_TABLE_SIZE + LEN_TABLE_SIZE
            ],
        }
    }

    /// Reset decoder state.
    pub fn reset(&mut self) {
        self.recent_offsets = [0; NUM_REPS];
        self.last_length = 0;
        self.window_pos = 0;
        self.output.clear();
        self.output_start = 0;
        self.output_mapped = false;
        self.tables_valid = false;
        self.use_align_bits = false;
        self.is_last_block = false;
    }

    /// Write a byte to the window and output.
    #[inline(always)]
    fn write_byte(&mut self, byte: u8) {
        // Write to window (for backreferences)
        // SAFETY: window_pos & window_mask is always < window.len()
        unsafe {
            *self
                .window
                .get_unchecked_mut(self.window_pos & self.window_mask) = byte;
        }

        // Direct output write when output is mapped to window
        if self.output_mapped {
            let idx = self.window_pos - self.output_start;
            unsafe {
                *self.output.get_unchecked_mut(idx) = byte;
            }
        } else {
            self.output.push(byte);
        }

        self.window_pos = self.window_pos.wrapping_add(1);
    }

    /// Copy bytes from earlier position in window.
    #[inline]
    fn copy_bytes(&mut self, offset: usize, length: usize) {
        let src_start = self.window_pos.wrapping_sub(offset);

        // Overlap case is when offset < length (run-length encoding pattern)
        if offset < length && offset > 1 {
            // Doubling copy: seed window, double in window, single copy to output
            let window_ptr = self.window.as_mut_ptr();

            // Seed: copy first `offset` bytes into window
            for i in 0..offset {
                let src_pos = src_start.wrapping_add(i) & self.window_mask;
                let dst_pos = self.window_pos.wrapping_add(i) & self.window_mask;
                unsafe {
                    *window_ptr.add(dst_pos) = *self.window.get_unchecked(src_pos);
                }
            }

            // Doubling in window (only works if dest doesn't wrap)
            let dst_mask_start = self.window_pos & self.window_mask;
            if dst_mask_start + length <= self.window.len() {
                let win_ptr = unsafe { window_ptr.add(dst_mask_start) };
                let mut copied = offset;
                while copied < length {
                    let chunk = copied.min(length - copied);
                    unsafe {
                        core::ptr::copy_nonoverlapping(win_ptr, win_ptr.add(copied), chunk);
                    }
                    copied += chunk;
                }
                // Single copy from window to output
                self.output.reserve(length);
                unsafe {
                    let src = win_ptr.cast_const();
                    let out_len = self.output.len();
                    core::ptr::copy_nonoverlapping(
                        src,
                        self.output.as_mut_ptr().add(out_len),
                        length,
                    );
                    self.output.set_len(out_len + length);
                }
            } else {
                // Wrapping: fall back to byte-by-byte for remaining
                let mut i = offset;
                while i < length {
                    let src_pos = self.window_pos.wrapping_add(i - offset) & self.window_mask;
                    let dst_pos = self.window_pos.wrapping_add(i) & self.window_mask;
                    unsafe {
                        *window_ptr.add(dst_pos) = *self.window.get_unchecked(src_pos);
                    }
                    i += 1;
                }
                // Copy window to output
                let out_start = self.output.len();
                self.output.reserve(length);
                unsafe { self.output.set_len(out_start + length) };
                for i in 0..length {
                    let pos = self.window_pos.wrapping_add(i) & self.window_mask;
                    unsafe {
                        *self.output.get_unchecked_mut(out_start + i) =
                            *self.window.get_unchecked(pos);
                    }
                }
            }
            self.window_pos = self.window_pos.wrapping_add(length);
        } else if offset < length {
            // offset == 1: RLE case — fill with single byte
            self.output.reserve(length);
            let byte = unsafe { *self.window.get_unchecked(src_start & self.window_mask) };

            // Fill window
            let window_ptr = self.window.as_mut_ptr();
            for i in 0..length {
                unsafe {
                    *window_ptr.add(self.window_pos.wrapping_add(i) & self.window_mask) = byte;
                }
            }

            // Fill output with memset-style bulk write
            let out_len = self.output.len();
            unsafe {
                core::ptr::write_bytes(self.output.as_mut_ptr().add(out_len), byte, length);
                self.output.set_len(out_len + length);
            }
            self.window_pos = self.window_pos.wrapping_add(length);
        } else {
            // No overlap - can copy efficiently
            let src_mask_start = src_start & self.window_mask;
            let dst_mask_start = self.window_pos & self.window_mask;
            let src_mask_end = (src_start.wrapping_add(length).wrapping_sub(1)) & self.window_mask;
            let dst_mask_end =
                (self.window_pos.wrapping_add(length).wrapping_sub(1)) & self.window_mask;

            // Fast path: no wraparound in either src or dst
            if src_mask_end >= src_mask_start && dst_mask_end >= dst_mask_start {
                // Single contiguous copy within window
                // Check for actual memory overlap (can happen with wrapping window_pos)
                let src_end = src_mask_start + length;
                let dst_end = dst_mask_start + length;
                let overlaps = src_mask_start < dst_end && dst_mask_start < src_end;
                unsafe {
                    let src = self.window.as_ptr().add(src_mask_start);
                    let dst = self.window.as_mut_ptr().add(dst_mask_start);
                    if overlaps {
                        std::ptr::copy(src, dst, length);
                    } else {
                        std::ptr::copy_nonoverlapping(src, dst, length);
                    }
                    // Extend output from window destination
                    self.output
                        .extend_from_slice(std::slice::from_raw_parts(dst.cast_const(), length));
                }
            } else {
                // Slow path: wraparound - copy byte by byte
                let output_start = self.output.len();
                self.output.reserve(length);
                // SAFETY: reserve() ensures capacity. All positions are written
                // by the byte-by-byte copy loop below.
                unsafe {
                    self.output.set_len(output_start + length);
                }

                for i in 0..length {
                    let src_pos = src_start.wrapping_add(i) & self.window_mask;
                    let dst_pos = self.window_pos.wrapping_add(i) & self.window_mask;
                    let byte = unsafe { *self.window.get_unchecked(src_pos) };
                    unsafe {
                        *self.window.get_unchecked_mut(dst_pos) = byte;
                        *self.output.get_unchecked_mut(output_start + i) = byte;
                    }
                }
            }
            self.window_pos = self.window_pos.wrapping_add(length);
        }
    }

    /// Get output from output buffer.
    #[allow(dead_code)]
    pub fn get_output(&self, start: usize, length: usize) -> Vec<u8> {
        let end = (start + length).min(self.output.len());
        self.output[start..end].to_vec()
    }

    /// Take ownership of output buffer (more efficient than get_output for full data).
    pub fn take_output(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.output)
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
        self.table_lens[..table_size].fill(0);
        let lens = &mut self.table_lens;

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

        // Pre-allocate output buffer if needed
        if self.output.capacity() < start_pos + output_size {
            self.output.reserve(output_size);
        }

        // Map output to window to avoid duplicate writes
        self.output_start = start_pos;
        if self.output.len() < start_pos + output_size {
            self.output
                .reserve(start_pos + output_size - self.output.len());
            // SAFETY: reserve() ensures capacity >= start_pos + output_size.
            // All positions in [start_pos..start_pos+output_size) are written by the
            // decode loop below (write_byte / copy_match). If the loop exits early,
            // set_len(pos) at the end truncates to actual written bytes.
            unsafe {
                self.output.set_len(start_pos + output_size);
            }
        }
        self.output_mapped = true;

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
                    let length = self.last_length;
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
                self.last_length = length;
                self.copy_bytes(offset, length);
            } else {
                // New offset with length (sym >= 262)
                let len_slot = sym - 262;
                let mut length = self.slot_to_length(len_slot as u32, bits)?;
                let offset = self.decode_offset(bits)?;

                // Adjust length based on distance (unrar: Distance>0x100, 0x2000, 0x40000)
                length += (offset > 0x100) as usize
                    + (offset > 0x2000) as usize
                    + (offset > 0x40000) as usize;

                // Update recent offsets
                for j in (1..NUM_REPS).rev() {
                    self.recent_offsets[j] = self.recent_offsets[j - 1];
                }
                self.recent_offsets[0] = offset as u32;
                self.last_length = length;

                self.copy_bytes(offset, length);
            }
        }

        self.output_mapped = false;
        Ok(filters)
    }

    /// Decode compressed data directly to a linear output buffer.
    /// Optimized for non-solid files: backreferences read from the output
    /// buffer itself, eliminating the separate window buffer overhead.
    /// Uses a 64-bit bit buffer for the inner decode loop.
    pub fn decode_block_direct(
        &mut self,
        bits: &mut BitDecoder,
        output_size: usize,
    ) -> Result<Vec<super::filter::UnpackFilter>, DecompressError> {
        let mut filters = Vec::new();

        // Pre-allocate output buffer without zeroing.
        // Extra 16 bytes allow 8-byte wide writes in copy_match without
        // bounds overflow (the extra bytes are capacity, never read as output).
        // SAFETY: all positions [0..output_size) are written before being read
        // (either directly as literals or as destinations of copy_match).
        let mut output: Vec<u8> = Vec::with_capacity(output_size + 16);
        unsafe {
            output.set_len(output_size);
        }
        let out_ptr = output.as_mut_ptr();
        let mut pos = 0usize;

        // Outer loop: iterate over blocks
        while pos < output_size {
            if bits.is_eof() || self.is_last_block {
                break;
            }

            let new_tables = self.read_block_header(bits)?;
            if new_tables {
                self.read_tables(bits)?;
            } else if !self.tables_valid {
                return Err(DecompressError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Continue block but no previous tables",
                )));
            }

            let block_end = bits.block_end_total_bits();
            let mut fb = FastBits::from_decoder(bits);

            // Inner loop with labeled break for tight literal sub-loop.
            'block: while pos < output_size && !fb.is_block_over(block_end) {
                let mut sym = fb.decode(&self.main_table) as usize;

                // Tight literal loop with safe burst optimization.
                // Compute a safe iteration count where neither output bounds
                // nor block-end can be reached, eliminating two checks per symbol.
                if sym < 256 {
                    unsafe {
                        *out_ptr.add(pos) = sym as u8;
                    }
                    pos += 1;

                    // Each Huffman symbol is at most 15 bits.
                    let bits_left = block_end.saturating_sub(fb.consumed_bits);
                    let safe = (bits_left / 15).min(output_size - pos);
                    let safe_end = pos + safe;

                    while pos < safe_end {
                        sym = fb.decode(&self.main_table) as usize;
                        if sym >= 256 {
                            break;
                        }
                        unsafe {
                            *out_ptr.add(pos) = sym as u8;
                        }
                        pos += 1;
                    }

                    if sym < 256 {
                        continue 'block;
                    }
                    // sym >= 256: fall through to non-literal handling
                }

                // Non-literal symbol handling
                if sym >= 262 {
                    let len_slot = sym - 262;
                    let mut length = self.fast_slot_to_length(len_slot as u32, &mut fb);
                    let offset = self.fast_decode_offset(&mut fb);
                    // Branchless length bonus based on offset magnitude
                    length += (offset > 0x100) as usize
                        + (offset > 0x2000) as usize
                        + (offset > 0x40000) as usize;
                    self.recent_offsets[3] = self.recent_offsets[2];
                    self.recent_offsets[2] = self.recent_offsets[1];
                    self.recent_offsets[1] = self.recent_offsets[0];
                    self.recent_offsets[0] = offset as u32;
                    self.last_length = length;
                    if offset > pos || pos + length > output_size {
                        return Err(backref_error());
                    }
                    copy_match(out_ptr, pos, offset, length);
                    pos += length;
                } else if sym >= 258 {
                    let rep_idx = sym - 258;
                    let offset = self.recent_offsets[rep_idx] as usize;
                    if offset == 0 {
                        return Err(DecompressError::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid zero offset",
                        )));
                    }
                    let length = self.fast_decode_length(&mut fb);
                    if rep_idx > 0 {
                        let off = self.recent_offsets[rep_idx];
                        if rep_idx >= 3 {
                            self.recent_offsets[3] = self.recent_offsets[2];
                        }
                        if rep_idx >= 2 {
                            self.recent_offsets[2] = self.recent_offsets[1];
                        }
                        self.recent_offsets[1] = self.recent_offsets[0];
                        self.recent_offsets[0] = off;
                    }
                    self.last_length = length;
                    if offset > pos || pos + length > output_size {
                        return Err(backref_error());
                    }
                    copy_match(out_ptr, pos, offset, length);
                    pos += length;
                } else if sym == 257 {
                    if self.last_length != 0 && self.recent_offsets[0] != 0 {
                        let length = self.last_length;
                        let offset = self.recent_offsets[0] as usize;
                        if offset > pos || pos + length > output_size {
                            return Err(backref_error());
                        }
                        copy_match(out_ptr, pos, offset, length);
                        pos += length;
                    }
                } else {
                    fb.sync_to(bits);
                    self.window_pos = pos;
                    if let Some(filter) = self.read_filter(bits)? {
                        filters.push(filter);
                    }
                    fb.reload_from(bits);
                }
            }

            // Sync back to BitDecoder for next block header
            fb.sync_to(bits);
        }

        self.window_pos = pos;
        self.output = output;
        if pos < output_size {
            // SAFETY: pos tracks actual bytes written; truncate to real size
            unsafe {
                self.output.set_len(pos);
            }
        }
        Ok(filters)
    }

    /// Decode match length using FastBits.
    #[inline(always)]
    fn fast_decode_length(&self, fb: &mut FastBits) -> usize {
        let sym = fb.decode(&self.len_table) as u32;
        self.fast_slot_to_length(sym, fb)
    }

    /// Convert length slot to actual length using FastBits.
    #[inline(always)]
    fn fast_slot_to_length(&self, slot: u32, fb: &mut FastBits) -> usize {
        if slot < 8 {
            (slot + 2) as usize
        } else {
            let extra_bits = ((slot - 4) / 4) as u32;
            let base = ((4u32 + (slot & 3)) << extra_bits).wrapping_add(2);
            let extra = fb.read(extra_bits);
            (base.wrapping_add(extra)) as usize
        }
    }

    /// Decode match offset using FastBits.
    #[inline(always)]
    fn fast_decode_offset(&self, fb: &mut FastBits) -> usize {
        let slot = fb.decode(&self.dist_table) as u32;
        if slot < 4 {
            (slot + 1) as usize
        } else {
            let num_bits = (slot / 2 - 1) as u32;
            let base = (2 | (slot & 1)) << num_bits;
            if num_bits < NUM_ALIGN_BITS as u32 {
                let extra = fb.read(num_bits);
                (base.wrapping_add(extra).wrapping_add(1)) as usize
            } else {
                let high = if num_bits > NUM_ALIGN_BITS as u32 {
                    fb.read(num_bits - NUM_ALIGN_BITS as u32)
                } else {
                    0
                };
                let low = if self.use_align_bits {
                    fb.decode(&self.align_table) as u32
                } else {
                    fb.read(NUM_ALIGN_BITS as u32)
                };
                (base
                    .wrapping_add(high << NUM_ALIGN_BITS)
                    .wrapping_add(low)
                    .wrapping_add(1)) as usize
            }
        }
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

        let flush_literals = |items: &mut Vec<DecodedItem>, buf: &mut [u8; 8], count: &mut u8| {
            if *count > 0 {
                items.push(DecodedItem::Literal {
                    bytes: *buf,
                    len: *count,
                });
                *count = 0;
            }
        };

        let block_end = bits.block_end_total_bits();
        let mut fb = FastBits::from_decoder(bits);

        while items.len() < max_symbols && !fb.is_block_over(block_end) {
            let sym = fb.decode(&self.main_table) as usize;

            if sym < 256 {
                literal_buf[literal_count as usize] = sym as u8;
                literal_count += 1;
                output_size += 1;
                if literal_count == 8 {
                    flush_literals(&mut items, &mut literal_buf, &mut literal_count);
                }
            } else {
                flush_literals(&mut items, &mut literal_buf, &mut literal_count);

                if sym == 256 {
                    // Filter — sync to BitDecoder for complex reads
                    fb.sync_to(bits);
                    let block_start = self.read_filter_data(bits) as usize;
                    let block_length = self.read_filter_data(bits) as usize;
                    let v = bits.get_value_high32();
                    let filter_type = bits.read_bits_big(3, v) as u8;
                    let channels = if filter_type == 0 {
                        let v = bits.get_value_high32();
                        (bits.read_bits_big(5, v) + 1) as u8
                    } else {
                        0
                    };
                    items.push(DecodedItem::Filter {
                        filter_type,
                        block_start,
                        block_length,
                        channels,
                    });
                    fb.reload_from(bits);
                } else if sym == 257 {
                    if self.last_length != 0 {
                        output_size += self.last_length;
                    }
                    items.push(DecodedItem::FullRep);
                } else if sym < 262 {
                    let rep_idx = (sym - 258) as u8;
                    let length_slot = fb.decode(&self.len_table) as u32;
                    let length = Self::slot_to_length_fast(length_slot, &mut fb);
                    output_size += length as usize;
                    self.last_length = length as usize;
                    items.push(DecodedItem::Rep { length, rep_idx });
                } else {
                    let len_slot = sym - 262;
                    let mut length = Self::slot_to_length_fast(len_slot as u32, &mut fb);
                    let offset = self.fast_decode_offset(&mut fb);
                    length += (offset > 0x100) as u32
                        + (offset > 0x2000) as u32
                        + (offset > 0x40000) as u32;
                    output_size += length as usize;
                    self.last_length = length as usize;
                    items.push(DecodedItem::Match { length, offset });
                }
            }
        }

        flush_literals(&mut items, &mut literal_buf, &mut literal_count);
        fb.sync_to(bits);

        Ok((items, output_size, self.is_last_block))
    }

    /// Apply decoded items to the sliding window (phase 2 of two-phase decoding).
    /// Must be called sequentially on the main thread.
    #[cfg(feature = "parallel")]
    pub fn apply_decoded(
        &mut self,
        items: &[DecodedItem],
    ) -> Result<Vec<super::filter::UnpackFilter>, DecompressError> {
        use super::filter::{FilterType, UnpackFilter};
        let mut filters = Vec::new();

        // Estimate output size and reserve (rough estimate: average 4 bytes per item)
        self.output.reserve(items.len() * 4);

        for item in items {
            match item {
                DecodedItem::Literal { bytes, len } => {
                    // Write multiple bytes at once
                    let len = *len as usize;
                    let bytes_slice = &bytes[..len];

                    // Check if we need to wrap around the window
                    let start_mask_pos = self.window_pos & self.window_mask;
                    let end_mask_pos = (self.window_pos + len - 1) & self.window_mask;

                    // Fast path: no window wraparound needed
                    if end_mask_pos >= start_mask_pos {
                        // Single contiguous copy to window
                        unsafe {
                            let dst = self.window.as_mut_ptr().add(start_mask_pos);
                            std::ptr::copy_nonoverlapping(bytes.as_ptr(), dst, len);
                        }
                    } else {
                        // Slow path: wraps around window boundary
                        for i in 0..len {
                            let byte = bytes[i];
                            unsafe {
                                *self
                                    .window
                                    .get_unchecked_mut((self.window_pos + i) & self.window_mask) =
                                    byte;
                            }
                        }
                    }

                    // Write to output buffer (mapped or append)
                    if self.output_mapped {
                        let output_idx = self.window_pos - self.output_start;
                        unsafe {
                            let dst = self.output.as_mut_ptr().add(output_idx);
                            std::ptr::copy_nonoverlapping(bytes.as_ptr(), dst, len);
                        }
                    } else {
                        self.output.extend_from_slice(bytes_slice);
                    }
                    self.window_pos += len;
                }
                DecodedItem::Match { length, offset } => {
                    // Update recent offsets (unrolled)
                    self.recent_offsets[3] = self.recent_offsets[2];
                    self.recent_offsets[2] = self.recent_offsets[1];
                    self.recent_offsets[1] = self.recent_offsets[0];
                    self.recent_offsets[0] = *offset as u32;
                    self.last_length = *length as usize;
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
                    // Rotate recent offsets (unrolled by index)
                    match rep_idx {
                        0 => {} // No rotation needed
                        1 => {
                            let off = self.recent_offsets[1];
                            self.recent_offsets[1] = self.recent_offsets[0];
                            self.recent_offsets[0] = off;
                        }
                        2 => {
                            let off = self.recent_offsets[2];
                            self.recent_offsets[2] = self.recent_offsets[1];
                            self.recent_offsets[1] = self.recent_offsets[0];
                            self.recent_offsets[0] = off;
                        }
                        _ => {
                            let off = self.recent_offsets[3];
                            self.recent_offsets[3] = self.recent_offsets[2];
                            self.recent_offsets[2] = self.recent_offsets[1];
                            self.recent_offsets[1] = self.recent_offsets[0];
                            self.recent_offsets[0] = off;
                        }
                    }
                    self.last_length = *length as usize;
                    self.copy_bytes(offset, *length as usize);
                }
                DecodedItem::FullRep => {
                    if self.last_length != 0 && self.recent_offsets[0] != 0 {
                        let length = self.last_length;
                        let offset = self.recent_offsets[0] as usize;
                        self.copy_bytes(offset, length);
                    }
                }
                DecodedItem::Filter {
                    filter_type,
                    block_start,
                    block_length,
                    channels,
                } => {
                    if let Some(ft) = FilterType::from_bits(*filter_type) {
                        let actual_start = *block_start + self.window_pos;
                        filters.push(UnpackFilter::new(
                            ft,
                            actual_start,
                            *block_length,
                            *channels,
                        ));
                    }
                }
            }
        }

        Ok(filters)
    }

    /// Apply decoded items directly to a pre-allocated output buffer.
    /// Uses the output buffer itself for backreferences (no sliding window).
    /// This is much faster than `apply_decoded` as it avoids double-writes.
    #[cfg(feature = "parallel")]
    pub fn apply_decoded_direct(
        &mut self,
        items: &[DecodedItem],
        out_ptr: *mut u8,
        pos: &mut usize,
    ) -> Result<Vec<super::filter::UnpackFilter>, DecompressError> {
        use super::filter::{FilterType, UnpackFilter};
        let mut filters = Vec::new();

        for item in items {
            match item {
                DecodedItem::Literal { bytes, len } => {
                    let len = *len as usize;
                    unsafe {
                        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_ptr.add(*pos), len);
                    }
                    *pos += len;
                }
                DecodedItem::Match { length, offset } => {
                    self.recent_offsets[3] = self.recent_offsets[2];
                    self.recent_offsets[2] = self.recent_offsets[1];
                    self.recent_offsets[1] = self.recent_offsets[0];
                    self.recent_offsets[0] = *offset as u32;
                    self.last_length = *length as usize;
                    copy_match(out_ptr, *pos, *offset, *length as usize);
                    *pos += *length as usize;
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
                    match rep_idx {
                        0 => {}
                        1 => {
                            let off = self.recent_offsets[1];
                            self.recent_offsets[1] = self.recent_offsets[0];
                            self.recent_offsets[0] = off;
                        }
                        2 => {
                            let off = self.recent_offsets[2];
                            self.recent_offsets[2] = self.recent_offsets[1];
                            self.recent_offsets[1] = self.recent_offsets[0];
                            self.recent_offsets[0] = off;
                        }
                        _ => {
                            let off = self.recent_offsets[3];
                            self.recent_offsets[3] = self.recent_offsets[2];
                            self.recent_offsets[2] = self.recent_offsets[1];
                            self.recent_offsets[1] = self.recent_offsets[0];
                            self.recent_offsets[0] = off;
                        }
                    }
                    self.last_length = *length as usize;
                    copy_match(out_ptr, *pos, offset, *length as usize);
                    *pos += *length as usize;
                }
                DecodedItem::FullRep => {
                    if self.last_length != 0 && self.recent_offsets[0] != 0 {
                        let length = self.last_length;
                        let offset = self.recent_offsets[0] as usize;
                        copy_match(out_ptr, *pos, offset, length);
                        *pos += length;
                    }
                }
                DecodedItem::Filter {
                    filter_type,
                    block_start,
                    block_length,
                    channels,
                } => {
                    if let Some(ft) = FilterType::from_bits(*filter_type) {
                        let actual_start = *block_start + *pos;
                        filters.push(UnpackFilter::new(
                            ft,
                            actual_start,
                            *block_length,
                            *channels,
                        ));
                    }
                }
            }
        }

        Ok(filters)
    }

    /// Apply split-buffer decoded output directly to a pre-allocated output buffer.
    /// Reads literal bytes from `split.literals` and commands from `split.commands`.
    #[cfg(feature = "parallel")]
    pub fn apply_split_direct(
        &mut self,
        split: &SplitDecoded,
        out_ptr: *mut u8,
        pos: &mut usize,
    ) -> Result<Vec<super::filter::UnpackFilter>, DecompressError> {
        use super::filter::{FilterType, UnpackFilter};
        let mut filters = Vec::new();
        let mut lit_pos = 0usize;

        for cmd in &split.commands {
            match cmd {
                DecodeCmd::LiteralRun { count } => {
                    let count = *count as usize;
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            split.literals.as_ptr().add(lit_pos),
                            out_ptr.add(*pos),
                            count,
                        );
                    }
                    lit_pos += count;
                    *pos += count;
                }
                DecodeCmd::Match { length, offset } => {
                    let off_usize = *offset as usize;
                    self.recent_offsets[3] = self.recent_offsets[2];
                    self.recent_offsets[2] = self.recent_offsets[1];
                    self.recent_offsets[1] = self.recent_offsets[0];
                    self.recent_offsets[0] = *offset;
                    self.last_length = *length as usize;
                    copy_match(out_ptr, *pos, off_usize, *length as usize);
                    *pos += *length as usize;
                }
                DecodeCmd::Rep { length, rep_idx } => {
                    let rep_idx = *rep_idx as usize;
                    let offset = self.recent_offsets[rep_idx] as usize;
                    if offset == 0 {
                        return Err(DecompressError::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid zero offset in Rep",
                        )));
                    }
                    match rep_idx {
                        0 => {}
                        1 => {
                            let off = self.recent_offsets[1];
                            self.recent_offsets[1] = self.recent_offsets[0];
                            self.recent_offsets[0] = off;
                        }
                        2 => {
                            let off = self.recent_offsets[2];
                            self.recent_offsets[2] = self.recent_offsets[1];
                            self.recent_offsets[1] = self.recent_offsets[0];
                            self.recent_offsets[0] = off;
                        }
                        _ => {
                            let off = self.recent_offsets[3];
                            self.recent_offsets[3] = self.recent_offsets[2];
                            self.recent_offsets[2] = self.recent_offsets[1];
                            self.recent_offsets[1] = self.recent_offsets[0];
                            self.recent_offsets[0] = off;
                        }
                    }
                    self.last_length = *length as usize;
                    copy_match(out_ptr, *pos, offset, *length as usize);
                    *pos += *length as usize;
                }
                DecodeCmd::FullRep => {
                    if self.last_length != 0 && self.recent_offsets[0] != 0 {
                        let length = self.last_length;
                        let offset = self.recent_offsets[0] as usize;
                        copy_match(out_ptr, *pos, offset, length);
                        *pos += length;
                    }
                }
                DecodeCmd::Filter {
                    filter_type,
                    block_start,
                    block_length,
                    channels,
                } => {
                    if let Some(ft) = FilterType::from_bits(*filter_type) {
                        let actual_start = *block_start as usize + *pos;
                        filters.push(UnpackFilter::new(
                            ft,
                            actual_start,
                            *block_length as usize,
                            *channels,
                        ));
                    }
                }
            }
        }

        Ok(filters)
    }

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

        // Calculate actual block start: block_start is relative to current output position
        // Since we use a linear output buffer (not wrapped window), don't apply modulo
        let actual_start = block_start + self.window_pos;

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
            let base = ((4u32 + (slot & 3)) << extra_bits).wrapping_add(2);
            let v = bits.get_value_high32();
            let extra = bits.read_bits_big(extra_bits, v);
            Ok((base.wrapping_add(extra)) as usize)
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
                Ok(base.wrapping_add(extra).wrapping_add(1) as usize)
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

                let offset = base
                    .wrapping_add(high << NUM_ALIGN_BITS)
                    .wrapping_add(low)
                    .wrapping_add(1);
                Ok(offset as usize)
            }
        }
    }
}

/// Copy bytes within the output buffer for a backreference match.
/// Handles overlap correctly (offset < length) for run-length patterns.
///
/// # Safety
/// Caller must ensure `buf[pos-offset..pos-offset+length]` and
/// `buf[pos..pos+length+8]` are within the allocated buffer
/// (the +8 allows wide writes into over-allocated headroom).
#[inline(always)]
fn copy_match(buf: *mut u8, pos: usize, offset: usize, length: usize) {
    let src = pos - offset;
    // SAFETY: caller guarantees all positions are within bounds
    // and buffer has 16 bytes of headroom past output_size.
    unsafe {
        if offset == 1 {
            // Very common RLE case: fill with repeated byte (memset)
            std::ptr::write_bytes(buf.add(pos), *buf.add(src), length);
        } else if offset >= length {
            // No overlap - single memcpy (vectorized by compiler/libc)
            std::ptr::copy_nonoverlapping(buf.add(src), buf.add(pos), length);
        } else if offset >= 8 && length <= 64 {
            // Short overlapping match with offset >= 8: copy 8 bytes at a time.
            // Safe because each 8-byte read finishes before reaching the write region.
            let src_p = buf.add(src);
            let dst_p = buf.add(pos);
            let mut i = 0;
            while i + 8 <= length {
                (dst_p.add(i) as *mut u64)
                    .write_unaligned((src_p.add(i) as *const u64).read_unaligned());
                i += 8;
            }
            if i < length {
                // Tail: wide write; extra bytes land in over-allocated headroom
                (dst_p.add(i) as *mut u64)
                    .write_unaligned((src_p.add(i) as *const u64).read_unaligned());
            }
        } else {
            // Long overlap or small offset (2-7): doubling trick.
            // Copy `offset` bytes, then double the filled region repeatedly.
            let dst = buf.add(pos);
            std::ptr::copy_nonoverlapping(buf.add(src), dst, offset);
            let mut copied = offset;
            while copied < length {
                let chunk = copied.min(length - copied);
                std::ptr::copy_nonoverlapping(dst, dst.add(copied), chunk);
                copied += chunk;
            }
        }
    }
}

/// Fast 64-bit bit buffer for the inner decode loop.
/// Keeps bits in a CPU register instead of reading from memory per symbol.
struct FastBits {
    buf: u64,
    n: u32,
    src: *const u8,
    pos: usize,
    /// Maximum byte position for safe 4-byte reads (buf_len - 4).
    src_safe_end: usize,
    /// Tracks consumed bits = pos*8 - n, updated incrementally.
    consumed_bits: usize,
}

impl FastBits {
    /// Initialize from a BitDecoder's current state.
    fn from_decoder(bits: &BitDecoder) -> Self {
        let byte_pos = bits.position();
        let bit_pos = bits.bit_pos();
        let mut fb = Self {
            buf: 0,
            n: 0,
            src: bits.buf_ptr(),
            pos: byte_pos,
            src_safe_end: bits.buf_len().saturating_sub(4),
            consumed_bits: 0,
        };
        fb.refill();
        if bit_pos > 0 {
            fb.buf <<= bit_pos;
            fb.n -= bit_pos as u32;
        }
        fb.consumed_bits = (fb.pos * 8).saturating_sub(fb.n as usize);
        fb
    }

    /// Sync state back to a BitDecoder.
    fn sync_to(&self, bits: &mut BitDecoder) {
        bits.set_position_with_bit(self.consumed_bits / 8, self.consumed_bits % 8);
    }

    /// Reload from a BitDecoder (after it was used for block headers/tables).
    fn reload_from(&mut self, bits: &BitDecoder) {
        self.pos = bits.position();
        self.buf = 0;
        self.n = 0;
        self.refill();
        let bp = bits.bit_pos();
        if bp > 0 {
            self.buf <<= bp;
            self.n -= bp as u32;
        }
        self.consumed_bits = (self.pos * 8).saturating_sub(self.n as usize);
    }

    /// Refill buffer with a 4-byte read when buffer is getting low.
    #[inline(always)]
    fn refill(&mut self) {
        if self.n <= 32 && self.pos <= self.src_safe_end {
            // SAFETY: pos <= src_safe_end = buf_len - 4, so reading
            // 4 bytes at pos is within the allocated buffer.
            let v =
                unsafe { u32::from_be((self.src.add(self.pos) as *const u32).read_unaligned()) };
            self.buf |= (v as u64) << (32 - self.n);
            self.n += 32;
            self.pos += 4;
        }
    }

    #[inline(always)]
    fn ensure(&mut self, need: u32) {
        if self.n < need {
            self.refill();
        }
    }

    #[inline(always)]
    fn skip(&mut self, bits: u32) {
        self.consumed_bits += bits as usize;
        if bits >= self.n {
            self.buf = 0;
            self.n = 0;
        } else {
            self.buf <<= bits;
            self.n -= bits;
        }
    }

    #[inline(always)]
    fn read(&mut self, num: u32) -> u32 {
        self.ensure(num);
        let v = (self.buf >> (64 - num)) as u32;
        self.consumed_bits += num as usize;
        self.buf <<= num;
        self.n -= num;
        v
    }

    #[inline(always)]
    fn is_block_over(&self, end_bits: usize) -> bool {
        self.consumed_bits >= end_bits
    }

    /// Decode a Huffman symbol using the quick table.
    #[inline(always)]
    fn decode(&mut self, table: &HuffTable) -> u16 {
        self.ensure(16);
        // Quick table lookup: entry is 0 for codes longer than quick_bits,
        // so we can skip the decode_len pre-check and just test the entry.
        let code = (self.buf >> (64 - table.quick_bits as u64)) as usize;
        let entry = unsafe { *table.quick_table.get_unchecked(code) };
        if entry != 0 {
            let len = (entry & 0xF) as u32;
            // Inline skip to avoid function call overhead on fast path
            self.consumed_bits += len as usize;
            self.buf <<= len;
            self.n -= len;
            return (entry >> 4) as u16;
        }

        let bit_field = (self.buf >> 48) as u32;
        self.decode_slow(bit_field, table)
    }

    /// Slow path for Huffman decoding — outlined to keep decode() small.
    #[cold]
    #[inline(never)]
    fn decode_slow(&mut self, bit_field: u32, table: &HuffTable) -> u16 {
        let mut bit_len = 15u32;
        for i in (table.quick_bits as u32 + 1)..15 {
            if bit_field < table.decode_len[i as usize] {
                bit_len = i;
                break;
            }
        }
        self.skip(bit_len);

        let dist = bit_field.wrapping_sub(table.decode_len[bit_len as usize - 1]);
        let dist_shifted = dist >> (16 - bit_len);
        let pos = (table.first_symbol[bit_len as usize] as u32) + dist_shifted;
        if (pos as usize) >= table.symbols.len() {
            return 0;
        }
        unsafe { *table.symbols.get_unchecked(pos as usize) }
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
                data_start: block_start, // Will be updated when tables are read
                data_start_bit: 0,
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
        // Must match read_tables exactly: use read_bits_9fix
        let mut level_lens = [0u8; LEVEL_TABLE_SIZE];
        let mut level_table = HuffTable::new(LEVEL_TABLE_SIZE, QUICK_BITS_LEVEL);

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

        level_table.build(&level_lens);

        // Calculate total symbols needed
        let total_symbols = MAIN_TABLE_SIZE + DIST_TABLE_SIZE + ALIGN_TABLE_SIZE + LEN_TABLE_SIZE;
        let mut lens = vec![0u8; total_symbols];

        // Decode all code lengths using level table
        // Must match read_tables exactly: use the same repeat code logic
        let mut idx = 0;
        while idx < total_symbols {
            let sym = level_table.decode(bits) as usize;

            if sym < 16 {
                lens[idx] = sym as u8;
                idx += 1;
            } else {
                // Repeat codes based on unrar5j logic:
                // num = ((sym - 16) & 1) * 4;
                // num += num + 3 + readBits9(num + 3);
                let num = ((sym - 16) & 1) * 4;
                let read_bits = num + 3;
                // count = 2*num + 3 + extra
                let mut count = 2 * num + 3 + bits.read_bits_9(read_bits) as usize;
                count += idx;
                if count > total_symbols {
                    count = total_symbols;
                }
                let v = if sym < 18 {
                    // sym 16-17: repeat previous
                    if idx > 0 {
                        lens[idx - 1]
                    } else {
                        0
                    }
                } else {
                    // sym 18-19: repeat zeros
                    0
                };
                while idx < count {
                    lens[idx] = v;
                    idx += 1;
                }
            }
        }

        // Build the four tables
        let mut offset = 0;
        tables
            .main_table
            .build(&lens[offset..offset + MAIN_TABLE_SIZE]);
        offset += MAIN_TABLE_SIZE;

        tables
            .dist_table
            .build(&lens[offset..offset + DIST_TABLE_SIZE]);
        offset += DIST_TABLE_SIZE;

        // Check if align table has non-default values
        tables.use_align_bits = false;
        for k in 0..ALIGN_TABLE_SIZE {
            if lens[offset + k] != NUM_ALIGN_BITS as u8 {
                tables
                    .align_table
                    .build(&lens[offset..offset + ALIGN_TABLE_SIZE]);
                tables.use_align_bits = true;
                break;
            }
        }
        offset += ALIGN_TABLE_SIZE;

        tables
            .len_table
            .build(&lens[offset..offset + LEN_TABLE_SIZE]);

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
        let mut items = Vec::with_capacity(8192.min(max_items));
        let mut output_size = 0usize;
        let mut literal_buf = [0u8; 8];
        let mut literal_count = 0u8;
        let mut last_length = 0u32;

        // Set block boundary
        let block_end_byte = header.block_start + header.block_size - 1;
        bits.set_block_end(block_end_byte, header.block_bit_size);

        let block_end = bits.block_end_total_bits();
        let mut fb = FastBits::from_decoder(bits);

        let flush_literals = |items: &mut Vec<DecodedItem>, buf: &mut [u8; 8], count: &mut u8| {
            if *count > 0 {
                items.push(DecodedItem::Literal {
                    bytes: *buf,
                    len: *count,
                });
                *count = 0;
            }
        };

        while items.len() < max_items && !fb.is_block_over(block_end) {
            let sym = fb.decode(&tables.main_table) as usize;

            if sym < 256 {
                literal_buf[literal_count as usize] = sym as u8;
                literal_count += 1;
                output_size += 1;
                if literal_count == 8 {
                    flush_literals(&mut items, &mut literal_buf, &mut literal_count);
                }
            } else {
                flush_literals(&mut items, &mut literal_buf, &mut literal_count);

                if sym == 256 {
                    // Filter command — sync back to BitDecoder for complex reads
                    fb.sync_to(bits);
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
                    let channels = if filter_type == 0 {
                        let v = bits.get_value_high32();
                        (bits.read_bits_big(5, v) + 1) as u8
                    } else {
                        0
                    };
                    items.push(DecodedItem::Filter {
                        filter_type,
                        block_start,
                        block_length,
                        channels,
                    });
                    fb.reload_from(bits);
                } else if sym == 257 {
                    if last_length != 0 {
                        output_size += last_length as usize;
                    }
                    items.push(DecodedItem::FullRep);
                } else if sym < 262 {
                    let rep_idx = (sym - 258) as u8;
                    let length_slot = fb.decode(&tables.len_table) as u32;
                    let length = Self::slot_to_length_fast(length_slot, &mut fb);
                    output_size += length as usize;
                    last_length = length;
                    items.push(DecodedItem::Rep { length, rep_idx });
                } else {
                    let len_slot = (sym - 262) as u32;
                    let mut length = Self::slot_to_length_fast(len_slot, &mut fb);
                    let dist_slot = fb.decode(&tables.dist_table) as u32;
                    let offset = Self::decode_offset_fast(dist_slot, tables, &mut fb);
                    length += (offset > 0x100) as u32
                        + (offset > 0x2000) as u32
                        + (offset > 0x40000) as u32;
                    output_size += length as usize;
                    last_length = length;
                    items.push(DecodedItem::Match { length, offset });
                }
            }
        }

        flush_literals(&mut items, &mut literal_buf, &mut literal_count);

        // Sync back to BitDecoder
        fb.sync_to(bits);

        Ok((items, output_size))
    }

    /// Decode a block into split buffers: literal bytes go into a flat Vec<u8>,
    /// non-literal commands go into a Vec<DecodeCmd>. This avoids the per-literal
    /// overhead of buffering into [u8;8] and pushing DecodedItem.
    #[cfg(feature = "parallel")]
    pub fn decode_block_split(
        bits: &mut BitDecoder,
        tables: &BlockTables,
        header: &BlockHeader,
        max_items: usize,
    ) -> Result<SplitDecoded, DecompressError> {
        // Pre-allocate literals with generous estimate to avoid reallocation
        // block_size is compressed size; output is typically larger
        let lit_cap = (header.block_size * 2).max(16384);
        let mut literals: Vec<u8> = Vec::with_capacity(lit_cap);
        let mut commands: Vec<DecodeCmd> = Vec::with_capacity(4096);
        let mut output_size = 0usize;
        let mut last_length = 0u32;
        let mut literal_run = 0u32;

        let block_end_byte = header.block_start + header.block_size - 1;
        bits.set_block_end(block_end_byte, header.block_bit_size);

        let block_end = bits.block_end_total_bits();
        let mut fb = FastBits::from_decoder(bits);
        let mut cmd_count = 0usize;
        let mut lit_len = 0usize;

        while cmd_count < max_items && !fb.is_block_over(block_end) {
            let sym = fb.decode(&tables.main_table) as usize;

            if sym < 256 {
                // Ensure capacity for literal write
                if lit_len >= literals.capacity() {
                    literals.reserve(lit_len + 8192);
                }
                // SAFETY: We just ensured capacity >= lit_len + 1
                unsafe {
                    *literals.as_mut_ptr().add(lit_len) = sym as u8;
                }
                lit_len += 1;
                literal_run += 1;
                output_size += 1;
            } else {
                // Flush pending literal run
                if literal_run > 0 {
                    commands.push(DecodeCmd::LiteralRun { count: literal_run });
                    literal_run = 0;
                    cmd_count += 1;
                }

                if sym == 256 {
                    fb.sync_to(bits);
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

                    let block_start = read_filter_data(bits);
                    let block_length = read_filter_data(bits);
                    let v = bits.get_value_high32();
                    let filter_type = bits.read_bits_big(3, v) as u8;
                    let channels = if filter_type == 0 {
                        let v = bits.get_value_high32();
                        (bits.read_bits_big(5, v) + 1) as u8
                    } else {
                        0
                    };
                    commands.push(DecodeCmd::Filter {
                        filter_type,
                        block_start,
                        block_length,
                        channels,
                    });
                    fb.reload_from(bits);
                } else if sym == 257 {
                    if last_length != 0 {
                        output_size += last_length as usize;
                    }
                    commands.push(DecodeCmd::FullRep);
                } else if sym < 262 {
                    let rep_idx = (sym - 258) as u8;
                    let length_slot = fb.decode(&tables.len_table) as u32;
                    let length = Self::slot_to_length_fast(length_slot, &mut fb);
                    output_size += length as usize;
                    last_length = length;
                    commands.push(DecodeCmd::Rep { length, rep_idx });
                } else {
                    let len_slot = (sym - 262) as u32;
                    let mut length = Self::slot_to_length_fast(len_slot, &mut fb);
                    let dist_slot = fb.decode(&tables.dist_table) as u32;
                    let offset = Self::decode_offset_fast(dist_slot, tables, &mut fb);
                    length += (offset > 0x100) as u32
                        + (offset > 0x2000) as u32
                        + (offset > 0x40000) as u32;
                    output_size += length as usize;
                    last_length = length;
                    commands.push(DecodeCmd::Match {
                        length,
                        offset: offset as u32,
                    });
                }
                cmd_count += 1;
            }
        }

        if literal_run > 0 {
            commands.push(DecodeCmd::LiteralRun { count: literal_run });
        }

        // SAFETY: we wrote lit_len bytes via raw pointer, set the actual length
        unsafe {
            literals.set_len(lit_len);
        }

        fb.sync_to(bits);

        Ok(SplitDecoded {
            literals,
            commands,
            output_size,
        })
    }

    fn slot_to_length_fast(slot: u32, fb: &mut FastBits) -> u32 {
        if slot < 8 {
            slot + 2
        } else {
            let extra_bits = ((slot - 4) / 4) as u32;
            let base = ((4u32 + (slot & 3)) << extra_bits).wrapping_add(2);
            let extra = fb.read(extra_bits);
            base.wrapping_add(extra)
        }
    }

    /// Fast version of decode_offset using FastBits.
    fn decode_offset_fast(slot: u32, tables: &BlockTables, fb: &mut FastBits) -> usize {
        if slot < 4 {
            (slot + 1) as usize
        } else {
            let num_bits = (slot / 2 - 1) as u32;
            let base = (2 | (slot & 1)) << num_bits;
            if num_bits < NUM_ALIGN_BITS as u32 {
                let extra = fb.read(num_bits);
                base.wrapping_add(extra).wrapping_add(1) as usize
            } else {
                let high = if num_bits > NUM_ALIGN_BITS as u32 {
                    fb.read(num_bits - NUM_ALIGN_BITS as u32)
                } else {
                    0
                };
                let low = if tables.use_align_bits {
                    fb.decode(&tables.align_table) as u32
                } else {
                    fb.read(NUM_ALIGN_BITS as u32)
                };
                base.wrapping_add(high << NUM_ALIGN_BITS)
                    .wrapping_add(low)
                    .wrapping_add(1) as usize
            }
        }
    }

    /// Static version of slot_to_length using BitDecoder (for large-block path).
    #[cfg(feature = "parallel")]
    #[allow(dead_code)]
    fn slot_to_length_static(slot: u32, bits: &mut BitDecoder) -> u32 {
        if slot < 8 {
            slot + 2
        } else {
            let extra_bits = ((slot - 4) / 4) as usize;
            let base = ((4u32 + (slot & 3)) << extra_bits).wrapping_add(2);
            let v = bits.get_value_high32();
            let extra = bits.read_bits_big(extra_bits, v);
            base.wrapping_add(extra)
        }
    }

    /// Static version of decode_offset using BitDecoder (for large-block path).
    #[cfg(feature = "parallel")]
    #[allow(dead_code)]
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
                Ok(base.wrapping_add(extra).wrapping_add(1) as usize)
            } else {
                let high = if num_bits > NUM_ALIGN_BITS {
                    let v = bits.get_value_high32();
                    bits.read_bits_big(num_bits - NUM_ALIGN_BITS, v)
                } else {
                    0
                };
                let low = if tables.use_align_bits {
                    tables.align_table.decode(bits) as u32
                } else {
                    bits.read_bits_9fix(NUM_ALIGN_BITS)
                };
                Ok(base
                    .wrapping_add(high << NUM_ALIGN_BITS)
                    .wrapping_add(low)
                    .wrapping_add(1) as usize)
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
        use super::bit_decoder::BitDecoder;
        use rayon::prelude::*;

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

        for (_header_start, header) in &block_infos {
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

    /// Decode with parallel configuration.
    /// Uses batched processing like unrar for better memory efficiency.
    #[cfg(feature = "parallel")]
    pub fn decode_parallel_with_config(
        &mut self,
        data: &[u8],
        output_size: usize,
        config: &ParallelConfig,
    ) -> Result<Vec<u8>, DecompressError> {
        use super::bit_decoder::BitDecoder;
        use rayon::prelude::*;

        // Determine batch size
        let num_threads = if config.num_threads == 0 {
            rayon::current_num_threads()
        } else {
            config.num_threads
        };
        let blocks_per_batch = if config.blocks_per_batch == 0 {
            num_threads * 2 // Like unrar's UNP_BLOCKS_PER_THREAD = 2
        } else {
            config.blocks_per_batch
        };

        let mut bits = BitDecoder::new(data);
        let mut current_tables = Arc::new(BlockTables::new());
        let mut tables_valid = false;
        let mut all_filters: Vec<super::filter::UnpackFilter> = Vec::new();

        // Pre-allocate output buffer for direct writes (no window needed for small blocks)
        // SAFETY: all positions [0..output_size) are written by decode workers via
        // out_ptr before being read. Truncated to actual size (pos) at the end.
        let mut output: Vec<u8> = Vec::with_capacity(output_size);
        unsafe {
            output.set_len(output_size);
        }
        let out_ptr = output.as_mut_ptr();
        let mut pos = 0usize;

        // Process in batches
        while pos < output_size && !bits.is_eof() {
            let mut batch_blocks: Vec<(BlockHeader, Arc<BlockTables>)> =
                Vec::with_capacity(blocks_per_batch);
            let mut large_blocks: Vec<(BlockHeader, Arc<BlockTables>)> = Vec::new();

            // Collect blocks for this batch
            while batch_blocks.len() + large_blocks.len() < blocks_per_batch {
                if bits.is_eof() {
                    break;
                }

                // Read block header
                bits.align_to_byte();

                if bits.position() + 3 > data.len() {
                    break;
                }

                let flags = bits.read_byte_aligned();
                let checksum = bits.read_byte_aligned();
                let mut check = flags ^ checksum;

                let num = ((flags >> 3) & 3) as usize;
                if num >= 3 {
                    break;
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
                    break; // Invalid checksum
                }

                let block_bit_size = ((flags & 7) + 1) as usize;
                let block_start = bits.position();
                let table_present = (flags & 0x80) != 0;
                let last_block = (flags & 0x40) != 0;

                if block_start + block_size > data.len() {
                    break; // Block extends beyond data
                }

                // Set block boundary for table reading
                let block_end = block_start + block_size - 1;
                bits.set_block_end(block_end, block_bit_size);

                // Read tables if present - create new Arc only when tables change
                if table_present {
                    // Get mutable reference to create new tables
                    let new_tables = Arc::make_mut(&mut current_tables);
                    self.read_tables_into(&mut bits, new_tables)?;
                    tables_valid = true;
                } else if !tables_valid {
                    return Err(DecompressError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Block without tables but no previous tables",
                    )));
                }

                let data_start = bits.position();
                let data_start_bit = bits.bit_pos();

                let header = BlockHeader {
                    block_start,
                    block_size,
                    block_bit_size,
                    table_present,
                    last_block,
                    data_start,
                    data_start_bit,
                };

                // Check if large block (use single-thread path)
                // Arc::clone is O(1) - just increments refcount
                if block_size > config.large_block_size {
                    large_blocks.push((header, Arc::clone(&current_tables)));
                } else {
                    batch_blocks.push((header, Arc::clone(&current_tables)));
                }

                // Move to end of block
                bits.set_position(block_start + block_size);

                if last_block {
                    break;
                }
            }

            if batch_blocks.is_empty() && large_blocks.is_empty() {
                break; // No more blocks
            }

            // Process large blocks single-threaded with direct apply
            for (header, tables) in &large_blocks {
                let mut block_bits = bits.clone_view();
                block_bits.set_position_with_bit(header.data_start, header.data_start_bit);

                let split = Self::decode_block_split(
                    &mut block_bits,
                    tables,
                    header,
                    config.max_items_per_block,
                )?;
                let filters = self.apply_split_direct(&split, out_ptr, &mut pos)?;
                all_filters.extend(filters);
            }

            // Parallel decode normal blocks
            if !batch_blocks.is_empty() {
                let decoded_results: Vec<Result<SplitDecoded, DecompressError>> = batch_blocks
                    .par_iter()
                    .map(|(header, tables)| {
                        let mut block_bits = bits.clone_view();
                        block_bits.set_position_with_bit(header.data_start, header.data_start_bit);
                        let block_end = header.block_start + header.block_size - 1;
                        block_bits.set_block_end(block_end, header.block_bit_size);

                        Self::decode_block_split(
                            &mut block_bits,
                            tables,
                            header,
                            config.max_items_per_block,
                        )
                    })
                    .collect();

                // Apply decoded items sequentially (order matters for REP references)
                for result in decoded_results {
                    let split = result?;
                    let filters = self.apply_split_direct(&split, out_ptr, &mut pos)?;
                    all_filters.extend(filters);
                }
            }

            // Check if we hit the last block
            let is_last = batch_blocks
                .last()
                .map(|(h, _)| h.last_block)
                .unwrap_or(false)
                || large_blocks
                    .last()
                    .map(|(h, _)| h.last_block)
                    .unwrap_or(false);
            if is_last {
                break;
            }
        }

        // Truncate output to actual decoded size
        let final_len = pos.min(output_size);
        // SAFETY: final_len <= capacity, all bytes [0..final_len) written by decode workers
        unsafe {
            output.set_len(final_len);
        }

        // Apply filters if any
        if !all_filters.is_empty() {
            // Sort filters by block start position
            all_filters.sort_by_key(|f| f.block_start);

            for filter in &all_filters {
                let start = filter.block_start;
                let end = start + filter.block_length;

                if end <= output.len() {
                    // Apply filter - may be in-place or return new buffer
                    let block = &mut output[start..end];
                    let filtered = super::filter::apply_filter(block, filter, start as u64);

                    // If filtered is non-empty, it's a Delta filter result (not in-place)
                    if !filtered.is_empty() {
                        output[start..end].copy_from_slice(&filtered);
                    }
                }
            }
        }

        Ok(output)
    }

    /// Decode a large block directly to window (single-threaded).
    /// Returns any filters found in the block.
    #[cfg(feature = "parallel")]
    #[allow(dead_code)]
    fn decode_large_block(
        &mut self,
        bits: &mut super::bit_decoder::BitDecoder,
        tables: &BlockTables,
    ) -> Result<Vec<super::filter::UnpackFilter>, DecompressError> {
        let mut filters = Vec::new();

        while !bits.is_eof() && !bits.is_block_over_read() {
            let sym = tables.main_table.decode(bits) as usize;

            if sym < 256 {
                self.write_byte(sym as u8);
            } else if sym == 256 {
                if let Some(filter) = self.read_filter(bits)? {
                    filters.push(filter);
                }
            } else if sym == 257 {
                if self.last_length != 0 && self.recent_offsets[0] != 0 {
                    let length = self.last_length;
                    let offset = self.recent_offsets[0] as usize;
                    self.copy_bytes(offset, length);
                }
            } else if sym < 262 {
                let rep_idx = sym - 258;
                let offset = self.recent_offsets[rep_idx] as usize;
                if offset == 0 {
                    return Err(DecompressError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid zero offset in large block",
                    )));
                }

                let length = self.decode_length_with_tables(bits, tables)?;

                if rep_idx > 0 {
                    let off = self.recent_offsets[rep_idx];
                    for j in (1..=rep_idx).rev() {
                        self.recent_offsets[j] = self.recent_offsets[j - 1];
                    }
                    self.recent_offsets[0] = off;
                }
                self.last_length = length;
                self.copy_bytes(offset, length);
            } else {
                let len_slot = (sym - 262) as u32;
                let length = Self::slot_to_length_static(len_slot, bits);
                let dist_slot = tables.dist_table.decode(bits) as u32;
                let offset = Self::decode_offset_static(dist_slot, tables, bits)?;

                let mut adj_length = length;
                adj_length +=
                    (offset > 0x100) as u32 + (offset > 0x2000) as u32 + (offset > 0x40000) as u32;

                for j in (1..NUM_REPS).rev() {
                    self.recent_offsets[j] = self.recent_offsets[j - 1];
                }
                self.recent_offsets[0] = offset as u32;
                self.last_length = adj_length as usize;
                self.copy_bytes(offset, adj_length as usize);
            }
        }
        Ok(filters)
    }

    /// Decode length using provided tables.
    #[cfg(feature = "parallel")]
    #[allow(dead_code)]
    fn decode_length_with_tables(
        &self,
        bits: &mut super::bit_decoder::BitDecoder,
        tables: &BlockTables,
    ) -> Result<usize, DecompressError> {
        let sym = tables.len_table.decode(bits) as u32;
        Ok(Self::slot_to_length_static(sym, bits) as usize)
    }

    /// Pipeline-based parallel decode with minimal sync overhead.
    /// Uses streaming instead of batch sync barriers.
    #[cfg(feature = "parallel")]
    pub fn decode_pipeline(
        &mut self,
        data: &[u8],
        output_size: usize,
    ) -> Result<Vec<u8>, DecompressError> {
        use super::bit_decoder::BitDecoder;
        use rayon::prelude::*;
        use std::thread;

        // Phase 1: Scan all block headers and prepare decode jobs
        let mut bits = BitDecoder::new(data);
        let mut current_tables = Arc::new(BlockTables::new());
        let mut tables_valid = false;

        // Collect all blocks with their sequence numbers and tables
        let mut all_blocks: Vec<(usize, BlockHeader, Arc<BlockTables>)> = Vec::new();
        let mut seq = 0usize;

        while !bits.is_eof() {
            bits.align_to_byte();

            if bits.position() + 3 > data.len() {
                break;
            }

            let flags = bits.read_byte_aligned();
            let checksum = bits.read_byte_aligned();
            let mut check = flags ^ checksum;

            let num = ((flags >> 3) & 3) as usize;
            if num >= 3 {
                break;
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
                break;
            }

            let block_bit_size = ((flags & 7) + 1) as usize;
            let block_start = bits.position();
            let table_present = (flags & 0x80) != 0;
            let last_block = (flags & 0x40) != 0;

            if block_start + block_size > data.len() {
                break;
            }

            let block_end = block_start + block_size - 1;
            bits.set_block_end(block_end, block_bit_size);

            // Read tables if present
            if table_present {
                let new_tables = Arc::make_mut(&mut current_tables);
                self.read_tables_into(&mut bits, new_tables)?;
                tables_valid = true;
            } else if !tables_valid {
                return Err(DecompressError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Block without tables but no previous tables",
                )));
            }

            let data_start = bits.position();
            let data_start_bit = bits.bit_pos();

            let header = BlockHeader {
                block_start,
                block_size,
                block_bit_size,
                table_present,
                last_block,
                data_start,
                data_start_bit,
            };

            all_blocks.push((seq, header, Arc::clone(&current_tables)));
            seq += 1;

            bits.set_position(block_start + block_size);

            if last_block {
                break;
            }
        }

        if all_blocks.is_empty() {
            return Ok(Vec::new());
        }

        let total_blocks = all_blocks.len();

        // Phase 2: Create channel and spawn apply thread
        let (tx, rx) = crossbeam_channel::unbounded::<(usize, SplitDecoded)>();

        let dict_size_log = (self.dict_size.trailing_zeros() as u8).max(17);
        let apply_handle = thread::spawn(move || {
            Self::apply_pipeline_split(rx, output_size, dict_size_log, total_blocks)
        });

        // Phase 3: Parallel decode all blocks, streaming to channel
        let max_items = 0x4100usize;

        all_blocks
            .into_par_iter()
            .for_each(|(seq, header, tables)| {
                let mut block_bits = bits.clone_view();
                block_bits.set_position_with_bit(header.data_start, header.data_start_bit);
                let block_end = header.block_start + header.block_size - 1;
                block_bits.set_block_end(block_end, header.block_bit_size);

                match Self::decode_block_split(&mut block_bits, &tables, &header, max_items) {
                    Ok(split) => {
                        let _ = tx.send((seq, split));
                    }
                    Err(_) => {
                        let _ = tx.send((
                            seq,
                            SplitDecoded {
                                literals: Vec::new(),
                                commands: Vec::new(),
                                output_size: 0,
                            },
                        ));
                    }
                }
            });

        drop(tx); // Signal completion

        // Phase 4: Wait for apply thread
        apply_handle.join().map_err(|_| {
            DecompressError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Apply thread panicked",
            ))
        })?
    }

    /// Apply thread for pipeline decoding (split-buffer version).
    #[cfg(feature = "parallel")]
    fn apply_pipeline_split(
        rx: crossbeam_channel::Receiver<(usize, SplitDecoded)>,
        output_size: usize,
        dict_size_log: u8,
        _total_blocks: usize,
    ) -> Result<Vec<u8>, DecompressError> {
        use std::collections::BTreeMap;

        let mut decoder = Rar5BlockDecoder::new(dict_size_log);

        // SAFETY: all positions [0..output_size) are written by apply_split_direct
        // via out_ptr before being read. Truncated to actual size at the end.
        let mut output: Vec<u8> = Vec::with_capacity(output_size);
        unsafe {
            output.set_len(output_size);
        }
        let out_ptr = output.as_mut_ptr();
        let mut pos = 0usize;

        let mut pending: BTreeMap<usize, SplitDecoded> = BTreeMap::new();
        let mut next_seq = 0usize;
        let mut all_filters: Vec<super::filter::UnpackFilter> = Vec::new();

        while let Ok((seq, split)) = rx.recv() {
            if seq == next_seq {
                let filters = decoder.apply_split_direct(&split, out_ptr, &mut pos)?;
                all_filters.extend(filters);
                next_seq += 1;

                while let Some(split) = pending.remove(&next_seq) {
                    let filters = decoder.apply_split_direct(&split, out_ptr, &mut pos)?;
                    all_filters.extend(filters);
                    next_seq += 1;
                }
            } else {
                pending.insert(seq, split);
            }
        }

        while let Some((seq, split)) = pending.pop_first() {
            if seq != next_seq {
                return Err(DecompressError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Missing block {} in pipeline", next_seq),
                )));
            }

            let filters = decoder.apply_split_direct(&split, out_ptr, &mut pos)?;
            all_filters.extend(filters);
            next_seq += 1;
        }

        let final_len = pos.min(output_size);
        // SAFETY: final_len <= capacity, all bytes [0..final_len) written by apply_split_direct
        unsafe {
            output.set_len(final_len);
        }

        if !all_filters.is_empty() {
            all_filters.sort_by_key(|f| f.block_start);

            for filter in &all_filters {
                let start = filter.block_start;
                let end = start + filter.block_length;

                if end <= output.len() {
                    let block = &mut output[start..end];
                    let filtered = super::filter::apply_filter(block, filter, start as u64);

                    if !filtered.is_empty() {
                        output[start..end].copy_from_slice(&filtered);
                    }
                }
            }
        }

        Ok(output)
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
