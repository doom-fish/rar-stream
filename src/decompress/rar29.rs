//! RAR 2.9 (RAR4) decompression.
//!
//! Implements the LZSS + Huffman decompression used in RAR versions 2.x-4.x.
//! This is the most common format for scene releases.

// Allow disabled debug blocks in test code (written >= 0 && written < 0 is intentionally false)
#![cfg_attr(test, allow(clippy::logic_bug))]

use super::{
    bit_reader::BitReader,
    huffman::HuffmanDecoder,
    lzss::LzssDecoder,
    ppm::{PpmModel, RangeCoder},
    vm::RarVM,
    DecompressError, Result,
};

#[allow(dead_code)]
/// Number of main codes (literals + length symbols).
const MAIN_CODES: usize = 299;

#[allow(dead_code)]
/// Number of distance codes.
const DIST_CODES: usize = 60;

#[allow(dead_code)]
/// Number of low distance codes.
const LOW_DIST_CODES: usize = 17;

#[allow(dead_code)]
/// Number of length codes.
const LEN_CODES: usize = 28;

#[allow(dead_code)]
/// Maximum match length.
const MAX_MATCH_LEN: u32 = 258;

/// Short distance bases for symbols 263-270.
const SHORT_BASES: [u32; 8] = [0, 4, 8, 16, 32, 64, 128, 192];

/// Short distance extra bits for symbols 263-270.
const SHORT_BITS: [u8; 8] = [2, 2, 3, 4, 5, 6, 6, 6];

/// Base lengths for length codes.
const LENGTH_BASE: [u32; 28] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 14, 16, 20, 24, 28, 32, 40, 48, 56, 64, 80, 96, 112, 128,
    160, 192, 224,
];

/// Extra bits for length codes.
const LENGTH_EXTRA: [u8; 28] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5,
];

/// Base distances for distance codes (48 entries for RAR3).
const DIST_BASE: [u32; 60] = [
    0, 1, 2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64, 96, 128, 192, 256, 384, 512, 768, 1024, 1536,
    2048, 3072, 4096, 6144, 8192, 12288, 16384, 24576, 32768, 49152, 65536, 98304, 131072, 196608,
    262144, 327680, 393216, 458752, 524288, 589824, 655360, 720896, 786432, 851968, 917504, 983040,
    1048576, 1310720, 1572864, 1835008, 2097152, 2359296, 2621440, 2883584, 3145728, 3407872,
    3670016, 3932160,
];

/// Extra bits for distance codes (60 entries for RAR3).
const DIST_EXTRA: [u8; 60] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13, 14, 14, 15, 15, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 18, 18, 18, 18, 18,
    18, 18, 18, 18, 18, 18, 18,
];

/// RAR 2.9 (RAR4) decoder state.
///
/// Handles LZSS + Huffman decompression with PPMd fallback and VM-based
/// filters for RAR 1.5–4.x archives.
///
/// # Example
///
/// ```
/// use rar_stream::Rar29Decoder;
///
/// let mut decoder = Rar29Decoder::new();
/// // decoder.decompress(&compressed_data, expected_size) to decompress
/// ```
pub struct Rar29Decoder {
    /// LZSS sliding window
    lzss: LzssDecoder,
    /// Huffman decoder
    huffman: HuffmanDecoder,
    /// VM for filter execution
    vm: RarVM,
    /// PPMd model (used when ppm_mode is true)
    ppm: Option<PpmModel>,
    /// PPMd range coder (used when ppm_mode is true)
    ppm_coder: Option<RangeCoder>,
    /// PPMd escape character
    ppm_esc_char: i32,
    /// Previous distances for repeat matches
    old_dist: [u32; 4],
    /// Current distance history index
    old_dist_ptr: usize,
    /// Last distance used
    last_dist: u32,
    /// Last length used
    last_len: u32,
    /// PPMd mode flag
    ppm_mode: bool,
    /// Tables need reading
    tables_read: bool,
    /// Previous low offset value for repeat
    prev_low_offset: u32,
    /// Low offset repeat counter
    low_offset_repeat_count: u32,
    /// Next position where we need to check filters (optimization to avoid O(n) scan)
    next_filter_check: u64,
}

impl Rar29Decoder {
    /// Create a new RAR29 decoder with default window size (4MB).
    pub fn new() -> Self {
        Self::with_window_size(0x400000) // 4MB default (max common size)
    }

