//! Huffman decoder for RAR compression.
//!
//! RAR uses canonical Huffman codes with up to 15-bit code lengths.

use super::{BitReader, DecompressError, Result};

/// Maximum code length in bits.
pub const MAX_CODE_LENGTH: usize = 15;

/// Table sizes for RAR3/4 format.
pub const MAINCODE_SIZE: usize = 299;
pub const OFFSETCODE_SIZE: usize = 60;
pub const LOWOFFSETCODE_SIZE: usize = 17;
pub const LENGTHCODE_SIZE: usize = 28;
pub const HUFFMAN_TABLE_SIZE: usize =
    MAINCODE_SIZE + OFFSETCODE_SIZE + LOWOFFSETCODE_SIZE + LENGTHCODE_SIZE;

/// Huffman decoding table entry.
#[derive(Clone, Copy, Default)]
pub struct HuffmanEntry {
    /// Symbol value
    pub symbol: u16,
    /// Code length in bits
    pub length: u8,
}

/// Huffman decoding table.
/// Uses a lookup table for fast decoding of short codes.
pub struct HuffmanTable {
    /// Quick lookup table for codes up to QUICK_BITS
    quick_table: Vec<HuffmanEntry>,
    /// Sorted symbols for longer codes
    symbols: Vec<u16>,
    /// Code length counts
    length_counts: [u16; MAX_CODE_LENGTH + 1],
    /// First code value for each length (right-aligned, canonical)
    first_code: [u32; MAX_CODE_LENGTH + 1],
    /// First symbol index for each length (same as unrar's DecodePos)
    first_symbol: [u16; MAX_CODE_LENGTH + 1],
    /// Left-aligned upper limit for each length (unrar's DecodeLen)
    decode_len: [u32; MAX_CODE_LENGTH + 1],
}

/// Bits for quick lookup table.
const QUICK_BITS: u32 = 10;
const QUICK_SIZE: usize = 1 << QUICK_BITS;

impl HuffmanTable {
    /// Create a new Huffman table from code lengths.
    pub fn new(lengths: &[u8]) -> Result<Self> {
        let mut table = Self {
            quick_table: vec![HuffmanEntry::default(); QUICK_SIZE],
            symbols: vec![0; lengths.len()],
            length_counts: [0; MAX_CODE_LENGTH + 1],
            first_code: [0; MAX_CODE_LENGTH + 1],
            first_symbol: [0; MAX_CODE_LENGTH + 1],
            decode_len: [0; MAX_CODE_LENGTH + 1],
        };

        // Count code lengths
        for &len in lengths {
            if len > 0 && (len as usize) <= MAX_CODE_LENGTH {
                table.length_counts[len as usize] += 1;
            }
        }

        // Calculate first code for each length (canonical Huffman)
        // AND decode_len (left-aligned upper limit, like unrar)
        let mut code = 0u32;
        let mut upper_limit = 0u32;
        for i in 1..=MAX_CODE_LENGTH {
            code = (code + table.length_counts[i - 1] as u32) << 1;
            table.first_code[i] = code;
            
            // unrar's DecodeLen calculation
            upper_limit += table.length_counts[i] as u32;
            table.decode_len[i] = upper_limit << (16 - i);
            upper_limit *= 2;
        }

        // Calculate first symbol index for each length (same as unrar's DecodePos)
        let mut idx = 0u16;
        for i in 1..=MAX_CODE_LENGTH {
            table.first_symbol[i] = idx;
            idx += table.length_counts[i];
        }

        // Build symbol list sorted by code (unrar's DecodeNum)
        let mut indices = table.first_symbol;
        for (symbol, &len) in lengths.iter().enumerate() {
            if len > 0 && (len as usize) <= MAX_CODE_LENGTH {
                let i = indices[len as usize] as usize;
                if i < table.symbols.len() {
                    table.symbols[i] = symbol as u16;
                    indices[len as usize] += 1;
                }
            }
        }

        // Build quick lookup table
        for (symbol, &len) in lengths.iter().enumerate() {
            if len > 0 && len as u32 <= QUICK_BITS {
                let len = len as u32;
                // Calculate the canonical code for this symbol
                let symbol_idx = table.symbols[..table.first_symbol[len as usize + 1] as usize]
                    .iter()
                    .position(|&s| s == symbol as u16);

                if let Some(idx) = symbol_idx {
                    let code = table.first_code[len as usize] + idx as u32
                        - table.first_symbol[len as usize] as u32;

                    // Fill all table entries that start with this code
                    let fill_bits = QUICK_BITS - len;
                    let start = (code << fill_bits) as usize;
                    let count = 1 << fill_bits;

                    for j in 0..count {
                        let entry_idx = start + j;
                        if entry_idx < QUICK_SIZE {
                            table.quick_table[entry_idx] = HuffmanEntry {
                                symbol: symbol as u16,
                                length: len as u8,
                            };
                        }
                    }
                }
            }
        }

        Ok(table)
    }

