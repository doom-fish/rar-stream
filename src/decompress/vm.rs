//! RAR3 VM filter implementation.
//!
//! RAR3 uses a virtual machine for post-processing decompressed data.
//! In practice, only 6 standard filters are used, identified by CRC.

use crate::crc32::crc32;

/// VM memory size (256KB)
pub const VM_MEMSIZE: usize = 0x40000;
pub const VM_MEMMASK: u32 = (VM_MEMSIZE - 1) as u32;

/// Maximum channels for audio/delta filters
pub const MAX_UNPACK_CHANNELS: usize = 1024;

/// Standard filter types (identified by CRC, not bytecode)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardFilter {
    None,
    /// x86 CALL (E8) address conversion
    E8,
    /// x86 CALL/JMP (E8/E9) address conversion
    E8E9,
    /// IA-64 Itanium preprocessing
    Itanium,
    /// Byte delta encoding (audio/images)
    Delta,
    /// RGB predictive filter
    Rgb,
    /// Audio sample predictor
    Audio,
}

/// Known filter signatures
struct FilterSignature {
    length: u32,
    crc: u32,
    filter_type: StandardFilter,
}

const FILTER_SIGNATURES: &[FilterSignature] = &[
    FilterSignature {
        length: 53,
        crc: 0xad576887,
        filter_type: StandardFilter::E8,
    },
    FilterSignature {
        length: 57,
        crc: 0x3cd7e57e,
        filter_type: StandardFilter::E8E9,
    },
    FilterSignature {
        length: 120,
        crc: 0x3769893f,
        filter_type: StandardFilter::Itanium,
    },
    FilterSignature {
        length: 29,
        crc: 0x0e06077d,
        filter_type: StandardFilter::Delta,
    },
    FilterSignature {
        length: 149,
        crc: 0x1c2c5dc8,
        filter_type: StandardFilter::Rgb,
    },
    FilterSignature {
        length: 216,
        crc: 0xbc85e701,
        filter_type: StandardFilter::Audio,
    },
];

/// A prepared filter ready for execution
#[derive(Debug, Clone)]
pub struct PreparedFilter {
    pub filter_type: StandardFilter,
    /// Initial register values [R0-R6]
    pub init_r: [u32; 7],
    /// Block start position in output (absolute, for scheduling)
    pub block_start: u64,
    /// Block length
    pub block_length: u32,
    /// Window mask for indexing
    pub window_mask: u32,
}

/// Stored filter definition (reusable)
#[derive(Debug, Clone)]
pub struct StoredFilter {
    pub filter_type: StandardFilter,
}

/// RAR VM state
pub struct RarVM {
    /// VM memory buffer
    mem: Vec<u8>,
    /// Stored filter definitions (by index)
    filters: Vec<StoredFilter>,
    /// Filter execution stack
    stack: Vec<PreparedFilter>,
    /// Last used filter index
    last_filter: usize,
    /// Old filter block lengths (for reuse)
    old_lengths: Vec<u32>,
}

impl RarVM {
    pub fn new() -> Self {
        Self {
            mem: vec![0u8; VM_MEMSIZE + 4],
            filters: Vec::new(),
            stack: Vec::new(),
            last_filter: 0,
            old_lengths: Vec::new(),
        }
    }

    /// Reset VM state
    pub fn reset(&mut self) {
        self.filters.clear();
        self.stack.clear();
        self.last_filter = 0;
        self.old_lengths.clear();
    }

    /// Identify filter type from VM code using CRC
    fn identify_filter(code: &[u8]) -> StandardFilter {
        if code.is_empty() {
            return StandardFilter::None;
        }

        // Verify XOR checksum
        let mut xor_sum: u8 = 0;
        for &b in &code[1..] {
            xor_sum ^= b;
        }
        #[cfg(test)]
        let checksum_ok = xor_sum == code[0];
        
        if xor_sum != code[0] {
            #[cfg(test)]
            eprintln!("    identify_filter: XOR checksum FAILED (expected 0x{:02x}, got 0x{:02x})", code[0], xor_sum);
            return StandardFilter::None;
        }

        // Calculate CRC and match against known filters
        let code_crc = crc32(code);
        let code_len = code.len() as u32;

        #[cfg(test)]
        eprintln!("    identify_filter: len={}, crc=0x{:08x}, checksum={}", code_len, code_crc, checksum_ok);

        for sig in FILTER_SIGNATURES {
            if sig.crc == code_crc && sig.length == code_len {
                #[cfg(test)]
                eprintln!("    identified: {:?}", sig.filter_type);
                return sig.filter_type;
            }
        }

        #[cfg(test)]
        eprintln!("    identified: None (no matching signature)");
        
        StandardFilter::None
    }