    /// Create a new RAR29 decoder with specified window size.
    /// Window size must be a power of 2.
    pub fn with_window_size(window_size: usize) -> Self {
        Self {
            lzss: LzssDecoder::new(window_size),
            huffman: HuffmanDecoder::new(),
            vm: RarVM::new(),
            ppm: None,
            ppm_coder: None,
            ppm_esc_char: -1,
            old_dist: [0; 4],
            old_dist_ptr: 0,
            last_dist: 0,
            last_len: 0,
            ppm_mode: false,
            tables_read: false,
            prev_low_offset: 0,
            low_offset_repeat_count: 0,
            next_filter_check: u64::MAX,
        }
    }

    /// Get partial output (for debugging failed decompression)
    #[cfg(test)]
    pub fn get_output(&self) -> Vec<u8> {
        self.lzss.output().to_vec()
    }

    /// Decompress a block of data.
    /// Returns the decompressed data.
    pub fn decompress(&mut self, data: &[u8], unpacked_size: u64) -> Result<Vec<u8>> {
        let mut reader = BitReader::new(data);

        // Enable output accumulation for files (especially those larger than window)
        self.lzss.enable_output(unpacked_size as usize);

        // Read tables if needed
        if !self.tables_read {
            self.read_tables(&mut reader)?;
        }

        // Decompress until we have enough data
        while self.lzss.total_written() < unpacked_size {
            if reader.is_eof() {
                break;
            }

            self.decode_block(&mut reader, unpacked_size)?;
        }

        // Execute any remaining pending VM filters
        let total_written = self.lzss.total_written();
        let window_mask = self.lzss.window_mask() as usize;

        // Execute filters in order of their block_start position
        loop {
            // Find the earliest filter that is ready
            let (filter_idx, next_pos) = match self.vm.find_ready_filter(total_written) {
                Some((idx, pos)) => (idx, pos),
                None => break,
            };

            // Flush up to filter start
            let flushed = self.lzss.flushed_pos();
            if flushed < next_pos {
                self.lzss.flush_to_output(next_pos);
            }

            let window = self.lzss.window();
            if let Some((_filter_end, filtered_data)) =
                self.vm
                    .execute_filter_at_index(filter_idx, window, window_mask, total_written)
            {
                // Write filtered data directly to output
                self.lzss.write_filtered_to_output(filtered_data, next_pos);
            } else {
                break;
            }
        }

        // Flush any remaining data to output
        self.lzss.flush_to_output(total_written);

        // Extract the decompressed data
        Ok(self.lzss.take_output())
    }

    /// Read Huffman tables from the bit stream.
    fn read_tables(&mut self, reader: &mut BitReader) -> Result<()> {
        #[cfg(test)]
        {
            let byte_pos = reader.bit_position() / 8;
            eprintln!(
                "read_tables ENTRY: bit_pos={}, byte_pos={}",
                reader.bit_position(),
                byte_pos
            );
            eprintln!("  raw bytes at pos: {:02x?}", reader.peek_bytes(8));
        }
        // Align to byte boundary (like unrar)
        reader.align_to_byte();
        #[cfg(test)]
        {
            let byte_pos = reader.bit_position() / 8;
            eprintln!(
                "read_tables AFTER align: bit_pos={}, byte_pos={}",
                reader.bit_position(),
                byte_pos
            );
            eprintln!("  raw bytes at pos: {:02x?}", reader.peek_bytes(8));
        };

        // Peek at the high bit to check for PPM mode
        // In unrar, this is done by peeking 16 bits and checking bit 15
        let ppm_flag = reader.peek_bits(1) != 0;

        self.ppm_mode = ppm_flag;

        if self.ppm_mode {
            // DON'T consume the PPM flag bit - it's part of the MaxOrder byte
            // Initialize or reuse PPMd model
            let ppm = self.ppm.get_or_insert_with(PpmModel::new);
            match ppm.init(reader) {
                Ok((coder, esc_char)) => {
                    self.ppm_coder = Some(coder);
                    self.ppm_esc_char = esc_char;
                    #[cfg(test)]
                    println!("PPMd initialized: esc_char={}", esc_char);
                }
                Err(e) => {
                    #[cfg(test)]
                    println!("PPMd init failed: {}", e);
                    #[cfg(not(test))]
                    let _ = e;
                    return Err(DecompressError::UnsupportedMethod(0x33));
                }
            }
        } else {
            // LZ mode - reset low dist state (per unrar ReadTables30)
            self.prev_low_offset = 0;
            self.low_offset_repeat_count = 0;

            // Check bit 1 (0x4000) for reset tables
            let reset_tables = reader.peek_bits(2) & 1 == 0; // Bit 14 inverted (0 means reset)
                                                             // Consume the 2 header bits (PPM flag + reset flag)
            reader.advance_bits(2);

            if reset_tables {
                self.huffman.reset_tables();
            }

            // Read Huffman tables
            self.huffman.read_tables_after_header(reader)?;
        }

        self.tables_read = true;
        Ok(())
    }

