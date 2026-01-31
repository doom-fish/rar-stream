//! RAR 2.9 (RAR4) decompression.
//!
//! Implements the LZSS + Huffman decompression used in RAR versions 2.x-4.x.
//! This is the most common format for scene releases.

use super::{
    bit_reader::BitReader,
    huffman::HuffmanDecoder,
    lzss::LzssDecoder,
    ppm::{PpmModel, RangeCoder},
    vm::RarVM,
    DecompressError, Result,
};

/// Number of main codes (literals + length symbols).
const MAIN_CODES: usize = 299;

/// Number of distance codes.
const DIST_CODES: usize = 60;

/// Number of low distance codes.
const LOW_DIST_CODES: usize = 17;

/// Number of length codes.
const LEN_CODES: usize = 28;

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
const DIST_BASE: [u32; 48] = [
    0, 1, 2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64, 96, 128, 192, 256, 384, 512, 768, 1024, 1536,
    2048, 3072, 4096, 6144, 8192, 12288, 16384, 24576, 32768, 49152, 65536, 98304, 131072, 196608,
    262144, 327680, 393216, 458752, 524288, 589824, 655360, 720896, 786432, 851968, 917504, 983040,
];

/// Extra bits for distance codes (48 entries for RAR3).
const DIST_EXTRA: [u8; 48] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13, 14, 14, 15, 15, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16,
];

/// RAR 2.9 decoder state.
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
}

impl Rar29Decoder {
    /// Create a new RAR29 decoder.
    pub fn new() -> Self {
        Self {
            lzss: LzssDecoder::rar29(),
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
        }
    }

    /// Decompress a block of data.
    /// Returns the decompressed data.
    pub fn decompress(&mut self, data: &[u8], unpacked_size: u64) -> Result<Vec<u8>> {
        let mut reader = BitReader::new(data);

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

        // Extract the decompressed data from the start
        let len = unpacked_size as usize;
        Ok(self.lzss.get_output(0, len))
    }