    /// Read variable-length data value from bit input
    fn read_data(data: &[u8], bit_pos: &mut usize) -> u32 {
        // We need at least 2 bits to determine the type, and up to 34 bits total for case 0xc000
        // But we can handle cases where we have less data by checking as we go
        let bits_available = data.len() * 8 - *bit_pos;
        if bits_available < 2 {
            #[cfg(test)]
            eprintln!("      read_data: EOF at bit_pos={}, data.len={}", *bit_pos, data.len());
            return 0;
        }

        // Read up to 24 bits (3 bytes) for initial 16-bit value
        let byte_pos = *bit_pos / 8;
        let bit_off = *bit_pos % 8;

        // unrar reads 3 bytes and shifts
        let mut val: u32 = 0;
        if byte_pos < data.len() {
            val |= (data[byte_pos] as u32) << 16;
        }
        if byte_pos + 1 < data.len() {
            val |= (data[byte_pos + 1] as u32) << 8;
        }
        if byte_pos + 2 < data.len() {
            val |= data[byte_pos + 2] as u32;
        }
        val >>= 8 - bit_off;
        val &= 0xffff;

        #[cfg(test)]
        if *bit_pos < 100 {
            eprintln!("      read_data at bit {}: bytes[{}..{}]=[{:02x},{:02x},{:02x}], bit_off={}, val=0x{:04x}", 
                *bit_pos, byte_pos, byte_pos+3, 
                data.get(byte_pos).copied().unwrap_or(0),
                data.get(byte_pos+1).copied().unwrap_or(0),
                data.get(byte_pos+2).copied().unwrap_or(0),
                bit_off, val);
        }

        match val & 0xc000 {
            0 => {
                *bit_pos += 6;
                (val >> 10) & 0xf
            }
            0x4000 => {
                if (val & 0x3c00) == 0 {
                    *bit_pos += 14;
                    0xffffff00 | ((val >> 2) & 0xff)
                } else {
                    *bit_pos += 10;
                    (val >> 6) & 0xff
                }
            }
            0x8000 => {
                // 16-bit value follows (after 2-bit marker)
                *bit_pos += 2;
                // Read 16 bits aligned to current bit position
                let byte_pos = *bit_pos / 8;
                let bit_off = *bit_pos % 8;
                
                let mut raw: u32 = 0;
                if byte_pos < data.len() {
                    raw |= (data[byte_pos] as u32) << 16;
                }
                if byte_pos + 1 < data.len() {
                    raw |= (data[byte_pos + 1] as u32) << 8;
                }
                if byte_pos + 2 < data.len() {
                    raw |= data[byte_pos + 2] as u32;
                }
                raw >>= 8 - bit_off;
                
                *bit_pos += 16;
                raw & 0xffff
            }
            _ => {
                // 32-bit value follows (after 2-bit marker)
                *bit_pos += 2;
                // Read first 16 bits
                let byte_pos = *bit_pos / 8;
                let bit_off = *bit_pos % 8;
                
                let mut raw1: u32 = 0;
                if byte_pos < data.len() {
                    raw1 |= (data[byte_pos] as u32) << 16;
                }
                if byte_pos + 1 < data.len() {
                    raw1 |= (data[byte_pos + 1] as u32) << 8;
                }
                if byte_pos + 2 < data.len() {
                    raw1 |= data[byte_pos + 2] as u32;
                }
                raw1 >>= 8 - bit_off;
                let high16 = raw1 & 0xffff;
                
                *bit_pos += 16;
                
                // Read second 16 bits
                let byte_pos = *bit_pos / 8;
                let bit_off = *bit_pos % 8;
                
                let mut raw2: u32 = 0;
                if byte_pos < data.len() {
                    raw2 |= (data[byte_pos] as u32) << 16;
                }
                if byte_pos + 1 < data.len() {
                    raw2 |= (data[byte_pos + 1] as u32) << 8;
                }
                if byte_pos + 2 < data.len() {
                    raw2 |= data[byte_pos + 2] as u32;
                }
                raw2 >>= 8 - bit_off;
                let low16 = raw2 & 0xffff;
                
                *bit_pos += 16;
                (high16 << 16) | low16
            }
        }
    }