    /// Decode a block of data.
    fn decode_block(&mut self, reader: &mut BitReader, max_size: u64) -> Result<()> {
        if self.ppm_mode {
            return self.decode_block_ppm(reader, max_size);
        }

        // Validate tables exist
        if self.huffman.main_table.is_none() || self.huffman.dist_table.is_none() {
            return Err(DecompressError::InvalidHuffmanCode);
        }

        #[cfg(test)]
        let mut symbol_count = 0;

        while self.lzss.total_written() < max_size && !reader.is_eof() {
            // Check if we need to execute pending VM filters
            self.maybe_execute_filters();

            // Decode main symbol
            #[cfg(test)]
            let bit_pos_main_start = reader.bit_position();
            #[cfg(test)]
            let peek_bits = reader.peek_bits(16);

            // SAFETY: We validated main_table.is_some() above
            let symbol = unsafe {
                self.huffman
                    .main_table
                    .as_ref()
                    .unwrap_unchecked()
                    .decode(reader)?
            };

            #[cfg(test)]
            {
                let pos = self.lzss.total_written();
                if pos >= 1498580 && pos <= 1498610 {
                    let bit_pos_after = reader.bit_position();
                    eprintln!(
                        "MAIN sym={} at pos={}, bits {}->{}  peek={:016b}",
                        symbol, pos, bit_pos_main_start, bit_pos_after, peek_bits
                    );
                }
            }

            if symbol < 256 {
                // Literal byte — most common case, skip rest of dispatch
                #[cfg(test)]
                {
                    let pos = self.lzss.total_written();
                    if pos >= 1498595 && pos <= 1498610 {
                        eprintln!("WRITING literal 0x{:02x} at output pos {}", symbol, pos);
                    }
                }
                self.lzss.write_literal(symbol as u8);
            } else if symbol == 256 {
                // End of block / new tables
                // From unrar ReadEndOfBlock:
                // "1"  - no new file, new table just here.
                // "00" - new file,    no new table.
                // "01" - new file,    new table (in beginning of next file).
                #[cfg(test)]
                eprintln!(
                    "\n=== SYMBOL 256 (end of block) at output pos {}, bit_pos {} ===",
                    self.lzss.total_written(),
                    reader.bit_position()
                );
                if !reader.is_eof() {
                    let first_bit = reader.read_bit()?;
                    #[cfg(test)]
                    eprintln!(
                        "  first_bit={}, bit_pos after={}",
                        first_bit,
                        reader.bit_position()
                    );
                    if first_bit {
                        // "1" = new tables, continue decompression
                        // Reset low dist state when reading new tables
                        self.prev_low_offset = 0;
                        self.low_offset_repeat_count = 0;
                        // Call full read_tables which aligns to byte and reads header
                        self.read_tables(reader)?;
                        #[cfg(test)]
                        {
                            eprintln!(
                                "After new tables: bit_pos={}, next 16 bits={:016b}",
                                reader.bit_position(),
                                reader.peek_bits(16)
                            );
                            eprintln!("  About to decode first symbol after table read");
                        }
                        // Continue decompressing - don't break!
                        continue;
                    }
                    // "0x" = new file (end of this file's data)
                    let _second_bit = reader.read_bit()?; // consume the second bit
                                                          // Break out - we're done with this file
                }
                break;
            } else if symbol == 257 {
                // VM filter code - read and skip it
                #[cfg(test)]
                eprintln!(
                    "\n=== SYMBOL 257 (VM code) at output pos {} ===",
                    self.lzss.total_written()
                );
                self.read_vm_code(reader)?;
            } else if symbol == 258 {
                // Repeat last match
                if self.last_len > 0 {
                    #[cfg(test)]
                    {
                        let pos = self.lzss.total_written();
                        let end = pos + self.last_len as u64;
                        if pos <= 1498598 && end > 1498598 {
                            eprintln!(
                                "!!! AT 1498598: symbol 258 repeat, last_dist={}, last_len={}",
                                self.last_dist, self.last_len
                            );
                        }
                    }
                    self.lzss.copy_match(self.last_dist, self.last_len)?;
                }
            } else if symbol < 263 {
                // Use one of the old distances (symbols 259-262 = indices 0-3)
                let idx = (symbol - 259) as usize;
                let distance = self.old_dist[idx];

                // Decode length using the length table
                let length = self.decode_length_from_table(reader)?;

                #[cfg(test)]
                {
                    let written = self.lzss.total_written();
                    let end = written + length as u64;
                    if written <= 1498598 && end > 1498598 {
                        eprintln!(
                            "!!! AT 1498598: old idx={},len={},dist={}",
                            idx, length, distance
                        );
                    }
                }

                self.lzss.copy_match(distance, length)?;

                // Shift old distances: move entries 0..idx up by 1, put this at 0
                for i in (1..=idx).rev() {
                    self.old_dist[i] = self.old_dist[i - 1];
                }
                self.old_dist[0] = distance;
                self.last_dist = distance;
                self.last_len = length;
            } else if symbol <= 270 {
                // Short match (symbols 263-270): fixed length=2, short distance
                let idx = (symbol - 263) as usize;
                let base = SHORT_BASES[idx];
                let bits = SHORT_BITS[idx];
                let extra = if bits > 0 {
                    reader.read_bits(bits as u32)?
                } else {
                    0
                };
                let distance = base + extra + 1;
                let length = 2u32;

                #[cfg(test)]
                {
                    let written = self.lzss.total_written();
                    let end = written + length as u64;
                    if written <= 1498598 && end > 1498598 {
                        eprintln!(
                            "!!! AT 1498598: short sym={}, idx={}, base={}, bits={}, extra={}, dist={}",
                            symbol, idx, base, bits, extra, distance
                        );
                    }
                }

                self.lzss.copy_match(distance, length)?;

                // Shift old distances
                for i in (1..4).rev() {
                    self.old_dist[i] = self.old_dist[i - 1];
                }
                self.old_dist[0] = distance;
                self.old_dist_ptr = 0;
                self.last_dist = distance;
                self.last_len = length;
            } else {
                // Long match (symbols 271-298): length from main symbol, distance from offset table
                #[cfg(test)]
                let bit_before_len = reader.bit_position();

                let len_idx = (symbol - 271) as usize;
                let length = if len_idx < LENGTH_BASE.len() {
                    let base = LENGTH_BASE[len_idx];
                    let extra = LENGTH_EXTRA[len_idx];
                    let extra_val = if extra > 0 {
                        reader.read_bits(extra as u32)?
                    } else {
                        0
                    };
                    #[cfg(test)]
                    {
                        let written = self.lzss.total_written();
                        if written >= 1498595 && written <= 1498602 {
                            let bit_after_len = reader.bit_position();
                            eprintln!(
                                "!!! LONG DECODE at {}: sym={}, len_idx={}, len={}, bits {}->{}]",
                                written,
                                symbol,
                                len_idx,
                                base + extra_val + 3,
                                bit_before_len,
                                bit_after_len
                            );
                        }
                    }
                    base + extra_val + 3 // +3 because minimum match length for long matches is 3
                } else {
                    #[cfg(test)]
                    eprintln!(
                        "\nlen_idx {} out of range at written={}",
                        len_idx,
                        self.lzss.total_written()
                    );
                    return Err(DecompressError::InvalidHuffmanCode);
                };

                // Decode distance from offset table
                let dist_symbol = {
                    #[cfg(test)]
                    let bit_pos_before = reader.bit_position();

                    // SAFETY: We validated dist_table.is_some() at function start
                    let dist_table = unsafe { self.huffman.dist_table.as_ref().unwrap_unchecked() };
                    match dist_table.decode(reader) {
                        Ok(s) => {
                            #[cfg(test)]
                            {
                                let written = self.lzss.total_written();
                                if written >= 1498595 && written <= 1498610 {
                                    let bit_pos_after = reader.bit_position();
                                    eprintln!(
                                        "  dist_symbol={} at pos {} (bits {}->{})",
                                        s, written, bit_pos_before, bit_pos_after
                                    );
                                }
                            }
                            s
                        }
                        Err(e) => {
                            #[cfg(test)]
                            eprintln!(
                                "\nOffset decode failed at written={}, len={}",
                                self.lzss.total_written(),
                                length
                            );
                            return Err(e);
                        }
                    }
                };

                let dist_code = dist_symbol as usize;
                let distance = if dist_code < DIST_BASE.len() {
                    let base = DIST_BASE[dist_code];
                    let extra = DIST_EXTRA[dist_code];

                    let extra_val = if extra > 0 {
                        if dist_code > 9 {
                            // For dist_code > 9, use low offset table
                            // First read high bits if extra > 4
                            let high = if extra > 4 {
                                #[cfg(test)]
                                let high_bit_pos = reader.bit_position();
                                let h = reader.read_bits((extra - 4) as u32)?;
                                #[cfg(test)]
                                {
                                    let written = self.lzss.total_written();
                                    if (written >= 1498595 && written <= 1498610)
                                        || (written >= 2176060 && written <= 2176080)
                                    {
                                        eprintln!(
                                            "    high bits at {}: {} bits = {} (0b{:016b}), pos {}->{}",
                                            written,
                                            extra - 4,
                                            h, h,
                                            high_bit_pos,
                                            reader.bit_position()
                                        );
                                    }
                                }
                                h << 4
                            } else {
                                0
                            };
                            // Then decode low offset (0-15 or 16 for repeat)
                            let low = if self.low_offset_repeat_count > 0 {
                                self.low_offset_repeat_count -= 1;
                                #[cfg(test)]
                                {
                                    let written = self.lzss.total_written();
                                    if written >= 1498550 && written <= 1498610 {
                                        eprintln!(
                                            "!!! low_offset REPEAT at {}: prev={}",
                                            written, self.prev_low_offset
                                        );
                                    }
                                }
                                self.prev_low_offset
                            } else {
                                #[cfg(test)]
                                let bit_pos_before = reader.bit_position();
                                #[cfg(test)]
                                let raw_bits_16 = reader.peek_bits(16);
                                // SAFETY: low_dist_table is always initialized when we reach here
                                let low_table = unsafe {
                                    self.huffman.low_dist_table.as_ref().unwrap_unchecked()
                                };
                                #[cfg(test)]
                                {
                                    let written = self.lzss.total_written();
                                    if written == 1498598 {
                                        // Dump the decode_len array and symbols
                                        eprintln!(
                                            "!!! LOW_TABLE at 1498598 decode_len: {:?}",
                                            low_table.dump_decode_len()
                                        );
                                        eprintln!(
                                            "!!! LOW_TABLE at 1498598 symbols: {:?}",
                                            low_table.dump_symbols()
                                        );
                                    }
                                }
                                let sym = low_table.decode(reader)? as u32;
                                #[cfg(test)]
                                {
                                    let written = self.lzss.total_written();
                                    if written >= 1498550 && written <= 1498610 {
                                        let bit_pos_after = reader.bit_position();
                                        eprintln!("!!! low_offset at {}: sym={} (bits {}->{}), raw peek = {:016b}", 
                                            written, sym, bit_pos_before, bit_pos_after, raw_bits_16);
                                    }
                                }

                                if sym == 16 {
                                    // Repeat previous low offset - total 16 uses (this one + 15 more)
                                    // unrar: LowDistRepCount=LOW_DIST_REP_COUNT-1 where LOW_DIST_REP_COUNT=16
                                    self.low_offset_repeat_count = 16 - 1; // 15 more uses after this one
                                    self.prev_low_offset
                                } else {
                                    self.prev_low_offset = sym;
                                    sym
                                }
                            };
                            #[cfg(test)]
                            {
                                let written = self.lzss.total_written();
                                if written >= 2176060 && written <= 2176080 {
                                    if self.low_offset_repeat_count > 0 {
                                        eprintln!(
                                            "  low_offset REPEAT at {}: prev={}, remaining={}",
                                            written,
                                            self.prev_low_offset,
                                            self.low_offset_repeat_count
                                        );
                                    } else {
                                        eprintln!("  low_offset at {}: dist_code={}, base={}, extra={}, high={}, low={}, dist={}", 
                                            written, dist_code, base, extra, high, low, base + high + low + 1);
                                    }
                                }
                            }
                            high + low
                        } else {
                            // For dist_code <= 9, read extra bits directly
                            #[cfg(test)]
                            let peek = reader.peek_bits(extra as u32);
                            let val = reader.read_bits(extra as u32)?;
                            #[cfg(test)]
                            {
                                let written = self.lzss.total_written();
                                if written >= 0 && written < 0 {
                                    eprintln!("  direct: dist_code={}, base={}, extra_bits={}, peek={:04b}, extra_val={}, distance={}", 
                                        dist_code, base, extra, peek, val, base + val + 1);
                                }
                            }
                            val
                        }
                    } else {
                        0
                    };
                    base + extra_val + 1
                } else {
                    #[cfg(test)]
                    eprintln!(
                        "\ndist_code {} out of range at written={}",
                        dist_code,
                        self.lzss.total_written()
                    );
                    return Err(DecompressError::InvalidHuffmanCode);
                };

                // Length bonus for long distances (RAR3 specific)
                // Per unrar: if (Distance>=0x2000) { Length++; if (Distance>=0x40000) Length++; }
                let length = if distance >= 0x2000 {
                    if distance >= 0x40000 {
                        length + 2
                    } else {
                        length + 1
                    }
                } else {
                    length
                };

                #[cfg(test)]
                {
                    let written = self.lzss.total_written();
                    let end = written + length as u64;
                    if written <= 1498598 && end > 1498598 {
                        eprintln!(
                            "!!! AT 1498598: long match dist={}, len={}",
                            distance, length
                        );
                        // Check what's in the window at source position
                        let src_pos = (written as u32).wrapping_sub(distance) as usize;
                        let mask = self.lzss.window_mask() as usize;
                        let window = self.lzss.window();
                        eprintln!(
                            "  window src[{}..{}]: {:02x?}",
                            src_pos,
                            src_pos + length as usize,
                            &window[src_pos..src_pos + length as usize]
                        );
                    }
                    if written >= 1498595 && written <= 1498602 {
                        eprintln!(
                            "LONG MATCH at {}: dist={}, len={}",
                            written, distance, length
                        );
                    }
                }

                self.lzss.copy_match(distance, length)?;

                // Shift old distances
                for i in (1..4).rev() {
                    self.old_dist[i] = self.old_dist[i - 1];
                }
                self.old_dist[0] = distance;
                self.old_dist_ptr = 0;
                self.last_dist = distance;
                self.last_len = length;
            }
        }

        Ok(())
    }