    /// Read Huffman tables from the bit stream.
    fn read_tables(&mut self, reader: &mut BitReader) -> Result<()> {
        // Align to byte boundary (like unrar)
        reader.align_to_byte();

        // Peek at the high bit to check for PPM mode
        // In unrar, this is done by peeking 16 bits and checking bit 15
        let ppm_flag = reader.peek_bits(1) != 0;

        self.ppm_mode = ppm_flag;

        if self.ppm_mode {
            // DON'T consume the PPM flag bit - it's part of the MaxOrder byte
            // Initialize PPMd model (which will read MaxOrder byte including the flag)
            let mut ppm = PpmModel::new();
            match ppm.init(reader) {
                Ok((coder, esc_char)) => {
                    self.ppm = Some(ppm);
                    self.ppm_coder = Some(coder);
                    self.ppm_esc_char = esc_char;
                    #[cfg(test)]
                    println!("PPMd initialized: esc_char={}", esc_char);
                }
                Err(_e) => {
                    #[cfg(test)]
                    println!("PPMd init failed: {}", _e);
                    return Err(DecompressError::UnsupportedMethod(0x33));
                }
            }
        } else {
            // LZ mode - check bit 1 (0x4000) for reset tables
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
            // Decode main symbol
            #[cfg(test)]
            let bit_pos_main_start = reader.bit_position();

            let symbol = {
                let main_table = self.huffman.main_table.as_ref().unwrap();
                main_table.decode(reader)?
            };

            #[cfg(test)]
            {
                let written = self.lzss.total_written();
                if written >= 0 && written < 0 {
                    let bit_pos_main_end = reader.bit_position();
                    eprintln!(
                        "\nmain decode at pos {}: sym={} (bits {}->{})",
                        written, symbol, bit_pos_main_start, bit_pos_main_end
                    );
                }
            }

            #[cfg(test)]
            {
                if symbol_count < 0 {
                    eprint!("main[{}]={} ", symbol_count, symbol);
                    symbol_count += 1;
                }
            }

            if symbol < 256 {
                // Literal byte
                #[cfg(test)]
                {
                    let written = self.lzss.total_written();
                    if written >= 0 && written < 0 {
                        eprint!("[{}:{}='{}'] ", written, symbol, symbol as u8 as char);
                    }
                }
                self.lzss.write_literal(symbol as u8);
            } else if symbol == 256 {
                // End of block / new tables
                if !reader.is_eof() {
                    // Check if we need new tables
                    let new_tables = reader.read_bit()?;
                    if new_tables {
                        self.huffman.read_tables(reader)?;
                    }
                }
                break;
            } else if symbol == 257 {
                // File continuation marker
                break;
            } else if symbol == 258 {
                // Repeat last match
                if self.last_len > 0 {
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
                    if written >= 0 && written < 0 {
                        eprintln!(
                            "[{}:old idx={},len={},dist={}]",
                            written, idx, length, distance
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
                    if written >= 0 && written < 0 {
                        eprintln!(
                            "[{}:short sym={}, idx={}, base={}, bits={}, extra={}, dist={}]",
                            written, symbol, idx, base, bits, extra, distance
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
                        if written >= 0 && written < 0 {
                            let bit_after_len = reader.bit_position();
                            eprintln!("[{}:long sym={}, len_idx={}, base={}, extra_bits={}, extra_val={}, len={}, bits {}->{}]", 
                                written, symbol, len_idx, base, extra, extra_val, base + extra_val + 3,
                                bit_before_len, bit_after_len);
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

                    let dist_table = self.huffman.dist_table.as_ref().unwrap();
                    match dist_table.decode(reader) {
                        Ok(s) => {
                            #[cfg(test)]
                            {
                                let written = self.lzss.total_written();
                                if written >= 0 && written < 0 {
                                    let bit_pos_after = reader.bit_position();
                                    eprintln!(
                                        "  decoded dist_symbol={} (bits {}->{})",
                                        s, bit_pos_before, bit_pos_after
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
                                    if written >= 0 && written < 0 {
                                        eprintln!(
                                            "    high bits: {} bits = {}, pos {}->{}",
                                            extra - 4,
                                            h,
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
                                    if written >= 0 && written < 0 {
                                        eprintln!(
                                            "    low_offset repeat: prev={}",
                                            self.prev_low_offset
                                        );
                                    }
                                }
                                self.prev_low_offset
                            } else {
                                #[cfg(test)]
                                let bit_pos_before = reader.bit_position();
                                #[cfg(test)]
                                let raw_bits_16 = reader.peek_bits(16);
                                let low_table = self.huffman.low_dist_table.as_ref().unwrap();
                                let sym = low_table.decode(reader)? as u32;
                                #[cfg(test)]
                                {
                                    let written = self.lzss.total_written();
                                    if written >= 0 && written < 0 {
                                        let bit_pos_after = reader.bit_position();
                                        eprintln!("    low_offset decode: sym={} (bits {}->{}), raw peek = {:016b}", 
                                            sym, bit_pos_before, bit_pos_after, raw_bits_16);
                                    }
                                }

                                if sym == 16 {
                                    // Repeat previous low offset 15 times
                                    self.low_offset_repeat_count = 15 - 1; // -1 because we use one now
                                    self.prev_low_offset
                                } else {
                                    self.prev_low_offset = sym;
                                    sym
                                }
                            };
                            #[cfg(test)]
                            {
                                let written = self.lzss.total_written();
                                if written >= 0 && written < 0 {
                                    eprintln!("  low_offset: dist_code={}, base={}, extra={}, high={}, low={}, dist={}", 
                                        dist_code, base, extra, high, low, base + high + low + 1);
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

                #[cfg(test)]
                {
                    let written = self.lzss.total_written();
                    if written >= 0 && written < 0 {
                        eprintln!("[{}:long len={},dist={}]", written, length, distance);
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
            let ch = ppm.decode_char(coder, reader).map_err(|_e| {
                #[cfg(test)]
                eprintln!(
                    "PPM decode_char failed at pos {}: {}",
                    self.lzss.total_written(),
                    _e
                );
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
                        self.vm.add_code(first_byte, &vm_code);
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
        self.ppm = None;
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

/// Streaming decompressor for RAR29.
/// Allows decompressing chunks at a time.
pub struct Rar29StreamDecoder {
    decoder: Rar29Decoder,
    /// Accumulated compressed data
    input_buffer: Vec<u8>,
    /// Current position in input buffer
    input_pos: usize,
    /// Total expected unpacked size
    unpacked_size: u64,
}

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