    /// Add VM code and create filter
    /// `total_written` is the absolute total bytes written so far (not wrapped)
    /// `window_mask` is used to wrap block_start for window access
    pub fn add_code(&mut self, first_byte: u8, code: &[u8], total_written: u64, window_mask: u32) -> bool {
        let mut bit_pos = 0;
        
        #[cfg(test)]
        eprintln!("  add_code: first_byte=0x{:02x}, code.len={}", first_byte, code.len());
        
        // Determine filter position
        let filt_pos = if (first_byte & 0x80) != 0 {
            let pos = Self::read_data(code, &mut bit_pos);
            if pos == 0 {
                // Reset filters
                self.filters.clear();
                self.old_lengths.clear();
            }
            pos.saturating_sub(1) as usize
        } else {
            self.last_filter
        };

        if filt_pos > self.filters.len() || filt_pos > 1024 {
            return false;
        }

        self.last_filter = filt_pos;
        let new_filter = filt_pos == self.filters.len();

        // Read block_start (offset from current position)
        let mut block_start = Self::read_data(code, &mut bit_pos);
        if (first_byte & 0x40) != 0 {
            block_start = block_start.wrapping_add(258);
        }
        
        // Read block_length
        let block_length = if (first_byte & 0x20) != 0 {
            let len = Self::read_data(code, &mut bit_pos);
            #[cfg(test)]
            eprintln!("    block_length read from code: {}", len);
            if filt_pos < self.old_lengths.len() {
                self.old_lengths[filt_pos] = len;
            } else if new_filter {
                // Will be pushed below
            }
            len
        } else if filt_pos < self.old_lengths.len() {
            #[cfg(test)]
            eprintln!("    block_length from old_lengths[{}]: {}", filt_pos, self.old_lengths[filt_pos]);
            self.old_lengths[filt_pos]
        } else {
            #[cfg(test)]
            eprintln!("    block_length: 0 (filt_pos {} >= old_lengths.len {})", filt_pos, self.old_lengths.len());
            0
        };

        // Compute absolute block_start (where filter should execute in output stream)
        // block_start from code is an offset from current total_written position
        let absolute_block_start = total_written + block_start as u64;

        // Read initial registers
        let mut init_r = [0u32; 7];
        init_r[3] = VM_MEMSIZE as u32;
        init_r[4] = block_length;
        init_r[5] = 0; // ExecCount
        init_r[6] = (absolute_block_start & 0xFFFFFFFF) as u32; // FileOffset - position in output (truncated to u32)

        if (first_byte & 0x10) != 0 {
            // Read 7-bit init mask like unrar: fgetbits()>>9, then faddbits(7)
            let byte_pos = bit_pos / 8;
            let bit_off = bit_pos % 8;
            
            // Read 3 bytes and form 16-bit value like getbits()
            let mut val: u32 = 0;
            if byte_pos < code.len() {
                val |= (code[byte_pos] as u32) << 16;
            }
            if byte_pos + 1 < code.len() {
                val |= (code[byte_pos + 1] as u32) << 8;
            }
            if byte_pos + 2 < code.len() {
                val |= code[byte_pos + 2] as u32;
            }
            val >>= 8 - bit_off;
            let init_mask = ((val >> 9) & 0x7f) as u8;
            bit_pos += 7;
            
            #[cfg(test)]
            eprintln!("    init_mask=0x{:02x} at bit {}", init_mask, bit_pos - 7);
            
            for i in 0..7 {
                if (init_mask & (1 << i)) != 0 {
                    init_r[i] = Self::read_data(code, &mut bit_pos);
                    #[cfg(test)]
                    eprintln!("    init_r[{}]={}", i, init_r[i]);
                }
            }
        }

        // For new filters, read VM bytecode and identify
        let filter_type = if new_filter {
            // Read VM code size
            let vm_code_size = Self::read_data(code, &mut bit_pos) as usize;
            #[cfg(test)]
            eprintln!("    new_filter: vm_code_size={}, bit_pos={}, code.len={}", vm_code_size, bit_pos, code.len());
            
            if vm_code_size == 0 || vm_code_size >= 0x10000 {
                return false;
            }
            
            // Read VM bytecode - bit aligned, reading each byte via getbits
            let mut vm_code = vec![0u8; vm_code_size];
            for i in 0..vm_code_size {
                if bit_pos + 8 > code.len() * 8 {
                    return false;
                }
                // Read 8 bits like unrar's (fgetbits()>>8)
                let byte_idx = bit_pos / 8;
                let bit_off = bit_pos % 8;
                
                let mut val: u32 = 0;
                if byte_idx < code.len() {
                    val |= (code[byte_idx] as u32) << 16;
                }
                if byte_idx + 1 < code.len() {
                    val |= (code[byte_idx + 1] as u32) << 8;
                }
                if byte_idx + 2 < code.len() {
                    val |= code[byte_idx + 2] as u32;
                }
                val >>= 8 - bit_off;
                vm_code[i] = ((val >> 8) & 0xff) as u8;
                bit_pos += 8;
            }
            
            #[cfg(test)]
            eprintln!("    vm_code first 4 bytes: {:02x} {:02x} {:02x} {:02x}", 
                vm_code.get(0).copied().unwrap_or(0),
                vm_code.get(1).copied().unwrap_or(0),
                vm_code.get(2).copied().unwrap_or(0),
                vm_code.get(3).copied().unwrap_or(0));
            
            Self::identify_filter(&vm_code)
        } else if filt_pos < self.filters.len() {
            self.filters[filt_pos].filter_type
        } else {
            StandardFilter::None
        };

        if new_filter {
            self.filters.push(StoredFilter { filter_type });
            self.old_lengths.push(block_length);
        }

        #[cfg(test)]
        eprintln!("    filter: type={:?}, block_start={} (raw {}+total_written {}), len={}", 
            filter_type, absolute_block_start, block_start, total_written, block_length);

        let filter = PreparedFilter {
            filter_type,
            init_r,
            block_start: absolute_block_start as u64,
            block_length,
            window_mask,
        };

        self.stack.push(filter);
        true
    }