    /// Decode a length value using the length table.
    fn decode_length_from_table(&mut self, reader: &mut BitReader) -> Result<u32> {
        let symbol = {
            let len_table = self
                .huffman
                .len_table
                .as_ref()
                .ok_or(DecompressError::InvalidHuffmanCode)?;
            len_table.decode(reader)?
        };

        let sym = symbol as usize;
        if sym < LENGTH_BASE.len() {
            let base = LENGTH_BASE[sym];
            let extra = LENGTH_EXTRA[sym];
            let extra_val = if extra > 0 {
                reader.read_bits(extra as u32)?
            } else {
                0
            };
            Ok(base + extra_val + 2)
        } else {
            Err(DecompressError::InvalidHuffmanCode)
        }
    }

    /// Read VM filter code from bit stream (for LZ mode, symbol 257).
    /// We read the VM code and register it with the VM for later execution.
    #[cold]
    fn read_vm_code(&mut self, reader: &mut BitReader) -> Result<()> {
        #[cfg(test)]
        let bit_pos_start = reader.bit_position();

        // Read first byte
        let first_byte = reader.read_bits(8)? as u8;

        // Calculate length based on unrar's ReadVMCode logic:
        // Length = (FirstByte & 7) + 1
        // if Length == 7, read another byte and add 7
        // if Length == 8, read 16 bits as length
        let length = {
            let base = (first_byte & 7) + 1;
            match base {
                7 => {
                    // Read one more byte, add 7
                    let next = reader.read_bits(8)? as u32;
                    next + 7
                }
                8 => {
                    // Read 16 bits as length
                    reader.read_bits(16)?
                }
                _ => base as u32,
            }
        };

        #[cfg(test)]
        eprintln!(
            "  read_vm_code: first_byte=0x{:02x}, length={}, bit_pos_start={}",
            first_byte, length, bit_pos_start
        );

        if length == 0 {
            return Ok(());
        }

        // Read VM code bytes
        let mut vm_code = vec![0u8; length as usize];
        for i in 0..length as usize {
            vm_code[i] = reader.read_bits(8)? as u8;
        }

        #[cfg(test)]
        eprintln!("    vm_code end bit_pos={}", reader.bit_position());

        // Add to VM for later execution - use absolute total_written, not wrapped window position
        let total_written = self.lzss.total_written();
        let window_mask = self.lzss.window_mask();

        #[cfg(test)]
        eprintln!(
            "    add_code: total_written={}, window_mask={:x}",
            total_written, window_mask
        );

        #[cfg(test)]
        {
            let had_pending_before = self.vm.has_pending_filters();
            let result = self
                .vm
                .add_code(first_byte, &vm_code, total_written, window_mask);
            let has_pending_after = self.vm.has_pending_filters();
            if let Some(next_pos) = self.vm.next_filter_pos() {
                eprintln!(
                    "    vm.add_code: added={}, pending={}->{}, next_pos={}",
                    result, had_pending_before, has_pending_after, next_pos
                );
            } else {
                eprintln!(
                    "    vm.add_code: added={}, pending={}->{}, next_pos=NONE",
                    result, had_pending_before, has_pending_after
                );
            }
        }
        #[cfg(not(test))]
        self.vm
            .add_code(first_byte, &vm_code, total_written, window_mask);

        // Update next_filter_check when a filter is added
        if let Some(end) = self.vm.next_filter_end() {
            self.next_filter_check = self.next_filter_check.min(end);
        }

        Ok(())
    }