    /// Rebuild the table from new code lengths, reusing existing allocations.
    pub fn rebuild(&mut self, lengths: &[u8]) -> Result<()> {
        // Resize symbols vec if needed (no fill - we overwrite valid entries)
        if self.symbols.len() != lengths.len() {
            self.symbols.resize(lengths.len(), 0);
        }

        // Reset array fields
        self.length_counts = [0; MAX_CODE_LENGTH + 1];
        self.first_code = [0; MAX_CODE_LENGTH + 1];
        self.first_symbol = [0; MAX_CODE_LENGTH + 1];
        self.decode_len = [0; MAX_CODE_LENGTH + 1];

        // Count code lengths
        for &len in lengths {
            if len > 0 && (len as usize) <= MAX_CODE_LENGTH {
                self.length_counts[len as usize] += 1;
            }
        }

        // Calculate first code for each length (canonical Huffman)
        // AND decode_len (left-aligned upper limit, like unrar)
        let mut code = 0u32;
        let mut upper_limit = 0u32;
        for i in 1..=MAX_CODE_LENGTH {
            code = (code + self.length_counts[i - 1] as u32) << 1;
            self.first_code[i] = code;
            
            // unrar's DecodeLen calculation
            upper_limit += self.length_counts[i] as u32;
            self.decode_len[i] = upper_limit << (16 - i);
            upper_limit *= 2;
        }

        // Calculate first symbol index for each length
        let mut idx = 0u16;
        for i in 1..=MAX_CODE_LENGTH {
            self.first_symbol[i] = idx;
            idx += self.length_counts[i];
        }

        // Build symbol list sorted by code
        let mut indices = self.first_symbol;
        for (symbol, &len) in lengths.iter().enumerate() {
            if len > 0 && (len as usize) <= MAX_CODE_LENGTH {
                let i = indices[len as usize] as usize;
                if i < self.symbols.len() {
                    self.symbols[i] = symbol as u16;
                    indices[len as usize] += 1;
                }
            }
        }

        // Rebuild quick lookup table - MUST clear to avoid stale entries
        // When table structure changes (e.g. from [4,4,4,...] to [1,0,0,...,1,0,...])
        // old entries that aren't overwritten would give wrong decode results
        self.quick_table.fill(HuffmanEntry::default());
        for (symbol, &len) in lengths.iter().enumerate() {
            if len > 0 && len as u32 <= QUICK_BITS {
                let len = len as u32;
                let symbol_idx = self.symbols[..self.first_symbol[len as usize + 1] as usize]
                    .iter()
                    .position(|&s| s == symbol as u16);

                if let Some(idx) = symbol_idx {
                    let code = self.first_code[len as usize] + idx as u32
                        - self.first_symbol[len as usize] as u32;
                    let fill_bits = QUICK_BITS - len;
                    let start = (code << fill_bits) as usize;
                    let count = 1 << fill_bits;

                    for j in 0..count {
                        let entry_idx = start + j;
                        if entry_idx < QUICK_SIZE {
                            self.quick_table[entry_idx] = HuffmanEntry {
                                symbol: symbol as u16,
                                length: len as u8,
                            };
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Debug: dump the canonical codes for each symbol
    #[cfg(test)]
    pub fn dump_codes(&self, name: &str, lengths: &[u8]) {
        eprintln!("=== {} Huffman codes ===", name);
        eprintln!("length_counts[1..=15]: {:?}", &self.length_counts[1..=15]);
        eprintln!("first_code[1..=15]: {:?}", &self.first_code[1..=15]);
        eprintln!("first_symbol[1..=15]: {:?}", &self.first_symbol[1..=15]);
        eprintln!("symbols: {:?}", &self.symbols);

        for (symbol, &len) in lengths.iter().enumerate() {
            if len > 0 && (len as usize) <= MAX_CODE_LENGTH {
                // Find where this symbol is in the sorted list
                let first_sym = self.first_symbol[len as usize] as usize;
                let count = self.length_counts[len as usize] as usize;
                let end = first_sym + count;

                for i in first_sym..end {
                    if i < self.symbols.len() && self.symbols[i] == symbol as u16 {
                        let code = self.first_code[len as usize]
                            + (i as u32 - self.first_symbol[len as usize] as u32);
                        // Print code in binary with proper length padding
                        let code_str: String = format!("{:0width$b}", code, width = len as usize);
                        eprintln!("  symbol {:>2}: len={}, code={}", symbol, len, code_str);
                        break;
                    }
                }
            }
        }
    }

    /// Get quick table entry for debugging
    #[cfg(test)]
    pub fn quick_table_entry(&self, index: usize) -> (u16, u8) {
        if index < self.quick_table.len() {
            let entry = &self.quick_table[index];
            (entry.symbol, entry.length)
        } else {
            (0, 0)
        }
    }

    /// Dump symbols array for debugging
    #[cfg(test)]
    pub fn dump_symbols(&self) -> Vec<u16> {
        self.symbols.clone()
    }

    /// Dump first_symbol for debugging
    #[cfg(test)]
    pub fn dump_first_symbol(&self) -> Vec<u16> {
        self.first_symbol[1..6].to_vec()
    }

    /// Dump decode_len for debugging
    #[cfg(test)]
    pub fn dump_decode_len(&self) -> Vec<u32> {
        self.decode_len[1..8].to_vec()
    }

    /// Decode a symbol from the bit reader.
    /// Uses unrar's DecodeNumber algorithm with left-aligned comparisons.
    pub fn decode(&self, reader: &mut BitReader) -> Result<u16> {
        // Get 16 bits left-aligned like unrar
        let bit_field = reader.peek_bits(16) & 0xfffe;  // Clear LSB like unrar
        
        // Debug for position around issue
        #[cfg(test)]
        let bit_pos_start = reader.bit_position();
        
        // Quick decode path - check against decode_len (upper limit)
        if bit_field < self.decode_len[QUICK_BITS as usize] {
            // Use quick table
            let code = bit_field >> (16 - QUICK_BITS);
            let entry = &self.quick_table[code as usize];
            #[cfg(test)]
            {
                if bit_pos_start >= 6268415 && bit_pos_start <= 6268425 {
                    eprintln!("  DECODE at bit {}: bit_field={:016b}, code={}, entry.symbol={}, entry.length={}, decode_len[{}]={}", 
                        bit_pos_start, bit_field, code, entry.symbol, entry.length, QUICK_BITS, self.decode_len[QUICK_BITS as usize]);
                }
            }
            if entry.length > 0 {
                reader.advance_bits(entry.length as u32);
                return Ok(entry.symbol);
            }
        }

        // Slow path: find the matching bit length using unrar's algorithm
        let mut bits = MAX_CODE_LENGTH;
        for i in (QUICK_BITS as usize + 1)..MAX_CODE_LENGTH {
            if bit_field < self.decode_len[i] {
                bits = i;
                break;
            }
        }

        // Consume the bits
        reader.advance_bits(bits as u32);

        // Calculate distance from start of this length's codes
        let dist = if bits > 0 {
            let prev_len = self.decode_len[bits - 1];
            ((bit_field - prev_len) >> (16 - bits)) as usize
        } else {
            0
        };

        // Calculate position in symbol list
        let pos = self.first_symbol[bits] as usize + dist;

        // Safety check - if position is out of bounds, return first symbol as fallback
        if pos >= self.symbols.len() {
            // This matches unrar's behavior for corrupt data
            return Ok(self.symbols.first().copied().unwrap_or(0));
        }

        Ok(self.symbols[pos])
    }
}

/// Huffman decoder that can read code lengths from the stream.
pub struct HuffmanDecoder {
    /// Main code table (literals + lengths)
    pub main_table: Option<HuffmanTable>,
    /// Distance/offset table
    pub dist_table: Option<HuffmanTable>,
    /// Low distance table
    pub low_dist_table: Option<HuffmanTable>,
    /// Length table
    pub len_table: Option<HuffmanTable>,
    /// Old length table for delta encoding (like unrar's UnpOldTable)
    old_length_table: [u8; HUFFMAN_TABLE_SIZE],
    /// New length table being built (like unrar's local Table)
    new_length_table: [u8; HUFFMAN_TABLE_SIZE],
}

impl HuffmanDecoder {
    pub fn new() -> Self {
        Self {
            main_table: None,
            dist_table: None,
            low_dist_table: None,
            len_table: None,
            old_length_table: [0; HUFFMAN_TABLE_SIZE],
            new_length_table: [0; HUFFMAN_TABLE_SIZE],
        }
    }

    /// Reset the old length table (like unrar's memset(UnpOldTable,0,...))
    pub fn reset_tables(&mut self) {
        self.old_length_table = [0; HUFFMAN_TABLE_SIZE];
    }

    /// Read code lengths from the bit stream and build tables.
    /// This matches the RAR3/4 format.
    pub fn read_tables(&mut self, reader: &mut BitReader) -> Result<()> {
        // Read reset flag - if 0, we keep previous length table
        let reset_tables = reader.read_bit()?;
        if reset_tables {
            self.old_length_table = [0; HUFFMAN_TABLE_SIZE];
        }

        #[cfg(test)]
        eprintln!(
            "reset_tables={}, bit_pos={}",
            reset_tables,
            reader.bit_position()
        );

        self.read_tables_inner(reader)
    }

    /// Read tables after header bits have been consumed.
    pub fn read_tables_after_header(&mut self, reader: &mut BitReader) -> Result<()> {
        self.read_tables_inner(reader)
    }

    /// Internal table reading.
    fn read_tables_inner(&mut self, reader: &mut BitReader) -> Result<()> {
        // Read bit lengths for the precode (20 symbols, 4 bits each)
        let mut precode_lengths = [0u8; 20];
        let mut i = 0;
        
        #[cfg(test)]
        let precode_start_bit = reader.bit_position();
        
        #[cfg(test)]
        if precode_start_bit > 100000 {
            eprintln!("precode reader state: {}", reader.debug_state());
        }
        
        while i < 20 {
            let len = reader.read_bits(4)? as u8;
            
            if len == 0x0F {
                // Special case: 0x0F could be length 15 or a zero run indicator
                let zero_count = reader.read_bits(4)? as usize;
                #[cfg(test)]
                {
                    if precode_start_bit > 100000 {
                        eprintln!("  PRECODE[{}]: len=0x0F, zero_count={}", i, zero_count);
                    }
                }
                if zero_count == 0 {
                    // ZeroCount 0 means length is actually 15
                    precode_lengths[i] = 15;
                    i += 1;
                } else {
                    // ZeroCount > 0 means insert (ZeroCount + 2) zeros
                    let fill_count = (zero_count + 2).min(20 - i);
                    #[cfg(test)]
                    {
                        if precode_start_bit > 100000 {
                            eprintln!("    filling {} zeros (i={}..{})", fill_count, i, i + fill_count);
                        }
                    }
                    for _ in 0..fill_count {
                        precode_lengths[i] = 0;
                        i += 1;
                    }
                }
                continue;
            }
            precode_lengths[i] = len;
            i += 1;
        }

        #[cfg(test)]
        eprintln!("precode at bits {}..{}: {:?}", precode_start_bit, reader.bit_position(), precode_lengths);
        
        #[cfg(test)]
        if precode_start_bit > 100000 {
            // Show old length_table values before reading new table
            eprintln!("OLD length_table[0..30]: {:?}", &self.old_length_table[0..30]);
            // Show OLD low_dist portion
            let low_offset = MAINCODE_SIZE + OFFSETCODE_SIZE;
            eprintln!("OLD length_table low_dist [{}..{}]: {:?}", 
                low_offset, low_offset + LOWOFFSETCODE_SIZE,
                &self.old_length_table[low_offset..low_offset + LOWOFFSETCODE_SIZE]);
            // Show reader state after precode
            eprintln!("After precode: {}", reader.debug_state());
            eprintln!("  peek_bytes(8): {:02x?}", reader.peek_bytes(8));
        }

        let precode_table = HuffmanTable::new(&precode_lengths)?;
        
        #[cfg(test)]
        if precode_start_bit > 100000 {
            precode_table.dump_codes("SECOND precode", &precode_lengths);
            // Show bits at start of main table reading
            eprintln!("Bits at start of main table: {:016b} (peek 16) at bit_pos {}", reader.peek_bits(16), reader.bit_position());
        }

        // Read main length table using precode
        // Like unrar: write to new_length_table, read old values from old_length_table
        i = 0;
        #[cfg(test)]
        let mut sym_count = 0;
        while i < HUFFMAN_TABLE_SIZE {
            #[cfg(test)]
            {
                if precode_start_bit > 6060000 && precode_start_bit < 6065000 {
                    // Debug EVERY symbol decode for this specific table
                    if i >= 295 && i <= 365 {
                        eprintln!("DEBUG[{}]: bit_pos={}, peek16={:016b}", 
                            i, reader.bit_position(), reader.peek_bits(16));
                    }
                }
            }
            let sym = precode_table.decode(reader)?;

            #[cfg(test)]
            {
                if sym_count < 30 {
                    eprint!("sym[{}]={} ", i, sym);
                    sym_count += 1;
                }
                // For the problematic table, show every symbol
                if precode_start_bit > 6060000 && precode_start_bit < 6065000 && i >= 295 && i <= 365 {
                    eprintln!("  sym at [{}] = {}", i, sym);
                }
            }

            if sym < 16 {
                // Add sym to old value at this position (mod 16)
                // Read from old_length_table, write to new_length_table
                let old_val = self.old_length_table[i];
                self.new_length_table[i] = (old_val + sym as u8) & 0x0F;
                #[cfg(test)]
                {
                    if precode_start_bit > 100000 && sym_count < 30 {
                        eprintln!(" old={} sym={} new={}", old_val, sym, self.new_length_table[i]);
                    }
                }
                i += 1;
            } else if sym == 16 {
                // Repeat previous length, count = 3 + 3bits
                // Read from new_length_table (previous NEW value, like unrar's Table[I-1])
                if i == 0 {
                    return Err(DecompressError::InvalidHuffmanCode);
                }
                let count = 3 + reader.read_bits(3)? as usize;
                let prev = self.new_length_table[i - 1];
                for _ in 0..count.min(HUFFMAN_TABLE_SIZE - i) {
                    self.new_length_table[i] = prev;
                    i += 1;
                }
            } else if sym == 17 {
                // Repeat previous length, count = 11 + 7bits
                if i == 0 {
                    return Err(DecompressError::InvalidHuffmanCode);
                }
                let count = 11 + reader.read_bits(7)? as usize;
                let prev = self.new_length_table[i - 1];
                for _ in 0..count.min(HUFFMAN_TABLE_SIZE - i) {
                    self.new_length_table[i] = prev;
                    i += 1;
                }
            } else if sym == 18 {
                // Insert zeros, count = 3 + 3bits
                let count_bits = reader.read_bits(3)? as usize;
                let count = 3 + count_bits;
                #[cfg(test)]
                {
                    if precode_start_bit > 100000 {
                        eprintln!(" sym18: count_bits={} count={} filling i={}..{}", count_bits, count, i, i+count);
                    }
                }
                for _ in 0..count.min(HUFFMAN_TABLE_SIZE - i) {
                    self.new_length_table[i] = 0;
                    i += 1;
                }
            } else {
                // sym == 19: Insert zeros, count = 11 + 7bits
                let count = 11 + reader.read_bits(7)? as usize;
                #[cfg(test)]
                {
                    if precode_start_bit > 6060000 && precode_start_bit < 6065000 {
                        eprintln!(" sym19: count={} filling i={}..{}", count, i, i+count);
                    }
                }
                for _ in 0..count.min(HUFFMAN_TABLE_SIZE - i) {
                    self.new_length_table[i] = 0;
                    i += 1;
                }
            }
            #[cfg(test)]
            {
                // Track when we reach low_dist region
                let low_offset = MAINCODE_SIZE + OFFSETCODE_SIZE;
                if precode_start_bit > 6060000 && precode_start_bit < 6065000 && i >= low_offset && i < low_offset + 5 {
                    eprintln!("  -> entering low_dist region at i={}, new_length_table[{}]={}", i, i, self.new_length_table[i]);
                }
            }
        }
        
        // Copy new table to old table for next table read (like unrar's memcpy)
        self.old_length_table.copy_from_slice(&self.new_length_table);
        
        #[cfg(test)]
        eprintln!();

        #[cfg(test)]
        eprintln!("new_length_table first 20: {:?}", &self.new_length_table[..20]);
        
        #[cfg(test)]
        if precode_start_bit > 100000 {
            eprintln!("SECOND TABLE new_length_table[0..50]: {:?}", &self.new_length_table[0..50]);
            // Show low_dist portion (offset 359..376)
            let low_offset = MAINCODE_SIZE + OFFSETCODE_SIZE;
            eprintln!("new_length_table low_dist portion [{}..{}]: {:?}", 
                low_offset, low_offset + LOWOFFSETCODE_SIZE, 
                &self.new_length_table[low_offset..low_offset + LOWOFFSETCODE_SIZE]);
        }

        // Build the four Huffman tables from new_length_table
        // Use rebuild() if table exists to avoid allocation
        let mut offset = 0;

        let main_lengths = &self.new_length_table[offset..offset + MAINCODE_SIZE];
        if let Some(ref mut table) = self.main_table {
            table.rebuild(main_lengths)?;
        } else {
            self.main_table = Some(HuffmanTable::new(main_lengths)?);
        }
        
        #[cfg(test)]
        {
            // Debug: print codes for symbols we care about
            let table = self.main_table.as_ref().unwrap();
            for &sym in &[45u16, 71, 75, 89, 107, 185, 196, 256, 275] {
                let len = main_lengths.get(sym as usize).copied().unwrap_or(0);
                if len > 0 {
                    // Find symbol position in sorted list
                    let first_sym = table.first_symbol[len as usize];
                    for i in first_sym..first_sym + table.length_counts[len as usize] {
                        if table.symbols[i as usize] == sym {
                            let code = table.first_code[len as usize] + (i as u32 - first_sym as u32);
                            let bp = reader.bit_position();
                            if bp > 6140000 && bp < 6290000 {
                                eprintln!("main_table at {} symbol {}: len={}, code={:0width$b}", 
                                    bp, sym, len, code, width=len as usize);
                            }
                            break;
                        }
                    }
                } else {
                    let bp = reader.bit_position();
                    if bp > 6140000 && bp < 6290000 {
                        eprintln!("main_table at {} symbol {}: NOT IN TABLE", bp, sym);
                    }
                }
            }
        }
        
        offset += MAINCODE_SIZE;

        let dist_lengths = &self.new_length_table[offset..offset + OFFSETCODE_SIZE];
        #[cfg(test)]
        {
            let bp = reader.bit_position();
            if bp > 6140000 && bp < 6145000 {
                eprintln!("dist_table at bit_pos={} FULL lengths: {:?}", bp, dist_lengths);
            }
        }
        if let Some(ref mut table) = self.dist_table {
            table.rebuild(dist_lengths)?;
        } else {
            self.dist_table = Some(HuffmanTable::new(dist_lengths)?);
        }
        offset += OFFSETCODE_SIZE;

        #[cfg(test)]
        {
            let low_lengths = &self.new_length_table[offset..offset + LOWOFFSETCODE_SIZE];
            eprintln!("low_dist_table lengths at bit_pos={}: {:?}", reader.bit_position(), low_lengths);
        }

        let low_lengths = &self.new_length_table[offset..offset + LOWOFFSETCODE_SIZE];
        if let Some(ref mut table) = self.low_dist_table {
            table.rebuild(low_lengths)?;
        } else {
            self.low_dist_table = Some(HuffmanTable::new(low_lengths)?);
        }

        #[cfg(test)]
        {
            let low_lengths = &self.new_length_table[offset..offset + LOWOFFSETCODE_SIZE];
            self.low_dist_table
                .as_ref()
                .unwrap()
                .dump_codes("low_dist", low_lengths);
            // Dump symbols array
            eprintln!("low_dist symbols: {:?}", self.low_dist_table.as_ref().unwrap().dump_symbols());
            // Dump first_symbol
            eprintln!("low_dist first_symbol[1..6]: {:?}", self.low_dist_table.as_ref().unwrap().dump_first_symbol());
            // Dump quick_table entries 512-575 (symbol 8's range)
            eprintln!("low_dist quick_table[512..520]: {:?}", 
                (512..520).map(|i| self.low_dist_table.as_ref().unwrap().quick_table_entry(i)).collect::<Vec<_>>());
            eprintln!("low_dist quick_table[568]: {:?}", 
                self.low_dist_table.as_ref().unwrap().quick_table_entry(568));
        }

        offset += LOWOFFSETCODE_SIZE;

        let len_lengths = &self.new_length_table[offset..offset + LENGTHCODE_SIZE];
        #[cfg(test)]
        {
            eprintln!("len_table lengths[0..5]: {:?}", &len_lengths[0..5]);
        }
        if let Some(ref mut table) = self.len_table {
            table.rebuild(len_lengths)?;
        } else {
            self.len_table = Some(HuffmanTable::new(len_lengths)?);
        }

        #[cfg(test)]
        eprintln!("table reading done at bit_pos={}", reader.bit_position());

        Ok(())
    }
}

impl Default for HuffmanDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_huffman_table_simple() {
        // Simple table: 2 symbols with lengths [1, 1]
        // Symbol 0 = code 0, Symbol 1 = code 1
        let lengths = [1u8, 1];
        let table = HuffmanTable::new(&lengths).unwrap();

        let data = [0b10000000]; // First bit is 1 -> symbol 1
        let mut reader = BitReader::new(&data);
        assert_eq!(table.decode(&mut reader).unwrap(), 1);
    }

    #[test]
    fn test_huffman_table_varying_lengths() {
        // Symbol 0: length 1, code 0
        // Symbol 1: length 2, code 10
        // Symbol 2: length 2, code 11
        let lengths = [1u8, 2, 2];
        let table = HuffmanTable::new(&lengths).unwrap();

        let data = [0b01011000]; // 0 (sym 0), 10 (sym 1), 11 (sym 2)
        let mut reader = BitReader::new(&data);

        assert_eq!(table.decode(&mut reader).unwrap(), 0);
        assert_eq!(table.decode(&mut reader).unwrap(), 1);
        assert_eq!(table.decode(&mut reader).unwrap(), 2);
    }
    
    #[test]
    fn test_huffman_table_low_dist_all_4bit() {
        // All symbols 0-15 have length 4, symbol 16 has length 0
        // This is the typical low_dist table pattern
        let lengths: Vec<u8> = vec![4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 0];
        let table = HuffmanTable::new(&lengths).unwrap();
        
        // For all-4-bit codes:
        // Symbol 0: code 0000
        // Symbol 1: code 0001
        // ...
        // Symbol 8: code 1000
        // ...
        // Symbol 15: code 1111
        
        // Check quick_table for index 568 (which is 1000111000 in 10 bits)
        // First 4 bits = 1000 = 8, so symbol should be 8
        let (sym, len) = table.quick_table_entry(568);
        eprintln!("quick_table[568] = (sym={}, len={})", sym, len);
        assert_eq!(sym, 8, "Symbol at quick_table[568] should be 8");
        assert_eq!(len, 4, "Length at quick_table[568] should be 4");
        
        // Check all entries from 512-575 (where first 4 bits = 1000)
        for i in 512..576 {
            let (s, l) = table.quick_table_entry(i);
            assert_eq!(s, 8, "Symbol at quick_table[{}] should be 8, got {}", i, s);
            assert_eq!(l, 4, "Length at quick_table[{}] should be 4, got {}", i, l);
        }
        
        // Test actual decoding
        // Bits 1000... should decode to symbol 8
        let data = [0b10000000, 0b00000000];
        let mut reader = BitReader::new(&data);
        assert_eq!(table.decode(&mut reader).unwrap(), 8);
        assert_eq!(reader.bit_position(), 4); // Should consume exactly 4 bits
    }
    
    #[test]
    fn test_huffman_table_rebuild_changes() {
        // Start with a table that has different lengths
        // This tests that rebuild() properly clears and rebuilds the quick_table
        let initial_lengths: Vec<u8> = vec![4, 4, 5, 4, 3, 5, 4, 4, 4, 4, 4, 5, 4, 3, 5, 4, 0];
        let mut table = HuffmanTable::new(&initial_lengths).unwrap();
        
        // Now rebuild with all 4-bit codes
        let new_lengths: Vec<u8> = vec![4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 0];
        table.rebuild(&new_lengths).unwrap();
        
        // Check quick_table for index 568
        let (sym, len) = table.quick_table_entry(568);
        eprintln!("After rebuild: quick_table[568] = (sym={}, len={})", sym, len);
        assert_eq!(sym, 8, "Symbol at quick_table[568] should be 8 after rebuild");
        assert_eq!(len, 4, "Length at quick_table[568] should be 4 after rebuild");
        
        // Check all entries from 512-575 (where first 4 bits = 1000)
        for i in 512..576 {
            let (s, l) = table.quick_table_entry(i);
            assert_eq!(s, 8, "Symbol at quick_table[{}] should be 8 after rebuild, got {}", i, s);
            assert_eq!(l, 4, "Length at quick_table[{}] should be 4 after rebuild, got {}", i, l);
        }
        
        // Test actual decoding
        let data = [0b10000000, 0b00000000];
        let mut reader = BitReader::new(&data);
        assert_eq!(table.decode(&mut reader).unwrap(), 8);
        assert_eq!(reader.bit_position(), 4);
    }
}