    /// Check if there are pending filters
    pub fn has_pending_filters(&self) -> bool {
        !self.stack.is_empty()
    }

    /// Find the earliest filter that is ready to execute (block_end <= total_written)
    /// Returns the index and block_start of the earliest ready filter
    pub fn find_ready_filter(&self, total_written: u64) -> Option<(usize, u64)> {
        let mut earliest_idx = None;
        let mut earliest_start = u64::MAX;
        
        for (idx, filter) in self.stack.iter().enumerate() {
            let block_length = (filter.block_length & VM_MEMMASK) as u64;
            let block_end = filter.block_start + block_length;
            
            // Filter is ready if we've written past its end
            if block_end <= total_written && filter.block_start < earliest_start {
                #[cfg(test)]
                if filter.block_start < 1600000 && filter.block_start > 1400000 {
                    eprintln!("  find_ready_filter: found candidate idx={}, start={}, end={}, total_written={}", 
                        idx, filter.block_start, block_end, total_written);
                }
                earliest_start = filter.block_start;
                earliest_idx = Some(idx);
            }
        }
        
        earliest_idx.map(|idx| (idx, earliest_start))
    }

    /// Get the next filter's block start position
    pub fn next_filter_pos(&self) -> Option<u64> {
        self.stack.first().map(|f| f.block_start)
    }
    
    /// Get the earliest filter end position (block_start + block_length)
    pub fn next_filter_end(&self) -> Option<u64> {
        self.stack.iter()
            .map(|f| f.block_start + (f.block_length & VM_MEMMASK) as u64)
            .min()
    }
    
    /// Peek at the next filter without removing it
    pub fn peek_filter(&self) -> Option<&PreparedFilter> {
        self.stack.first()
    }