    /// Execute pending VM filters if we've reached their block_start position.
    /// Applies filters to window data, writes filtered output directly to output buffer.
    #[inline]
    fn maybe_execute_filters(&mut self) {
        let total_written = self.lzss.total_written();

        // Fast path: skip if we haven't reached the next filter check position
        if total_written < self.next_filter_check {
            return;
        }

        let window_mask = self.lzss.window_mask() as usize;

        // Execute filters that are ready, in order of their block_start position
        loop {
            // Find the earliest filter that is ready to execute
            let (filter_idx, next_pos) = match self.vm.find_ready_filter(total_written) {
                Some((idx, pos)) => (idx, pos),
                None => break,
            };

            // Flush up to filter start first (unfiltered data before this filter)
            let flushed = self.lzss.flushed_pos();
            if flushed < next_pos {
                self.lzss.flush_to_output(next_pos);
            }

            // Execute the filter on the window (read-only) and get filtered output
            let window = self.lzss.window();
            if let Some((filter_end, filtered_data)) =
                self.vm
                    .execute_filter_at_index(filter_idx, window, window_mask, total_written)
            {
                // Write filtered data directly to output (bypasses window)
                self.lzss.write_filtered_to_output(filtered_data, next_pos);
                // Update next check to after this filter
                self.next_filter_check = filter_end;
            } else {
                break;
            }
        }

        // Update next_filter_check based on remaining filters
        self.next_filter_check = self.vm.next_filter_end().unwrap_or(u64::MAX);
    }

