//! Huffman decoder for RAR compression.
//!
//! RAR uses canonical Huffman codes with up to 15-bit code lengths.

use super::{DecompressError, Result, BitReader};

/// Maximum code length in bits.
pub const MAX_CODE_LENGTH: usize = 15;

/// Table sizes for RAR3/4 format.
pub const MAINCODE_SIZE: usize = 299;
pub const OFFSETCODE_SIZE: usize = 60;
pub const LOWOFFSETCODE_SIZE: usize = 17;
pub const LENGTHCODE_SIZE: usize = 28;
pub const HUFFMAN_TABLE_SIZE: usize = MAINCODE_SIZE + OFFSETCODE_SIZE + LOWOFFSETCODE_SIZE + LENGTHCODE_SIZE;

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
    /// First code value for each length
    first_code: [u32; MAX_CODE_LENGTH + 1],
    /// First symbol index for each length
    first_symbol: [u16; MAX_CODE_LENGTH + 1],
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
        };

        // Count code lengths
        for &len in lengths {
            if len > 0 && (len as usize) <= MAX_CODE_LENGTH {
                table.length_counts[len as usize] += 1;
            }
        }

        // Calculate first code for each length (canonical Huffman)
        let mut code = 0u32;
        for i in 1..=MAX_CODE_LENGTH {
            code = (code + table.length_counts[i - 1] as u32) << 1;
            table.first_code[i] = code;
        }

        // Calculate first symbol index for each length
        let mut idx = 0u16;
        for i in 1..=MAX_CODE_LENGTH {
            table.first_symbol[i] = idx;
            idx += table.length_counts[i];
        }

        // Build symbol list sorted by code
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

    /// Debug: dump the canonical codes for each symbol
    #[cfg(test)]
    pub fn dump_codes(&self, name: &str, lengths: &[u8]) {
        eprintln!("=== {} Huffman codes ===", name);
        eprintln!("length_counts: {:?}", &self.length_counts[1..=5]);
        eprintln!("first_code: {:?}", &self.first_code[1..=5]);
        eprintln!("first_symbol: {:?}", &self.first_symbol[1..=5]);
        eprintln!("symbols: {:?}", &self.symbols);
        
        for (symbol, &len) in lengths.iter().enumerate() {
            if len > 0 && (len as usize) <= MAX_CODE_LENGTH {
                // Find where this symbol is in the sorted list
                let first_sym = self.first_symbol[len as usize] as usize;
                let count = self.length_counts[len as usize] as usize;
                let end = first_sym + count;
                
                for i in first_sym..end {
                    if i < self.symbols.len() && self.symbols[i] == symbol as u16 {
                        let code = self.first_code[len as usize] + (i as u32 - self.first_symbol[len as usize] as u32);
                        // Print code in binary with proper length padding
                        let code_str: String = format!("{:0width$b}", code, width = len as usize);
                        eprintln!("  symbol {:>2}: len={}, code={}", symbol, len, code_str);
                        break;
                    }
                }
            }
        }
    }

    /// Decode a symbol from the bit reader.
    pub fn decode(&self, reader: &mut BitReader) -> Result<u16> {
        let bits = reader.peek_bits(QUICK_BITS);
        let entry = &self.quick_table[bits as usize];
        
        if entry.length > 0 {
            reader.advance_bits(entry.length as u32);
            return Ok(entry.symbol);
        }

        // Slow path for longer codes
        let code = reader.peek_bits(MAX_CODE_LENGTH as u32);
        
        for len in (QUICK_BITS as usize + 1)..=MAX_CODE_LENGTH {
            let shift = MAX_CODE_LENGTH - len;
            let masked = code >> shift;
            
            if masked >= self.first_code[len] {
                let count = self.length_counts[len] as u32;
                let first = self.first_code[len];
                
                if masked < first + count {
                    let idx = self.first_symbol[len] as u32 + (masked - first);
                    if (idx as usize) < self.symbols.len() {
                        reader.advance_bits(len as u32);
                        return Ok(self.symbols[idx as usize]);
                    }
                }
            }
        }

        Err(DecompressError::InvalidHuffmanCode)
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
    /// Stored length table for incremental updates
    length_table: [u8; HUFFMAN_TABLE_SIZE],
}

impl HuffmanDecoder {
    pub fn new() -> Self {
        Self {
            main_table: None,
            dist_table: None,
            low_dist_table: None,
            len_table: None,
            length_table: [0; HUFFMAN_TABLE_SIZE],
        }
    }

    /// Reset the length table.
    pub fn reset_tables(&mut self) {
        self.length_table = [0; HUFFMAN_TABLE_SIZE];
    }