    /// Execute pending filters on the sliding window buffer.
    /// total_written is the absolute total bytes written so far.
    /// window is the circular sliding window (read-only!).
    /// window_mask is used for wrapping.
    /// Returns (filter_end_position, filtered_data) if a filter was executed.
    /// The filtered data should be written directly to output, NOT back to window.
    pub fn execute_filter_at_index(
        &mut self,
        filter_idx: usize,
        window: &[u8],
        window_mask: usize,
        total_written: u64,
    ) -> Option<(u64, Vec<u8>)> {
        if filter_idx >= self.stack.len() {
            return None;
        }

        // Remove and execute the filter at the specified index
        let filter = self.stack.remove(filter_idx);
        let block_start = filter.block_start;
        let block_length = (filter.block_length & VM_MEMMASK) as usize;
        let block_end = block_start + block_length as u64;

        #[cfg(test)]
        eprintln!("EXECUTE filter {:?}: start={}, len={}, total_written={}", 
            filter.filter_type, block_start, block_length, total_written);

        // Copy data from window to VM memory using bulk copy when possible
        let copy_len = block_length.min(VM_MEMSIZE);
        let window_start = (block_start as usize) & window_mask;
        
        // Check if we can do a contiguous copy (no wrap in window)
        if window_start + copy_len <= window.len() {
            self.mem[..copy_len].copy_from_slice(&window[window_start..window_start + copy_len]);
        } else {
            // Slow path: wrapping copy
            let first_part = window.len() - window_start;
            self.mem[..first_part].copy_from_slice(&window[window_start..]);
            self.mem[first_part..copy_len].copy_from_slice(&window[..copy_len - first_part]);
        }
        
        #[cfg(test)]
        {
            if block_start == 46592 {
                // Show window contents at filter start
                eprintln!("  WINDOW at filter start: window[46592..46608] = {:02x?}", 
                    (46592..46608).map(|p| window[p & window_mask]).collect::<Vec<_>>());
                // Show input bytes
                eprintln!("  INPUT to filter: mem[0..16] = {:02x?}", &self.mem[0..16]);
                eprintln!("  INPUT to filter: mem[9456..9472] = {:02x?}", &self.mem[9456..9472]);
            }
        }

        #[cfg(test)]
        {
            eprintln!("  init_r: {:?}", filter.init_r);
        }

        // Execute filter
        let (filtered_data_offset, filtered_size) = self.execute_filter(&filter, block_length);

        #[cfg(test)]
        eprintln!("  result: filtered_data_offset={}, filtered_size={}", filtered_data_offset, filtered_size);

        // Return the filtered data as a Vec - DO NOT write back to window!
        // The window must keep the original LZSS data for future match references.
        let output_size = filtered_size.max(block_length);
        let output_data = if filtered_size > 0 && filtered_size <= output_size {
            self.mem[filtered_data_offset..filtered_data_offset + filtered_size].to_vec()
        } else {
            // Filter failed or no output - return original data
            self.mem[0..block_length].to_vec()
        };

        Some((block_end, output_data))
    }

    /// Execute pending filters on the output buffer
    /// write_pos is the absolute total bytes written so far
    /// The buffer is the full output buffer, not the sliding window
    pub fn execute_filters(&mut self, buffer: &mut [u8], write_pos: u64) -> Option<(usize, usize)> {
        if self.stack.is_empty() {
            return None;
        }

        let filter = &self.stack[0];
        let block_start = filter.block_start as usize;
        let block_length = (filter.block_length & VM_MEMMASK) as usize;
        
        // Check if we've written enough data to cover this filter's range
        if block_start + block_length > buffer.len() {
            return None;
        }

        // Now safe to remove and execute
        let filter = self.stack.remove(0);

        #[cfg(test)]
        eprintln!("EXECUTE filter {:?}: start={}, len={}, buffer.len={}", 
            filter.filter_type, block_start, block_length, buffer.len());

        // Copy data to VM memory
        let copy_len = block_length.min(VM_MEMSIZE);
        self.mem[..copy_len].copy_from_slice(&buffer[block_start..block_start + copy_len]);

        #[cfg(test)]
        {
            eprintln!("  init_r: {:?}", filter.init_r);
            if block_start <= 4096 && block_start + block_length > 4096 {
                let offset = 4096 - block_start;
                eprintln!("  BEFORE buffer[4096..4104]: {:02x?}", &buffer[4096..4104.min(buffer.len())]);
            }
        }

        // Execute filter
        let (filtered_data, filtered_size) = self.execute_filter(&filter, block_length);

        #[cfg(test)]
        eprintln!("  result: filtered_data={}, filtered_size={}", filtered_data, filtered_size);

        if filtered_size > 0 && filtered_size <= block_length {
            // Copy filtered data back
            buffer[block_start..block_start + filtered_size]
                .copy_from_slice(&self.mem[filtered_data..filtered_data + filtered_size]);
            
            #[cfg(test)]
            if block_start <= 4096 && block_start + block_length > 4096 {
                eprintln!("  AFTER buffer[4096..4104]: {:02x?}", &buffer[4096..4104.min(buffer.len())]);
            }
        }

        Some((block_start, filtered_size.max(block_length)))
    }

    /// Execute a single filter
    fn execute_filter(&mut self, filter: &PreparedFilter, data_size: usize) -> (usize, usize) {
        let r = filter.init_r;

        match filter.filter_type {
            StandardFilter::None => (0, data_size),
            StandardFilter::E8 | StandardFilter::E8E9 => self.filter_e8e9(
                r[4] as usize,
                r[6],
                filter.filter_type == StandardFilter::E8E9,
            ),
            StandardFilter::Itanium => self.filter_itanium(r[4] as usize, r[6]),
            StandardFilter::Delta => self.filter_delta(r[4] as usize, r[0] as usize),
            StandardFilter::Rgb => self.filter_rgb(r[4] as usize, r[0] as usize, r[1] as usize),
            StandardFilter::Audio => self.filter_audio(r[4] as usize, r[0] as usize),
        }
    }