    /// Decode a block using PPMd.
    fn decode_block_ppm(&mut self, reader: &mut BitReader, max_size: u64) -> Result<()> {
        let ppm = self
            .ppm
            .as_mut()
            .ok_or(DecompressError::UnsupportedMethod(0x33))?;
        let coder = self
            .ppm_coder
            .as_mut()
            .ok_or(DecompressError::UnsupportedMethod(0x33))?;
        let esc_char = self.ppm_esc_char;

        while self.lzss.total_written() < max_size && !reader.is_eof() {
            let ch = ppm.decode_char(coder, reader).map_err(|e| {
                #[cfg(test)]
                eprintln!(
                    "PPM decode_char failed at pos {}: {}",
                    self.lzss.total_written(),
                    e
                );
                #[cfg(not(test))]
                let _ = e;
                DecompressError::InvalidHuffmanCode
            })?;

            if ch < 0 {
                // Decode error
                #[cfg(test)]
                eprintln!("PPM decode_char returned negative: {}", ch);
                return Err(DecompressError::InvalidHuffmanCode);
            }

            #[cfg(test)]
            {
                if self.lzss.total_written() < 20 {
                    eprint!("[{}:{}] ", self.lzss.total_written(), ch);
                }
            }

            if ch != esc_char {
                // Regular character
                self.lzss.write_literal(ch as u8);
            } else {
                // Escape sequence - decode control code
                let ctrl = ppm
                    .decode_char(coder, reader)
                    .map_err(|_| DecompressError::InvalidHuffmanCode)?;

                if ctrl < 0 {
                    return Err(DecompressError::InvalidHuffmanCode);
                }

                match ctrl {
                    0 => {
                        // Should not happen (NextCh starts at 0)
                        break;
                    }
                    1 => {
                        // Write escape character itself
                        self.lzss.write_literal(esc_char as u8);
                    }
                    2 => {
                        // End of PPM block
                        break;
                    }
                    3 => {
                        // VM code - read and add to VM
                        let first_byte = ppm
                            .decode_char(coder, reader)
                            .map_err(|_| DecompressError::InvalidHuffmanCode)?
                            as u8;

                        // Decode length from first byte
                        let mut length = ((first_byte & 7) + 1) as u32;
                        if length == 7 {
                            let b1 = ppm
                                .decode_char(coder, reader)
                                .map_err(|_| DecompressError::InvalidHuffmanCode)?;
                            length = (b1 as u32) + 7;
                        } else if length == 8 {
                            let b1 = ppm
                                .decode_char(coder, reader)
                                .map_err(|_| DecompressError::InvalidHuffmanCode)?;
                            let b2 = ppm
                                .decode_char(coder, reader)
                                .map_err(|_| DecompressError::InvalidHuffmanCode)?;
                            length = (b1 as u32) * 256 + (b2 as u32);
                        }

                        if length == 0 {
                            continue;
                        }

                        // Read VM code bytes
                        let mut vm_code = vec![0u8; length as usize];
                        for i in 0..length as usize {
                            let ch = ppm
                                .decode_char(coder, reader)
                                .map_err(|_| DecompressError::InvalidHuffmanCode)?;
                            vm_code[i] = ch as u8;
                        }

                        // Add to VM
                        let total_written = self.lzss.total_written();
                        let window_mask = self.lzss.window_mask();
                        self.vm
                            .add_code(first_byte, &vm_code, total_written, window_mask);

                        // Update next_filter_check when a filter is added
                        if let Some(end) = self.vm.next_filter_end() {
                            self.next_filter_check = self.next_filter_check.min(end);
                        }
                    }
                    4 => {
                        // LZ match: 3 bytes distance (MSB first), 1 byte length
                        let mut distance: u32 = 0;
                        for _ in 0..3 {
                            let ch = ppm
                                .decode_char(coder, reader)
                                .map_err(|_| DecompressError::InvalidHuffmanCode)?;
                            distance = (distance << 8) + (ch as u32);
                        }
                        let len = ppm
                            .decode_char(coder, reader)
                            .map_err(|_| DecompressError::InvalidHuffmanCode)?;

                        // Distance+2, Length+32
                        let distance = distance + 2;
                        let length = (len as u32) + 32;

                        self.lzss.copy_match(distance, length)?;
                        self.last_dist = distance;
                        self.last_len = length;
                    }
                    5 => {
                        // RLE match: 1 byte length, distance = 1
                        let len = ppm
                            .decode_char(coder, reader)
                            .map_err(|_| DecompressError::InvalidHuffmanCode)?;

                        // Length+4, Distance=1
                        let length = (len as u32) + 4;

                        self.lzss.copy_match(1, length)?;
                        self.last_dist = 1;
                        self.last_len = length;
                    }
                    _ => {
                        // Unknown control code - likely corruption
                        #[cfg(test)]
                        eprintln!("Unknown PPM control code: {}", ctrl);
                        return Err(DecompressError::InvalidHuffmanCode);
                    }
                }
            }
        }

        Ok(())
    }