    /// Read code lengths from the bit stream and build tables.
    /// This matches the RAR3/4 format.
    pub fn read_tables(&mut self, reader: &mut BitReader) -> Result<()> {
        // Read reset flag - if 0, we keep previous length table
        let reset_tables = reader.read_bit()?;
        if reset_tables {
            self.length_table = [0; HUFFMAN_TABLE_SIZE];
        }

        #[cfg(test)]
        eprintln!("reset_tables={}, bit_pos={}", reset_tables, reader.bit_position());

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
        while i < 20 {
            let len = reader.read_bits(4)? as u8;
            if len == 0x0F {
                // Special case: zero run
                let zero_count = reader.read_bits(4)? as usize;
                if zero_count > 0 {
                    for _ in 0..(zero_count + 2).min(20 - i) {
                        precode_lengths[i] = 0;
                        i += 1;
                    }
                    continue;
                }
            }
            precode_lengths[i] = len;
            i += 1;
        }

        #[cfg(test)]
        eprintln!("precode_lengths={:?}", precode_lengths);

        let precode_table = HuffmanTable::new(&precode_lengths)?;

        // Read main length table using precode
        i = 0;
        #[cfg(test)]
        let mut sym_count = 0;
        while i < HUFFMAN_TABLE_SIZE {
            let sym = precode_table.decode(reader)?;
            
            #[cfg(test)]
            {
                if sym_count < 30 {
                    eprint!("sym[{}]={} ", i, sym);
                    sym_count += 1;
                }
            }
            
            if sym < 16 {
                // Add to previous value (mod 16)
                self.length_table[i] = (self.length_table[i] + sym as u8) & 0x0F;
                i += 1;
            } else if sym == 16 {
                // Repeat previous length, count = 3 + 3bits
                if i == 0 {
                    return Err(DecompressError::InvalidHuffmanCode);
                }
                let count = 3 + reader.read_bits(3)? as usize;
                let prev = self.length_table[i - 1];
                for _ in 0..count.min(HUFFMAN_TABLE_SIZE - i) {
                    self.length_table[i] = prev;
                    i += 1;
                }
            } else if sym == 17 {
                // Repeat previous length, count = 11 + 7bits
                if i == 0 {
                    return Err(DecompressError::InvalidHuffmanCode);
                }
                let count = 11 + reader.read_bits(7)? as usize;
                let prev = self.length_table[i - 1];
                for _ in 0..count.min(HUFFMAN_TABLE_SIZE - i) {
                    self.length_table[i] = prev;
                    i += 1;
                }
            } else if sym == 18 {
                // Insert zeros, count = 3 + 3bits
                let count = 3 + reader.read_bits(3)? as usize;
                for _ in 0..count.min(HUFFMAN_TABLE_SIZE - i) {
                    self.length_table[i] = 0;
                    i += 1;
                }
            } else {
                // sym == 19: Insert zeros, count = 11 + 7bits
                let count = 11 + reader.read_bits(7)? as usize;
                for _ in 0..count.min(HUFFMAN_TABLE_SIZE - i) {
                    self.length_table[i] = 0;
                    i += 1;
                }
            }
        }
        #[cfg(test)]
        eprintln!();

        #[cfg(test)]
        eprintln!("length_table first 20: {:?}", &self.length_table[..20]);

        // Build the four Huffman tables from length_table
        let mut offset = 0;
        
        self.main_table = Some(HuffmanTable::new(&self.length_table[offset..offset + MAINCODE_SIZE])?);
        offset += MAINCODE_SIZE;
        
        self.dist_table = Some(HuffmanTable::new(&self.length_table[offset..offset + OFFSETCODE_SIZE])?);
        offset += OFFSETCODE_SIZE;
        
        #[cfg(test)]
        {
            let low_lengths = &self.length_table[offset..offset + LOWOFFSETCODE_SIZE];
            eprintln!("low_dist_table lengths: {:?}", low_lengths);
        }
        
        self.low_dist_table = Some(HuffmanTable::new(&self.length_table[offset..offset + LOWOFFSETCODE_SIZE])?);
        
        #[cfg(test)]
        {
            let low_lengths = &self.length_table[offset..offset + LOWOFFSETCODE_SIZE];
            self.low_dist_table.as_ref().unwrap().dump_codes("low_dist", low_lengths);
        }
        
        offset += LOWOFFSETCODE_SIZE;
        
        self.len_table = Some(HuffmanTable::new(&self.length_table[offset..offset + LENGTHCODE_SIZE])?);

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
}