    /// E8/E8E9 filter - x86 CALL/JMP address conversion
    fn filter_e8e9(
        &mut self,
        data_size: usize,
        file_offset: u32,
        include_e9: bool,
    ) -> (usize, usize) {
        if !(4..=VM_MEMSIZE).contains(&data_size) {
            return (0, 0);
        }

        const FILE_SIZE: u32 = 0x1000000;
        
        // Use SIMD-accelerated search for E8/E9 bytes
        let search_end = data_size - 4;
        let mut cur_pos: usize = 0;
        
        if include_e9 {
            // E8E9: search for either 0xE8 or 0xE9
            while cur_pos < search_end {
                // Find next E8 or E9 byte using SIMD
                if let Some(offset) = memchr::memchr2(0xe8, 0xe9, &self.mem[cur_pos..search_end]) {
                    cur_pos += offset;
                    // Process the found byte
                    let addr_pos = cur_pos + 1;
                    let offset_val = addr_pos as u32 + file_offset;
                    let addr = u32::from_le_bytes([
                        self.mem[addr_pos],
                        self.mem[addr_pos + 1],
                        self.mem[addr_pos + 2],
                        self.mem[addr_pos + 3],
                    ]);
                    Self::transform_e8e9_addr(&mut self.mem[addr_pos..addr_pos + 4], addr, offset_val, FILE_SIZE);
                    cur_pos = addr_pos + 4;
                } else {
                    break;
                }
            }
        } else {
            // E8 only: search for 0xE8
            while cur_pos < search_end {
                if let Some(offset) = memchr::memchr(0xe8, &self.mem[cur_pos..search_end]) {
                    cur_pos += offset;
                    let addr_pos = cur_pos + 1;
                    let offset_val = addr_pos as u32 + file_offset;
                    let addr = u32::from_le_bytes([
                        self.mem[addr_pos],
                        self.mem[addr_pos + 1],
                        self.mem[addr_pos + 2],
                        self.mem[addr_pos + 3],
                    ]);
                    Self::transform_e8e9_addr(&mut self.mem[addr_pos..addr_pos + 4], addr, offset_val, FILE_SIZE);
                    cur_pos = addr_pos + 4;
                } else {
                    break;
                }
            }
        }

        (0, data_size)
    }
    
    /// Transform an E8/E9 address in place
    #[inline(always)]
    fn transform_e8e9_addr(dest: &mut [u8], addr: u32, offset: u32, file_size: u32) {
        if (addr & 0x80000000) != 0 {
            // addr < 0
            if (addr.wrapping_add(offset) & 0x80000000) == 0 {
                let new_addr = addr.wrapping_add(file_size);
                dest.copy_from_slice(&new_addr.to_le_bytes());
            }
        } else {
            // addr >= 0
            if (addr.wrapping_sub(file_size) & 0x80000000) != 0 {
                let new_addr = addr.wrapping_sub(offset);
                dest.copy_from_slice(&new_addr.to_le_bytes());
            }
        }
    }

    /// Itanium filter - IA-64 address conversion
    fn filter_itanium(&mut self, data_size: usize, file_offset: u32) -> (usize, usize) {
        if !(21..=VM_MEMSIZE).contains(&data_size) {
            return (0, 0);
        }

        static MASKS: [u8; 16] = [4, 4, 6, 6, 0, 0, 7, 7, 4, 4, 0, 0, 4, 4, 0, 0];

        let mut cur_pos: usize = 0;
        let mut file_off = file_offset >> 4;

        while cur_pos < data_size - 21 {
            let byte_val = (self.mem[cur_pos] & 0x1f) as i32 - 0x10;
            if byte_val >= 0 {
                let cmd_mask = MASKS[byte_val as usize];
                if cmd_mask != 0 {
                    for i in 0..=2 {
                        if (cmd_mask & (1 << i)) != 0 {
                            let start_pos = i * 41 + 5;
                            let op_type = self.itanium_get_bits(cur_pos, start_pos + 37, 4);
                            if op_type == 5 {
                                let offset = self.itanium_get_bits(cur_pos, start_pos + 13, 20);
                                self.itanium_set_bits(
                                    cur_pos,
                                    (offset.wrapping_sub(file_off)) & 0xfffff,
                                    start_pos + 13,
                                    20,
                                );
                            }
                        }
                    }
                }
            }
            cur_pos += 16;
            file_off = file_off.wrapping_add(1);
        }

        (0, data_size)
    }