    /// Reset the decoder state for a new file.
    pub fn reset(&mut self) {
        self.lzss.reset();
        self.vm.reset();
        // Keep ppm model for reuse (SubAllocator reuses buffer if same size)
        self.ppm_coder = None;
        self.ppm_esc_char = -1;
        self.old_dist = [0; 4];
        self.old_dist_ptr = 0;
        self.last_dist = 0;
        self.last_len = 0;
        self.ppm_mode = false;
        self.tables_read = false;
        self.prev_low_offset = 0;
        self.low_offset_repeat_count = 0;
        self.next_filter_check = u64::MAX;
    }

    /// Get total bytes decompressed.
    pub fn bytes_written(&self) -> u64 {
        self.lzss.total_written()
    }
}

impl Default for Rar29Decoder {
    fn default() -> Self {
        Self::new()
    }
}

// WIP: streaming decoder
/// Streaming decompressor for RAR29.
/// Allows decompressing chunks at a time.
#[allow(dead_code)]
pub struct Rar29StreamDecoder {
    decoder: Rar29Decoder,
    /// Accumulated compressed data
    input_buffer: Vec<u8>,
    /// Current position in input buffer
    input_pos: usize,
    /// Total expected unpacked size
    unpacked_size: u64,
}

#[allow(dead_code)]
impl Rar29StreamDecoder {
    /// Create a new streaming decoder.
    pub fn new(unpacked_size: u64) -> Self {
        Self {
            decoder: Rar29Decoder::new(),
            input_buffer: Vec::new(),
            input_pos: 0,
            unpacked_size,
        }
    }

    /// Feed compressed data to the decoder.
    /// Returns decompressed data available so far.
    pub fn feed(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        self.input_buffer.extend_from_slice(data);

        // Try to decompress with available data
        let result = self
            .decoder
            .decompress(&self.input_buffer[self.input_pos..], self.unpacked_size)?;

        Ok(result)
    }

    /// Check if decompression is complete.
    pub fn is_complete(&self) -> bool {
        self.decoder.bytes_written() >= self.unpacked_size
    }

    /// Get total bytes decompressed.
    pub fn bytes_written(&self) -> u64 {
        self.decoder.bytes_written()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoder_creation() {
        let decoder = Rar29Decoder::new();
        assert_eq!(decoder.bytes_written(), 0);
        assert!(!decoder.tables_read);
    }

    // More tests would require actual RAR compressed data
}