    fn itanium_get_bits(&self, base: usize, bit_pos: usize, bit_count: usize) -> u32 {
        let in_addr = base + bit_pos / 8;
        let in_bit = bit_pos & 7;

        let mut bit_field: u32 = 0;
        if in_addr < self.mem.len() {
            bit_field |= self.mem[in_addr] as u32;
        }
        if in_addr + 1 < self.mem.len() {
            bit_field |= (self.mem[in_addr + 1] as u32) << 8;
        }
        if in_addr + 2 < self.mem.len() {
            bit_field |= (self.mem[in_addr + 2] as u32) << 16;
        }
        if in_addr + 3 < self.mem.len() {
            bit_field |= (self.mem[in_addr + 3] as u32) << 24;
        }

        bit_field >>= in_bit;
        bit_field & (0xffffffff >> (32 - bit_count))
    }

    fn itanium_set_bits(&mut self, base: usize, bit_field: u32, bit_pos: usize, bit_count: usize) {
        let in_addr = base + bit_pos / 8;
        let in_bit = bit_pos & 7;

        let and_mask = !(((1u32 << bit_count) - 1) << in_bit);
        let bit_field = bit_field << in_bit;

        for i in 0..4 {
            if in_addr + i < self.mem.len() {
                self.mem[in_addr + i] &= (and_mask >> (i * 8)) as u8;
                self.mem[in_addr + i] |= (bit_field >> (i * 8)) as u8;
            }
        }
    }

    /// Delta filter - byte delta encoding
    fn filter_delta(&mut self, data_size: usize, channels: usize) -> (usize, usize) {
        if data_size > VM_MEMSIZE / 2 || channels > MAX_UNPACK_CHANNELS || channels == 0 {
            return (0, 0);
        }

        let border = data_size * 2;
        let mut src_pos = 0;

        for cur_channel in 0..channels {
            let mut prev_byte: u8 = 0;
            let mut dest_pos = data_size + cur_channel;
            while dest_pos < border {
                prev_byte = prev_byte.wrapping_sub(self.mem[src_pos]);
                self.mem[dest_pos] = prev_byte;
                src_pos += 1;
                dest_pos += channels;
            }
        }

        (data_size, data_size)
    }

    /// RGB filter - predictive color filter
    fn filter_rgb(&mut self, data_size: usize, width: usize, pos_r: usize) -> (usize, usize) {
        let width = width.saturating_sub(3);
        if !(3..=VM_MEMSIZE / 2).contains(&data_size) || width > data_size || pos_r > 2 {
            return (0, 0);
        }

        const CHANNELS: usize = 3;
        let mut src_idx = 0;

        for cur_channel in 0..CHANNELS {
            let mut prev_byte: u32 = 0;

            let mut i = cur_channel;
            while i < data_size {
                let predicted = if i >= width + 3 {
                    let upper_idx = data_size + i - width;
                    let upper_byte = self.mem[upper_idx] as u32;
                    let upper_left_byte = self.mem[upper_idx - 3] as u32;

                    let mut pred = prev_byte
                        .wrapping_add(upper_byte)
                        .wrapping_sub(upper_left_byte);
                    let pa = (pred as i32 - prev_byte as i32).unsigned_abs();
                    let pb = (pred as i32 - upper_byte as i32).unsigned_abs();
                    let pc = (pred as i32 - upper_left_byte as i32).unsigned_abs();

                    if pa <= pb && pa <= pc {
                        pred = prev_byte;
                    } else if pb <= pc {
                        pred = upper_byte;
                    } else {
                        pred = upper_left_byte;
                    }
                    pred
                } else {
                    prev_byte
                };

                prev_byte = predicted.wrapping_sub(self.mem[src_idx] as u32) & 0xff;
                self.mem[data_size + i] = prev_byte as u8;
                src_idx += 1;
                i += CHANNELS;
            }
        }

        // Apply RGB correlation
        let border = data_size - 2;
        let mut i = pos_r;
        while i < border {
            let g = self.mem[data_size + i + 1];
            self.mem[data_size + i] = self.mem[data_size + i].wrapping_add(g);
            self.mem[data_size + i + 2] = self.mem[data_size + i + 2].wrapping_add(g);
            i += 3;
        }

        (data_size, data_size)
    }

    /// Audio filter - audio sample predictor
    fn filter_audio(&mut self, data_size: usize, channels: usize) -> (usize, usize) {
        if data_size > VM_MEMSIZE / 2 || channels > 128 || channels == 0 {
            return (0, 0);
        }

        let mut src_idx = 0;

        for cur_channel in 0..channels {
            let mut prev_byte: u32 = 0;
            let mut prev_delta: i32 = 0;
            let mut dif = [0u32; 7];
            let mut d1: i32 = 0;
            let mut d2: i32 = 0;
            let mut k1: i32 = 0;
            let mut k2: i32 = 0;
            let mut k3: i32 = 0;

            let mut i = cur_channel;
            let mut byte_count = 0u32;
            while i < data_size {
                let d3 = d2;
                d2 = prev_delta - d1;
                d1 = prev_delta;

                let predicted = (8i32 * prev_byte as i32 + k1 * d1 + k2 * d2 + k3 * d3) >> 3;
                let predicted = (predicted as u32) & 0xff;

                let cur_byte = self.mem[src_idx] as u32;
                src_idx += 1;

                let result = predicted.wrapping_sub(cur_byte) & 0xff;
                self.mem[data_size + i] = result as u8;
                // unrar: PrevDelta=(signed char)(Predicted-PrevByte);
                // Compute difference as unsigned, then cast to signed char
                prev_delta = (result.wrapping_sub(prev_byte) & 0xff) as u8 as i8 as i32;
                prev_byte = result;

                let d = ((cur_byte as i8) as i32) << 3;

                dif[0] = dif[0].wrapping_add(d.unsigned_abs());
                dif[1] = dif[1].wrapping_add((d - d1).unsigned_abs());
                dif[2] = dif[2].wrapping_add((d + d1).unsigned_abs());
                dif[3] = dif[3].wrapping_add((d - d2).unsigned_abs());
                dif[4] = dif[4].wrapping_add((d + d2).unsigned_abs());
                dif[5] = dif[5].wrapping_add((d - d3).unsigned_abs());
                dif[6] = dif[6].wrapping_add((d + d3).unsigned_abs());

                if (byte_count & 0x1f) == 0 {
                    let mut min_dif = dif[0];
                    let mut num_min_dif = 0;
                    dif[0] = 0;

                    for j in 1..7 {
                        if dif[j] < min_dif {
                            min_dif = dif[j];
                            num_min_dif = j;
                        }
                        dif[j] = 0;
                    }

                    match num_min_dif {
                        1 => {
                            if k1 >= -16 {
                                k1 -= 1;
                            }
                        }
                        2 => {
                            if k1 < 16 {
                                k1 += 1;
                            }
                        }
                        3 => {
                            if k2 >= -16 {
                                k2 -= 1;
                            }
                        }
                        4 => {
                            if k2 < 16 {
                                k2 += 1;
                            }
                        }
                        5 => {
                            if k3 >= -16 {
                                k3 -= 1;
                            }
                        }
                        6 => {
                            if k3 < 16 {
                                k3 += 1;
                            }
                        }
                        _ => {}
                    }
                }

                i += channels;
                byte_count += 1;
            }
        }

        (data_size, data_size)
    }
}

impl Default for RarVM {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_identification() {
        // Test that filter identification works with known CRCs
        assert_eq!(RarVM::identify_filter(&[]), StandardFilter::None);
    }

    #[test]
    fn test_delta_filter() {
        let mut vm = RarVM::new();

        // Simple delta test: 3 channels, 6 bytes
        vm.mem[0] = 10;
        vm.mem[1] = 20;
        vm.mem[2] = 30;
        vm.mem[3] = 5;
        vm.mem[4] = 10;
        vm.mem[5] = 15;

        let (offset, size) = vm.filter_delta(6, 3);
        assert_eq!(offset, 6);
        assert_eq!(size, 6);
    }

    #[test]
    fn test_e8_filter() {
        let mut vm = RarVM::new();

        // E8 filter test
        vm.mem[0] = 0xe8;
        vm.mem[1] = 0x00;
        vm.mem[2] = 0x00;
        vm.mem[3] = 0x10;
        vm.mem[4] = 0x00;

        let (offset, size) = vm.filter_e8e9(5, 0, false);
        assert_eq!(offset, 0);
        assert_eq!(size, 5);
    }
}
