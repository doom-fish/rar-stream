//! PPMd model for RAR decompression.
//!
//! Based on Dmitry Shkarin's PPMd implementation.

use super::super::BitReader;
use super::range_coder::{RangeCoder, SubRange};
use super::sub_alloc::SubAllocator;

/// PPMd constants.
const INT_BITS: u32 = 7;
const PERIOD_BITS: u32 = 7;
const TOT_BITS: u32 = INT_BITS + PERIOD_BITS;
const INTERVAL: u32 = 1 << INT_BITS;
const BIN_SCALE: u32 = 1 << TOT_BITS;
const MAX_FREQ: u32 = 124;
const MAX_O: usize = 64;
const INIT_ESC: u32 = 4;

/// PPMd state (symbol + frequency + successor).
#[derive(Clone, Copy, Default)]
struct State {
    symbol: u8,
    freq: u8,
    successor: u32, // Offset in sub-allocator
}

/// PPMd context.
struct Context {
    num_stats: u16,
    summ_freq: u16,
    stats: u32,  // Offset to states array
    suffix: u32, // Offset to suffix context
    // For single-stat contexts, we use OneState inline
    one_state: State,
}

/// SEE2 context for escape estimation.
#[derive(Clone, Copy)]
struct See2Context {
    summ: u16,
    shift: u8,
    count: u8,
}

impl See2Context {
    fn new(init_val: u16) -> Self {
        Self {
            summ: init_val << (PERIOD_BITS as u8 - 4),
            shift: PERIOD_BITS as u8 - 4,
            count: 4,
        }
    }

    fn get_mean(&mut self) -> u32 {
        let ret = (self.summ >> self.shift) as i16;
        self.summ = self.summ.wrapping_sub(ret as u16);
        if ret == 0 {
            1
        } else {
            ret as u32
        }
    }

    fn update(&mut self) {
        if self.shift < PERIOD_BITS as u8 {
            self.count = self.count.wrapping_sub(1);
            if self.count == 0 {
                self.summ = self.summ.wrapping_add(self.summ);
                self.count = 3 << self.shift;
                self.shift += 1;
            }
        }
    }
}

/// PPMd model.
pub struct PpmModel {
    /// Sub-allocator for contexts.
    sub_alloc: SubAllocator,
    /// Minimum context.
    min_context: u32,
    /// Medium context.
    med_context: u32,
    /// Maximum context.
    max_context: u32,
    /// Found state.
    found_state: u32,
    /// Number of masked symbols.
    num_masked: usize,
    /// Initial escape.
    init_esc: u32,
    /// Order fall.
    order_fall: i32,
    /// Maximum order.
    max_order: i32,
    /// Run length.
    run_length: i32,
    /// Initial run length.
    init_rl: i32,
    /// Character mask.
    char_mask: [u8; 256],
    /// NS2 index mapping.
    ns2_indx: [u8; 256],
    /// NS2 BS index mapping.
    ns2_bs_indx: [u8; 256],
    /// HB2 flag.
    hb2_flag: [u8; 256],
    /// Escape count.
    esc_count: u8,
    /// Previous success.
    prev_success: u8,
    /// High bits flag.
    hi_bits_flag: u8,
    /// Binary SEE contexts.
    bin_summ: [[u16; 64]; 128],
    /// SEE2 contexts.
    see2_cont: [[See2Context; 16]; 25],
    /// Dummy SEE2 context.
    dummy_see2: See2Context,
    /// Escape character.
    esc_char: i32,
    /// Debug: decode count
    debug_count: u32,
}

impl PpmModel {
    /// Create a new PPM model.
    pub fn new() -> Self {
        Self {
            sub_alloc: SubAllocator::new(1), // Start with 1MB
            min_context: 0,
            med_context: 0,
            max_context: 0,
            found_state: 0,
            num_masked: 0,
            init_esc: 0,
            order_fall: 0,
            max_order: 0,
            run_length: 0,
            init_rl: 0,
            char_mask: [0; 256],
            ns2_indx: [0; 256],
            ns2_bs_indx: [0; 256],
            hb2_flag: [0; 256],
            esc_count: 0,
            prev_success: 0,
            hi_bits_flag: 0,
            bin_summ: [[0; 64]; 128],
            see2_cont: [[See2Context::new(0); 16]; 25],
            dummy_see2: See2Context {
                summ: 0,
                shift: PERIOD_BITS as u8,
                count: 0,
            },
            esc_char: -1,

            debug_count: 0,
        }
    }

    /// Initialize the model from a byte stream. Returns (RangeCoder, esc_char).
    pub fn init(&mut self, reader: &mut BitReader) -> Result<(RangeCoder, i32), &'static str> {
        let max_order_byte = reader.read_byte().ok_or("EOF reading max order")?;
        let reset = (max_order_byte & 0x20) != 0;

        #[cfg(test)]
        eprintln!(
            "[PPM init] max_order_byte=0x{:02x} reset={}",
            max_order_byte, reset
        );

        // If reset flag is set, or if we haven't initialized yet, we need to initialize
        let need_init = reset || self.min_context == 0;

        let max_mb = if reset {
            reader.read_byte().ok_or("EOF reading max MB")? as usize
        } else {
            1 // Default
        };

        if (max_order_byte & 0x40) != 0 {
            self.esc_char = reader.read_byte().ok_or("EOF reading esc char")? as i32;
        }

        // Initialize range coder
        let coder = RangeCoder::new(reader);

        if need_init {
            let mut max_order = (max_order_byte & 0x1f) as i32 + 1;
            if max_order > 16 {
                max_order = 16 + (max_order - 16) * 3;
            }

            #[cfg(test)]
            eprintln!("[PPM init] max_order={} max_mb={}", max_order, max_mb);

            if max_order == 1 {
                return Err("Invalid max order");
            }

            // Resize sub-allocator (reuses buffer if size matches)
            self.sub_alloc.resize(max_mb + 1);
            self.start_model(max_order);
        }

        if self.min_context == 0 {
            return Err("Model initialization failed");
        }

        Ok((coder, self.esc_char))
    }

    /// Start/restart the model.
    fn start_model(&mut self, max_order: i32) {
        self.max_order = max_order;
        self.esc_count = 1;
        self.restart_model();

        // Initialize NS2 index tables
        self.ns2_bs_indx[0] = 0;
        self.ns2_bs_indx[1] = 2;
        for i in 2..11 {
            self.ns2_bs_indx[i] = 4;
        }
        for i in 11..256 {
            self.ns2_bs_indx[i] = 6;
        }

        for i in 0..3 {
            self.ns2_indx[i] = i as u8;
        }
        let mut m = 3u8;
        let mut k = 1usize;
        let mut step = 1usize;
        for i in 3..256 {
            self.ns2_indx[i] = m;
            k = k.saturating_sub(1);
            if k == 0 {
                step += 1;
                k = step;
                m += 1;
            }
        }

        for i in 0..0x40 {
            self.hb2_flag[i] = 0;
        }
        for i in 0x40..0x100 {
            self.hb2_flag[i] = 0x08;
        }

        self.dummy_see2.shift = PERIOD_BITS as u8;
    }

    /// Restart the model (clear and reinitialize).
    fn restart_model(&mut self) {
        self.char_mask = [0; 256];
        self.sub_alloc.init();

        self.init_rl = -(if self.max_order < 12 {
            self.max_order
        } else {
            12
        }) - 1;

        // Allocate root context
        let ctx = self.sub_alloc.alloc_context().unwrap_or(0);
        self.min_context = ctx as u32;
        self.max_context = ctx as u32;

        if ctx == 0 {
            return;
        }

        // Initialize root context with 256 symbols
        self.write_context_num_stats(ctx, 256);
        self.write_context_summ_freq(ctx, 257);

        let stats = self.sub_alloc.alloc_units(128).unwrap_or(0);
        self.write_context_stats(ctx, stats as u32);

        // Verify it was written correctly

        {
            let _read_back = self.read_context_stats(ctx);
        }
        self.write_context_stats(ctx, stats as u32);

        self.order_fall = self.max_order;
        self.found_state = stats as u32;

        // Initialize all 256 symbols with freq=1
        for i in 0..256 {
            self.write_state(stats + i * 6, i as u8, 1, 0);
        }

        // Verify initialization
        #[cfg(test)]
        {
            for i in 0..256 {
                let sym = self.read_state_symbol(stats + i * 6);
                if sym != i as u8 {
                    eprintln!(
                        "[INIT] ERROR: stats[{}] has sym {} instead of {}",
                        i, sym, i
                    );
                }
            }
        }

        self.run_length = self.init_rl;
        self.prev_success = 0;

        // Initialize binary SEE contexts
        let init_bin_esc: [u16; 8] = [
            0x3CDD, 0x1F3F, 0x59BF, 0x48F3, 0x64A1, 0x5ABC, 0x6632, 0x6051,
        ];

        for i in 0..128 {
            for k in 0..8 {
                for m in (0..64).step_by(8) {
                    self.bin_summ[i][k + m] =
                        (BIN_SCALE as u16).wrapping_sub(init_bin_esc[k] / (i as u16 + 2));
                }
            }
        }

        // Initialize SEE2 contexts
        for i in 0..25 {
            for k in 0..16 {
                self.see2_cont[i][k] = See2Context::new((5 * i + 10) as u16);
            }
        }
    }

    /// Decode a character.
    pub fn decode_char(
        &mut self,
        coder: &mut RangeCoder,
        reader: &mut BitReader,
    ) -> Result<i32, &'static str> {
        #[cfg(test)]
        {
            self.debug_count += 1;
        }

        #[cfg(test)]
        let start_bytes = reader.byte_position();

        #[cfg(test)]
        if self.debug_count == 0 {
            let (code, low, range) = coder.debug_state();
            eprintln!(
                "[ENTRY pos={}] low={} range={} code={} code-low={} prev_success={}",
                self.debug_count,
                low,
                range,
                code,
                code.wrapping_sub(low),
                self.prev_success
            );
        }

        // Check context validity
        let text_ptr = self.sub_alloc.text_ptr();
        let heap_end = self.sub_alloc.heap_end();

        if self.min_context as usize <= text_ptr || self.min_context as usize > heap_end {
            return Err("Invalid context");
        }

        let num_stats = self.read_context_num_stats(self.min_context as usize);

        #[cfg(test)]
        if self.debug_count == 0 {
            let (code, low, range) = coder.debug_state();
            eprintln!(
                "[ENTRY pos={}] low={} range={} code={} NumStats={} ctx={}",
                self.debug_count, low, range, code, num_stats, self.min_context
            );
        }

        #[cfg(test)]
        if self.debug_count == 0 {
            let summ = self.read_context_summ_freq(self.min_context as usize);
            eprintln!(
                "[pos={}] min_context={} NumStats={} SummFreq={} order_fall={}",
                self.debug_count, self.min_context, num_stats, summ, self.order_fall
            );
        }

        if num_stats != 1 {
            // Multi-symbol context
            let stats = self.read_context_stats(self.min_context as usize);
            if stats as usize <= text_ptr || stats as usize > heap_end {
                #[cfg(test)]
                eprintln!(
                    "[pos={}] INVALID STATS: ctx={} num_stats={} stats={} text_ptr={} heap_end={}",
                    self.debug_count, self.min_context, num_stats, stats, text_ptr, heap_end
                );
                return Err("Invalid stats pointer");
            }
            #[cfg(test)]
            if self.debug_count == 0 {
                eprintln!(
                    "[pos={}] Multi-symbol context at {}, num_stats={}",
                    self.debug_count, self.min_context, num_stats
                );
            }
            self.decode_symbol1(coder, reader)?;
        } else {
            // Binary context
            #[cfg(test)]
            if self.debug_count == 0 {
                // OneState symbol is at context+2
                let sym = self.sub_alloc.read_byte(self.min_context as usize + 2);
                let suffix = self.read_context_suffix(self.min_context as usize);
                eprintln!("[pos={}] Binary context at {}, sym='{}' ({}), suffix={}, order_fall={}, max_order={}", 
                         self.debug_count, self.min_context, sym as char, sym, suffix, self.order_fall, self.max_order);
            }
            self.decode_bin_symbol(coder, reader)?;
        }

        // Normalize is called in the escape loop or at the end of decode_char
        // Not here after a successful decode

        while self.found_state == 0 {
            coder.normalize(reader);
            #[cfg(test)]
            if self.debug_count == 0 {
                let (code, low, range) = coder.debug_state();
                eprintln!(
                    "[ESCAPE pos={}] After normalize: low={} range={} code={}",
                    self.debug_count, low, range, code
                );
            }

            // Walk up suffix chain
            loop {
                self.order_fall += 1;
                let suffix = self.read_context_suffix(self.min_context as usize);

                #[cfg(test)]
                if self.debug_count == 0 {
                    eprintln!(
                        "[ESCAPE pos={}] order_fall={} min_context={} suffix={}",
                        self.debug_count, self.order_fall, self.min_context, suffix
                    );
                }

                if suffix as usize <= text_ptr || suffix as usize > heap_end {
                    #[cfg(test)]
                    eprintln!(
                        "[ESCAPE pos={}] Invalid suffix={} (text_ptr={} heap_end={})",
                        self.debug_count, suffix, text_ptr, heap_end
                    );
                    return Err("Invalid suffix");
                }

                self.min_context = suffix;

                let ns = self.read_context_num_stats(suffix as usize);
                if ns as usize != self.num_masked {
                    #[cfg(test)]
                    if self.debug_count == 0 {
                        eprintln!(
                            "[ESCAPE pos={}] Found context with ns={} (masked={})",
                            self.debug_count, ns, self.num_masked
                        );
                    }
                    break;
                }
            }

            self.decode_symbol2(coder, reader)?;
            // No normalize here - unrar doesn't normalize after decodeSymbol2 inside the while loop
        }

        // Get the decoded symbol
        let symbol = self.read_state_symbol(self.found_state as usize);

        // Update model
        let successor = self.read_state_successor(self.found_state as usize);
        if self.order_fall == 0 && successor as usize > text_ptr {
            let succ = successor;
            self.min_context = succ;
            self.max_context = succ;
        } else {
            self.update_model();
            if self.esc_count == 0 {
                self.clear_mask();
            }
        }

        coder.normalize(reader);

        #[cfg(test)]
        {
            let end_bytes = reader.byte_position();
            let bytes_consumed = end_bytes - start_bytes;
            if self.debug_count == 0 || (self.debug_count >= 1120 && self.debug_count <= 1135) {
                eprintln!(
                    "[pos={}] sym='{}' ({}) bytes_consumed={} found_state={}",
                    self.debug_count, symbol as char, symbol, bytes_consumed, self.found_state
                );
            }
        }

        Ok(symbol as i32)
    }

    /// Decode from a multi-symbol context.
    fn decode_symbol1(
        &mut self,
        coder: &mut RangeCoder,
        _reader: &mut BitReader,
    ) -> Result<(), &'static str> {
        let summ_freq = self.read_context_summ_freq(self.min_context as usize);
        let stats = self.read_context_stats(self.min_context as usize);
        let num_stats = self.read_context_num_stats(self.min_context as usize);

        #[cfg(test)]
        if self.debug_count == 0 {
            eprintln!(
                "[DS1 pos={}] summ_freq={} num_stats={} prev_success_before={}",
                self.debug_count, summ_freq, num_stats, self.prev_success
            );
        }

        let count = coder.get_current_count(summ_freq as u32);

        #[cfg(test)]
        if self.debug_count == 0 {
            eprintln!("[DS1 pos={}] count={}", self.debug_count, count);
        }

        // Check for out-of-range count
        if count >= summ_freq as u32 {
            return Err("Count exceeds scale");
        }

        let mut hi_cnt = 0u32;

        for i in 0..num_stats {
            let state_ptr = stats as usize + (i as usize) * 6;
            let freq = self.read_state_freq(state_ptr) as u32;
            #[cfg(test)]
            let sym = self.read_state_symbol(state_ptr);
            hi_cnt += freq;

            #[cfg(test)]
            if self.debug_count == 0 {
                eprintln!(
                    "[DS1 pos={}] i={} sym='{}' ({}) freq={} hi_cnt={}",
                    self.debug_count, i, sym as char, sym, freq, hi_cnt
                );
            }

            if hi_cnt > count {
                let lo_cnt = hi_cnt - freq;

                #[cfg(test)]
                if self.debug_count == 0 {
                    eprintln!(
                        "[DS1 pos={}] Selected i={} sym='{}' ({}) lo={} hi={}",
                        self.debug_count, i, sym as char, sym, lo_cnt, hi_cnt
                    );
                    let (code, low, range) = coder.debug_state();
                    eprintln!(
                        "[DS1 pos={}] BEFORE decode: low={} range={} code={}",
                        self.debug_count, low, range, code
                    );
                }
                let sub = SubRange {
                    low_count: lo_cnt,
                    high_count: hi_cnt,
                    scale: summ_freq as u32,
                };
                coder.decode(&sub);

                #[cfg(test)]
                if self.debug_count == 0 {
                    let (code, low, range) = coder.debug_state();
                    eprintln!(
                        "[DS1 pos={}] AFTER decode: low={:#x} range={:#x} code={:#x}",
                        self.debug_count, low, range, code
                    );
                }

                // Calculate prev_success BEFORE updating frequencies (match unrar)
                // IMPORTANT: PrevSuccess is only calculated for FIRST symbol (i==0)
                // For other symbols, PrevSuccess = 0
                if i == 0 {
                    self.prev_success = if 2 * freq > summ_freq as u32 { 1 } else { 0 };
                    self.run_length += self.prev_success as i32;
                } else {
                    self.prev_success = 0;
                }

                // Update frequency and check for rescale
                let hi_cnt = freq + 4;
                self.write_state_freq(state_ptr, hi_cnt as u8);

                // Update summ_freq
                let new_summ = summ_freq.saturating_add(4);
                self.write_context_summ_freq(self.min_context as usize, new_summ);

                // Swap with previous state if this one has higher frequency (move-to-front)
                // This matches unrar's update1() behavior
                if i > 0 {
                    let prev_ptr = stats as usize + ((i - 1) as usize) * 6;
                    let prev_freq = self.read_state_freq(prev_ptr);
                    if hi_cnt as u8 > prev_freq {
                        // Swap the two states (6 bytes each)
                        let cur_sym = self.read_state_symbol(state_ptr);
                        let cur_succ = self.read_state_successor(state_ptr);
                        let prev_sym = self.read_state_symbol(prev_ptr);
                        let prev_succ = self.read_state_successor(prev_ptr);

                        self.write_state(prev_ptr, cur_sym, hi_cnt as u8, cur_succ);
                        self.write_state(state_ptr, prev_sym, prev_freq, prev_succ);

                        self.found_state = prev_ptr as u32;

                        // Check if rescale needed
                        if hi_cnt > MAX_FREQ {
                            self.rescale();
                        }
                    } else {
                        self.found_state = state_ptr as u32;
                        if hi_cnt > MAX_FREQ {
                            self.rescale();
                        }
                    }
                } else {
                    self.found_state = state_ptr as u32;
                    if hi_cnt > MAX_FREQ {
                        self.rescale();
                    }
                }

                return Ok(());
            }
        }

        // Escape
        #[cfg(test)]
        if self.debug_count == 0 {
            eprintln!(
                "[DS1 pos={} ESCAPE] hi_cnt={} summ_freq={} before decode",
                self.debug_count, hi_cnt, summ_freq
            );
        }

        // Set PrevSuccess = 0 on escape (matching unrar line 467)
        self.prev_success = 0;

        // Set HiBitsFlag based on previous FoundState's symbol (matching unrar's line 448)
        if self.found_state != 0 {
            let prev_sym = self.read_state_symbol(self.found_state as usize);
            self.hi_bits_flag = self.hb2_flag[prev_sym as usize];
        }

        let sub = SubRange {
            low_count: hi_cnt,
            high_count: summ_freq as u32,
            scale: summ_freq as u32,
        };
        coder.decode(&sub);

        #[cfg(test)]
        if self.debug_count == 0 {
            let (code, low, range) = coder.debug_state();
            eprintln!(
                "[DS1 pos=98 ESCAPE] After decode: low={} range={} code={}",
                low, range, code
            );
        }

        self.num_masked = num_stats as usize;
        self.found_state = 0;

        // Set masks - mark all symbols in this context as masked
        // NOTE: Do NOT increment esc_count here - that happens in update2() after decodeSymbol2 finds a symbol
        for i in 0..num_stats {
            let state_ptr = stats as usize + (i as usize) * 6;
            let sym = self.read_state_symbol(state_ptr);
            self.char_mask[sym as usize] = self.esc_count;
        }

        Ok(())
    }

    /// Decode from a binary context.
    fn decode_bin_symbol(
        &mut self,
        coder: &mut RangeCoder,
        _reader: &mut BitReader,
    ) -> Result<(), &'static str> {
        let state = self.read_context_one_state(self.min_context as usize);

        // Update HiBitsFlag based on previous FoundState's symbol (set at start of decode)
        if self.found_state != 0 {
            let prev_sym = self.read_state_symbol(self.found_state as usize);
            self.hi_bits_flag = self.hb2_flag[prev_sym as usize];
        }

        // Get binary probability - match unrar's index calculation exactly
        let suffix = self.read_context_suffix(self.min_context as usize);
        let suffix_num_stats = if suffix != 0 {
            self.read_context_num_stats(suffix as usize)
        } else {
            1 // Default if no suffix
        };

        // Use NS2BSIndx (not ns2_indx) with NumStats-1
        let ns_idx = if suffix_num_stats > 0 {
            suffix_num_stats - 1
        } else {
            0
        };
        let ns1 = self.ns2_bs_indx[ns_idx as usize] as usize;

        // Index calculation matching unrar:
        // PrevSuccess + NS2BSIndx[Suffix->NumStats-1] + HiBitsFlag + 2*HB2Flag[rs.Symbol] + ((RunLength >> 26) & 0x20)
        let idx1 = (self.prev_success as usize)
            + ns1
            + self.hi_bits_flag as usize
            + 2 * (self.hb2_flag[state.symbol as usize] as usize)
            + ((self.run_length >> 26) & 0x20) as usize;

        // BinSumm first index is Freq-1 (not freq>>2)
        let freq_idx = if state.freq > 0 {
            (state.freq - 1) as usize
        } else {
            0
        };
        let freq_idx = freq_idx.min(127); // BinSumm is [128][64]
        let idx1 = idx1.min(63);

        let bs = self.bin_summ[freq_idx][idx1];

        #[cfg(test)]
        if self.debug_count == 0 {
            eprintln!("[BIN pos={} idx] prev_success={} ns1={} hi_bits_flag={} hb2_flag[{}]={} run_length={}", 
                     self.debug_count, self.prev_success, ns1, self.hi_bits_flag, state.symbol, self.hb2_flag[state.symbol as usize], self.run_length);
            eprintln!(
                "[BIN pos={} idx] idx1={} freq_idx={} bs={}",
                self.debug_count, idx1, freq_idx, bs
            );
        }

        let count = coder.get_current_shift_count(TOT_BITS);

        #[cfg(test)]
        if self.debug_count == 0 {
            eprintln!(
                "[BIN pos={}] sym='{}' ({}) freq={} bs={} count={}",
                self.debug_count, state.symbol as char, state.symbol, state.freq, bs, count
            );
            let (code, low, range) = coder.debug_state();
            eprintln!(
                "[BIN pos={}] after get_count: low={} range={} code={}",
                self.debug_count, low, range, code
            );
        }

        if count < bs as u32 {
            // Symbol found
            let sub = SubRange {
                low_count: 0,
                high_count: bs as u32,
                scale: BIN_SCALE,
            };

            #[cfg(test)]
            if self.debug_count == 0 {
                let (code, low, range) = coder.debug_state();
                eprintln!("[BIN pos={}] FOUND: lo=0 hi={} scale={} | before decode: low={} range={} code={}", 
                         self.debug_count, bs, BIN_SCALE, low, range, code);
            }

            coder.decode(&sub);

            // Update frequency
            let new_freq = state.freq + (if state.freq < 128 { 1 } else { 0 });
            self.write_context_one_state_freq(self.min_context as usize, new_freq);

            // Update bin_summ: bs + INTERVAL - GET_MEAN(bs, PERIOD_BITS, 2)
            // GET_MEAN(SUMM,SHIFT,ROUND) = ((SUMM+(1 << (SHIFT-ROUND))) >> SHIFT)
            let mean = ((bs as u32 + (1 << (PERIOD_BITS - 2))) >> PERIOD_BITS) as u16;
            let new_bs = bs.saturating_add((INTERVAL as u16).saturating_sub(mean));
            self.bin_summ[freq_idx][idx1] = new_bs;

            self.found_state = self.min_context + 2; // OneState offset (in union at offset 2)
            self.prev_success = 1;
            self.run_length += 1;
        } else {
            // Escape
            let sub = SubRange {
                low_count: bs as u32,
                high_count: BIN_SCALE,
                scale: BIN_SCALE,
            };

            #[cfg(test)]
            if self.debug_count == 0 {
                let (code, low, range) = coder.debug_state();
                eprintln!("[BIN pos={}] ESCAPE: lo={} hi={} scale={} | before decode: low={} range={} code={}", 
                         self.debug_count, bs, BIN_SCALE, BIN_SCALE, low, range, code);
            }

            coder.decode(&sub);

            // Update bin_summ: bs - GET_MEAN(bs, PERIOD_BITS, 2)
            let mean = ((bs as u32 + (1 << (PERIOD_BITS - 2))) >> PERIOD_BITS) as u16;
            let new_bs = bs.saturating_sub(mean);
            self.bin_summ[freq_idx][idx1] = new_bs;

            // InitEsc = ExpEscape[bs >> 10]
            static EXP_ESCAPE: [u8; 16] = [25, 14, 9, 7, 5, 5, 4, 4, 4, 3, 3, 3, 2, 2, 2, 2];
            self.init_esc = EXP_ESCAPE[(new_bs >> 10) as usize] as u32;

            self.num_masked = 1;
            self.found_state = 0;
            self.char_mask[state.symbol as usize] = self.esc_count;
            // Don't increment esc_count here - it's done in update2 after successful decode
            self.prev_success = 0;
        }

        Ok(())
    }

    /// Decode from a masked context.
    fn decode_symbol2(
        &mut self,
        coder: &mut RangeCoder,
        _reader: &mut BitReader,
    ) -> Result<(), &'static str> {
        #[cfg(test)]
        if self.debug_count == 0 {
            let (code, low, range) = coder.debug_state();
            eprintln!(
                "[DS2 pos={} entry] coder: low={:010} range={:010} code={:010}",
                self.debug_count, low, range, code
            );
        }

        let num_stats = self.read_context_num_stats(self.min_context as usize);
        let stats = self.read_context_stats(self.min_context as usize);

        #[cfg(test)]
        if self.debug_count == 0 || self.debug_count == 58 {
            eprintln!(
                "[DS2 pos={}] min_context={} num_stats={} num_masked={}",
                self.debug_count, self.min_context, num_stats, self.num_masked
            );
        }

        // Calculate i = NumStats - NumMasked (number of unmasked symbols)
        let i = num_stats as usize - self.num_masked;
        if i == 0 {
            return Err("All symbols masked");
        }

        // Calculate escape frequency using SEE2 or simplified for root
        let esc_freq: u32;
        let see2_row: usize;
        let see2_col: usize;
        let is_root = num_stats == 256;

        if !is_root {
            // Use SEE2 - simplified version using NS2Indx
            let ns2_idx = self.ns2_indx[(i - 1).min(255)] as usize;
            let suffix = self.read_context_suffix(self.min_context as usize);
            let suffix_num_stats = if suffix != 0 {
                self.read_context_num_stats(suffix as usize)
            } else {
                num_stats
            };
            let summ_freq = self.read_context_summ_freq(self.min_context as usize);

            // Index into SEE2Cont
            let diff_suffix = (i < (suffix_num_stats as usize - num_stats as usize)) as usize;
            let freq_check = (summ_freq < 11 * num_stats) as usize;
            let masked_check = (self.num_masked > i) as usize;
            let see2_idx = ns2_idx
                + diff_suffix
                + 2 * freq_check
                + 4 * masked_check
                + self.hi_bits_flag as usize;
            let see2_idx = see2_idx.min(24 * 16 - 1);

            see2_row = ns2_idx.min(24);
            see2_col = see2_idx % 16;

            // Use get_mean() which decrements Summ (matching unrar's getMean behavior)
            esc_freq = self.see2_cont[see2_row][see2_col].get_mean();

            #[cfg(test)]
            if self.debug_count == 58 || self.debug_count == 0 {
                let suffix = self.read_context_suffix(self.min_context as usize);
                let suffix_ns = if suffix != 0 {
                    self.read_context_num_stats(suffix as usize)
                } else {
                    0
                };
                eprintln!("[DS2 pos={}] SEE2 inputs: i={} suffix_ns={} summ_freq={} num_masked={} hi_bits_flag={}",
                         self.debug_count, i, suffix_ns, summ_freq, self.num_masked, self.hi_bits_flag);
                eprintln!("[DS2 pos={}] SEE2 indices: ns2_idx={} diff_suffix={} freq_check={} masked_check={}", 
                         self.debug_count, ns2_idx, diff_suffix, freq_check, masked_check);
                eprintln!(
                    "[DS2 pos={}] SEE2: see2_idx={} esc_freq={}",
                    self.debug_count, see2_idx, esc_freq
                );
            }
        } else {
            // Root context uses scale=1 (escape freq = 1)
            esc_freq = 1;
            see2_row = 0;
            see2_col = 0;
        }

        // First pass: count unmasked symbol frequencies (avoid 3KB array allocation)
        let mut hi_cnt = 0u32;

        for j in 0..num_stats {
            let state_ptr = stats as usize + (j as usize) * 6;
            let sym = self.read_state_symbol(state_ptr);
            if self.char_mask[sym as usize] != self.esc_count {
                let freq = self.read_state_freq(state_ptr) as u32;
                hi_cnt += freq;
            }
        }

        #[cfg(test)]
        let _freq_histogram = [0u32; 256];

        #[cfg(test)]
        if self.debug_count == 15 {
            eprintln!(
                "[DS2 pos=15] stats pointer={}, root context={}",
                stats, self.min_context
            );

            // Find symbol 'l' (108) and 'p' (112)
            let mut cum = 0u32;
            for j in 0..num_stats {
                let state_ptr = stats as usize + (j as usize) * 6;
                let sym = self.read_state_symbol(state_ptr);
                if self.char_mask[sym as usize] != self.esc_count {
                    let freq = self.read_state_freq(state_ptr) as u32;
                    let prev_cum = cum;
                    cum += freq;
                    if sym == 108 {
                        // 'l'
                        eprintln!(
                            "[DS2 pos=15] 'l' at j={} prev_cum={} cum={} freq={}",
                            j, prev_cum, cum, freq
                        );
                    }
                    if sym == 112 {
                        // 'p'
                        eprintln!(
                            "[DS2 pos=15] 'p' at j={} prev_cum={} cum={} freq={}",
                            j, prev_cum, cum, freq
                        );
                    }
                    // Trace symbols with freq > 1
                    if freq > 1 {
                        eprintln!("[DS2 pos=15] sym='{}' ({}) j={} prev_cum={} cum={} freq={} state_ptr={}", 
                                 sym as char, sym, j, prev_cum, cum, freq, state_ptr);
                    }
                }
            }
        }

        #[cfg(test)]
        if self.debug_count == 13 {
            // Find symbol 'd' (100)
            let mut cum = 0u32;
            for j in 0..num_stats {
                let state_ptr = stats as usize + (j as usize) * 6;
                let sym = self.read_state_symbol(state_ptr);
                if self.char_mask[sym as usize] != self.esc_count {
                    let freq = self.read_state_freq(state_ptr) as u32;
                    let prev_cum = cum;
                    cum += freq;
                    if sym == 100 {
                        // 'd'
                        eprintln!(
                            "[DS2 pos=13] 'd' at j={} prev_cum={} cum={} freq={}",
                            j, prev_cum, cum, freq
                        );
                    }
                    // Also trace which symbol is at cumulative 238-239
                    if prev_cum <= 238 && cum > 238 {
                        eprintln!(
                            "[DS2 pos=13] At cum=238: j={} sym='{}' ({}) prev_cum={} cum={}",
                            j, sym as char, sym, prev_cum, cum
                        );
                    }
                }
            }
            eprintln!("[DS2 pos=13] To select 'd', need count in [prev_cum, cum)");
        }

        // Total scale = escape freq + sum of unmasked frequencies
        let scale = esc_freq + hi_cnt;

        let count = coder.get_current_count(scale);

        #[cfg(test)]
        if self.debug_count == 0 {
            eprintln!(
                "[DS2 pos={}] esc_freq={} hi_cnt={} scale={} count={}",
                self.debug_count, esc_freq, hi_cnt, scale, count
            );
        }

        // Find symbol or escape
        if count < hi_cnt {
            // Symbol found - re-iterate to find it
            let mut cum = 0u32;
            for j in 0..num_stats {
                let state_ptr = stats as usize + (j as usize) * 6;
                let sym = self.read_state_symbol(state_ptr);
                if self.char_mask[sym as usize] != self.esc_count {
                    let freq = self.read_state_freq(state_ptr) as u32;
                    cum += freq;
                    if cum > count {
                        let lo_cnt = cum - freq;

                        #[cfg(test)]
                        if self.debug_count == 0 || self.debug_count == 58 {
                            eprintln!(
                                "[DS2 pos={}] Selected j={} sym='{}' ({}) at cum={} lo={} freq={}",
                                self.debug_count, j, sym as char, sym, cum, lo_cnt, freq
                            );
                        }

                        #[cfg(test)]
                        if self.debug_count == 13 || self.debug_count == 58 {
                            let (code, low, range) = coder.debug_state();
                            eprintln!(
                                "[DS2 pos={}] FOUND lo={} hi={} scale={}",
                                self.debug_count, lo_cnt, cum, scale
                            );
                            eprintln!(
                                "[DS2 pos={}] Before decode: low={} range={} code={}",
                                self.debug_count, low, range, code
                            );
                        }

                        let sub = SubRange {
                            low_count: lo_cnt,
                            high_count: cum,
                            scale,
                        };
                        coder.decode(&sub);

                        #[cfg(test)]
                        if self.debug_count == 13 || self.debug_count == 58 {
                            let (code, low, range) = coder.debug_state();
                            eprintln!(
                                "[DS2 pos={}] After decode: low={} range={} code={}",
                                self.debug_count, low, range, code
                            );
                        }

                        // Update SEE2 context (matching unrar's psee2c->update())
                        if !is_root {
                            self.see2_cont[see2_row][see2_col].update();
                        }

                        self.found_state = state_ptr as u32;

                        // Update frequency and check for rescale (update2)
                        let new_freq = freq + 4;
                        self.write_state_freq(state_ptr, new_freq as u8);

                        let summ = self.read_context_summ_freq(self.min_context as usize);
                        self.write_context_summ_freq(self.min_context as usize, summ + 4);

                        // Check if rescale needed
                        if new_freq > MAX_FREQ {
                            self.rescale();
                        }

                        self.esc_count = self.esc_count.wrapping_add(1);
                        self.run_length = self.init_rl;

                        return Ok(());
                    }
                }
            }
        }

        // Escape - add scale to SEE2 Summ (matching unrar's psee2c->Summ += scale)
        if !is_root {
            self.see2_cont[see2_row][see2_col].summ = self.see2_cont[see2_row][see2_col]
                .summ
                .wrapping_add(scale as u16);
        }

        let sub = SubRange {
            low_count: hi_cnt,
            high_count: scale,
            scale,
        };
        coder.decode(&sub);

        // Mask remaining unmasked symbols
        for j in 0..num_stats {
            let state_ptr = stats as usize + (j as usize) * 6;
            let sym = self.read_state_symbol(state_ptr);
            if self.char_mask[sym as usize] != self.esc_count {
                self.char_mask[sym as usize] = self.esc_count;
            }
        }
        self.num_masked = num_stats as usize;

        Ok(())
    }

    /// Update the model after decoding.
    /// Create a child context.
    /// Returns the new context pointer, or 0 on failure.
    fn create_child(
        &mut self,
        parent_ctx: u32,
        p_stats: usize,
        first_state_symbol: u8,
        first_state_freq: u8,
        first_state_successor: u32,
    ) -> u32 {
        let pc = match self.sub_alloc.alloc_context() {
            Some(ctx) => ctx as u32,
            None => return 0,
        };

        // NumStats = 1 (binary context)
        self.write_context_num_stats(pc as usize, 1);

        // OneState = FirstState (stored inline at offset 2, in the union with SummFreq+Stats)
        // Layout: Symbol(1) + Freq(1) + Successor(4) = 6 bytes at offset 2-7
        self.write_state(
            pc as usize + 2,
            first_state_symbol,
            first_state_freq,
            first_state_successor,
        );

        // Suffix = parent context
        self.write_context_suffix(pc as usize, parent_ctx);

        // Update pStats->Successor to point to new context
        self.write_state_successor(p_stats, pc);

        pc
    }

    /// Rescale frequencies in the current context when they exceed MAX_FREQ.
    /// This halves all frequencies while maintaining sorted order.
    fn rescale(&mut self) {
        let ctx = self.min_context as usize;
        let old_ns = self.read_context_num_stats(ctx);
        let stats = self.read_context_stats(ctx);

        // Move FoundState to front (swap chain)
        let mut p = self.found_state as usize;
        while p != stats as usize {
            // Swap p with p-6 (previous state)
            let prev_p = p - 6;
            let p_sym = self.read_state_symbol(p);
            let p_freq = self.read_state_freq(p);
            let p_succ = self.read_state_successor(p);
            let prev_sym = self.read_state_symbol(prev_p);
            let prev_freq = self.read_state_freq(prev_p);
            let prev_succ = self.read_state_successor(prev_p);
            self.write_state(p, prev_sym, prev_freq, prev_succ);
            self.write_state(prev_p, p_sym, p_freq, p_succ);
            p = prev_p;
        }

        // Add 4 to first symbol's freq and SummFreq
        let first_freq = self.read_state_freq(stats as usize);
        self.write_state_freq(stats as usize, first_freq.saturating_add(4));
        let summ = self.read_context_summ_freq(ctx);
        self.write_context_summ_freq(ctx, summ.saturating_add(4));

        // Calculate EscFreq and Adder
        let new_first_freq = self.read_state_freq(stats as usize) as u32;
        let mut esc_freq = self.read_context_summ_freq(ctx) as i32 - new_first_freq as i32;
        let adder = if self.order_fall != 0 { 1 } else { 0 };

        // Halve first symbol's freq
        let halved = ((new_first_freq + adder) >> 1) as u8;
        self.write_state_freq(stats as usize, halved);
        let mut new_summ = halved as u16;

        // Halve all other frequencies, maintaining sorted order
        for i in 1..old_ns as usize {
            let state_ptr = stats as usize + i * 6;
            let freq = self.read_state_freq(state_ptr);
            esc_freq -= freq as i32;

            let halved = ((freq as u32 + adder) >> 1) as u8;
            self.write_state_freq(state_ptr, halved);
            new_summ += halved as u16;

            // Bubble up if needed (maintain sorted order by freq)
            if halved > self.read_state_freq(state_ptr - 6) {
                // Save current state
                let sym = self.read_state_symbol(state_ptr);
                let succ = self.read_state_successor(state_ptr);

                // Find insertion point
                let mut j = state_ptr - 6;
                while j >= stats as usize + 6 && halved > self.read_state_freq(j - 6) {
                    j -= 6;
                }

                // Shift states down
                let mut k = state_ptr;
                while k > j {
                    let prev_sym = self.read_state_symbol(k - 6);
                    let prev_freq = self.read_state_freq(k - 6);
                    let prev_succ = self.read_state_successor(k - 6);
                    self.write_state(k, prev_sym, prev_freq, prev_succ);
                    k -= 6;
                }

                // Insert at j
                self.write_state(j, sym, halved, succ);
            }
        }

        // Handle zero-frequency states (remove them)
        let mut num_zeros = 0;
        for i in (0..old_ns as usize).rev() {
            let state_ptr = stats as usize + i * 6;
            if self.read_state_freq(state_ptr) == 0 {
                num_zeros += 1;
            } else {
                break;
            }
        }

        if num_zeros > 0 {
            esc_freq += num_zeros;
            let new_ns = old_ns - num_zeros as u16;

            if new_ns == 1 {
                // Convert back to binary context
                let sym = self.read_state_symbol(stats as usize);
                let mut freq = self.read_state_freq(stats as usize);
                let succ = self.read_state_successor(stats as usize);

                // Halve freq until EscFreq <= 1
                while esc_freq > 1 {
                    freq = freq.saturating_sub(freq >> 1);
                    esc_freq >>= 1;
                }

                // Free the stats array
                let units = (old_ns as usize + 1) >> 1;
                self.sub_alloc.free_units(stats as usize, units);

                // Write OneState
                self.write_context_num_stats(ctx, 1);
                self.write_state(ctx + 2, sym, freq, succ);
                self.found_state = (ctx + 2) as u32;
                return;
            }

            self.write_context_num_stats(ctx, new_ns);

            // TODO: Shrink stats array if needed (requires shrink_units in allocator)
            // For now, we just leave the extra space allocated
        }

        // Update SummFreq with remaining escape frequency
        new_summ += (esc_freq - (esc_freq >> 1)) as u16;
        self.write_context_summ_freq(ctx, new_summ);

        // FoundState is now the first state
        let new_stats = self.read_context_stats(ctx);
        self.found_state = new_stats;
    }

    /// Create successors for the current context chain.
    /// Returns the new context, or 0 on failure.
    fn create_successors(&mut self, skip: bool, p1: Option<usize>) -> u32 {
        let up_branch = self.read_state_successor(self.found_state as usize);
        let fs_symbol = self.read_state_symbol(self.found_state as usize);

        #[cfg(test)]
        if self.debug_count == 12 {
            eprintln!(
                "[CS pos=12] Entry: skip={} p1={:?} up_branch={} fs_symbol='{}'",
                skip, p1, up_branch, fs_symbol as char
            );
        }

        let mut pc = self.min_context;
        // Initialize array - optimizer may elide this when it sees writes before reads
        let mut ps: [usize; MAX_O] = [0; MAX_O];
        let mut pps_idx = 0;

        if !skip {
            ps[pps_idx] = self.found_state as usize;
            pps_idx += 1;
            let suffix = self.read_context_suffix(pc as usize);
            if suffix == 0 {
                // goto NO_LOOP
                if pps_idx == 0 {
                    return pc;
                }
                return self.create_successors_finish(pc, &ps, pps_idx, up_branch, fs_symbol);
            }
        }

        let mut p: usize;
        let mut start_in_loop = false;
        if let Some(p1_val) = p1 {
            p = p1_val;
            pc = self.read_context_suffix(pc as usize);
            start_in_loop = true;
        } else {
            p = 0; // Will be set in loop
        }

        // Main loop
        loop {
            if !start_in_loop {
                pc = self.read_context_suffix(pc as usize);
                if pc == 0 {
                    break;
                }

                let num_stats = self.read_context_num_stats(pc as usize);
                if num_stats != 1 {
                    let stats = self.read_context_stats(pc as usize);
                    p = stats as usize;
                    if self.read_state_symbol(p) != fs_symbol {
                        loop {
                            p += 6;
                            if self.read_state_symbol(p) == fs_symbol {
                                break;
                            }
                        }
                    }
                } else {
                    // OneState at context+2 (in union)
                    p = pc as usize + 2;
                }
            }
            start_in_loop = false; // Only skip to LOOP_ENTRY on first iteration

            // LOOP_ENTRY
            let p_successor = self.read_state_successor(p);
            if p_successor != up_branch {
                pc = p_successor;
                break;
            }

            if pps_idx >= MAX_O {
                return 0;
            }
            ps[pps_idx] = p;
            pps_idx += 1;

            let suffix = self.read_context_suffix(pc as usize);
            if suffix == 0 {
                break;
            }
        }

        self.create_successors_finish(pc, &ps, pps_idx, up_branch, fs_symbol)
    }

    fn create_successors_finish(
        &mut self,
        mut pc: u32,
        ps: &[usize; MAX_O],
        pps_idx: usize,
        up_branch: u32,
        fs_symbol: u8,
    ) -> u32 {
        #[cfg(test)]
        if self.debug_count == 12 {
            eprintln!(
                "[CS_FINISH pos=12] pc={} pps_idx={} up_branch={} fs_symbol='{}'",
                pc, pps_idx, up_branch, fs_symbol as char
            );
            for i in 0..pps_idx {
                let sym = self.read_state_symbol(ps[i]);
                eprintln!(
                    "[CS_FINISH pos=12] ps[{}]={} sym='{}'",
                    i, ps[i], sym as char
                );
            }
        }

        // Suppress unused warning when not in test mode
        let _ = fs_symbol;

        if pps_idx == 0 {
            return pc;
        }

        // UpState.Symbol = *(byte*)UpBranch
        let up_state_symbol = self.sub_alloc.read_byte(up_branch as usize);
        // UpState.Successor = (byte*)UpBranch + 1
        let up_state_successor = up_branch + 1;

        let up_state_freq: u8;
        let num_stats = self.read_context_num_stats(pc as usize);
        if num_stats != 1 {
            let text_ptr = self.sub_alloc.get_text_ptr();
            if pc as usize <= text_ptr {
                return 0;
            }

            let stats = self.read_context_stats(pc as usize);
            let mut p = stats as usize;
            if self.read_state_symbol(p) != up_state_symbol {
                loop {
                    p += 6;
                    if self.read_state_symbol(p) == up_state_symbol {
                        break;
                    }
                }
            }

            let cf = self.read_state_freq(p) as u32 - 1;
            let s0 = self.read_context_summ_freq(pc as usize) as u32 - num_stats as u32 - cf;
            // unrar: UpState.Freq=1+((2*cf <= s0)?(5*cf > s0):((2*cf+3*s0-1)/(2*s0)));
            // Note: the 1+ applies to the entire expression!
            up_state_freq = (1 + if 2 * cf <= s0 {
                if 5 * cf > s0 {
                    1
                } else {
                    0
                }
            } else {
                (2 * cf + 3 * s0 - 1) / (2 * s0)
            })
            .min(255) as u8;
        } else {
            // OneState.Freq (at offset 2+1=3)
            up_state_freq = self.read_state_freq(pc as usize + 2);
        }

        // Create children in reverse order
        let mut i = pps_idx;
        while i > 0 {
            i -= 1;
            pc = self.create_child(
                pc,
                ps[i],
                up_state_symbol,
                up_state_freq,
                up_state_successor,
            );
            if pc == 0 {
                return 0;
            }
        }

        pc
    }

    fn update_model(&mut self) {
        #[cfg(test)]
        if self.debug_count == 11 || self.debug_count == 12 {
            eprintln!(
                "[UPDATE pos={}] Before: min_context={} max_context={} order_fall={}",
                self.debug_count, self.min_context, self.max_context, self.order_fall
            );
        }

        // Read the found state
        let fs_symbol = self.read_state_symbol(self.found_state as usize);
        let fs_freq = self.read_state_freq(self.found_state as usize);
        let fs_successor = self.read_state_successor(self.found_state as usize);

        #[cfg(test)]
        if self.debug_count == 12 {
            eprintln!(
                "[UPDATE pos=12] found_state={} fs_sym='{}' fs_freq={} fs_successor={}",
                self.found_state, fs_symbol as char, fs_freq, fs_successor
            );
            let text_ptr = self.sub_alloc.get_text_ptr();
            eprintln!(
                "[UPDATE pos=12] text_ptr={}, fs_successor<=text_ptr: {}",
                text_ptr,
                (fs_successor as usize) <= text_ptr
            );
        }

        // Update frequency in parent context (suffix) and find p
        let mut p: Option<usize> = None;
        let suffix = self.read_context_suffix(self.min_context as usize);
        if suffix != 0 && (fs_freq as u32) < MAX_FREQ / 4 {
            let num_stats = self.read_context_num_stats(suffix as usize);
            if num_stats != 1 {
                // Find the symbol in parent's stats
                let stats = self.read_context_stats(suffix as usize);
                let mut state_ptr = stats as usize;
                if self.read_state_symbol(state_ptr) != fs_symbol {
                    loop {
                        state_ptr += 6;
                        if self.read_state_symbol(state_ptr) == fs_symbol {
                            break;
                        }
                    }
                    // Swap with previous if freq >= prev freq (move to front)
                    let freq = self.read_state_freq(state_ptr);
                    let prev_freq = self.read_state_freq(state_ptr - 6);
                    if freq >= prev_freq {
                        // Swap states
                        let prev_ptr = state_ptr - 6;
                        let curr_sym = self.read_state_symbol(state_ptr);
                        let curr_freq = self.read_state_freq(state_ptr);
                        let curr_succ = self.read_state_successor(state_ptr);
                        let prev_sym = self.read_state_symbol(prev_ptr);
                        let prev_freq = self.read_state_freq(prev_ptr);
                        let prev_succ = self.read_state_successor(prev_ptr);
                        self.write_state(state_ptr, prev_sym, prev_freq, prev_succ);
                        self.write_state(prev_ptr, curr_sym, curr_freq, curr_succ);
                        state_ptr = prev_ptr;
                    }
                }
                p = Some(state_ptr);
                let freq = self.read_state_freq(state_ptr);
                if (freq as u32) < MAX_FREQ - 9 {
                    self.write_state_freq(state_ptr, freq + 2);
                    let sf = self.read_context_summ_freq(suffix as usize);
                    self.write_context_summ_freq(suffix as usize, sf + 2);
                }
            } else {
                // Binary context - OneState at suffix+2 (in union)
                let one_state_ptr = suffix as usize + 2;
                p = Some(one_state_ptr);
                let freq = self.read_state_freq(one_state_ptr);
                if freq < 32 {
                    self.write_state_freq(one_state_ptr, freq + 1);
                }
            }
        }

        // If order_fall == 0, just create successors and return
        if self.order_fall == 0 {
            let new_ctx = self.create_successors(true, p);
            if new_ctx == 0 {
                self.restart_model();
                return;
            }
            self.write_state_successor(self.found_state as usize, new_ctx);
            self.min_context = new_ctx;
            self.max_context = new_ctx;
            return;
        }

        // Write symbol to text memory
        let text_ptr = self.sub_alloc.get_text_ptr();
        let units_start = self.sub_alloc.get_units_start();
        if text_ptr >= units_start {
            self.restart_model();
            return;
        }
        self.sub_alloc.write_byte(text_ptr, fs_symbol);
        self.sub_alloc.advance_text_ptr();

        let mut successor = self.sub_alloc.get_text_ptr() as u32;

        // fs_successor_new tracks what we'll use for max/min context at the end
        let fs_successor_new: u32;

        if fs_successor != 0 {
            let text_ptr = self.sub_alloc.get_text_ptr();
            if (fs_successor as usize) <= text_ptr {
                let new_succ = self.create_successors(false, p);
                if new_succ == 0 {
                    self.restart_model();
                    return;
                }
                self.write_state_successor(self.found_state as usize, new_succ);
                fs_successor_new = new_succ;
            } else {
                fs_successor_new = fs_successor;
            }
            self.order_fall -= 1;
            if self.order_fall == 0 {
                // Update successor to use fs.Successor instead of text pointer
                successor = fs_successor_new;
                if self.max_context != self.min_context {
                    // Undo text ptr advance
                    self.sub_alloc.retreat_text_ptr();
                }
                // NOTE: Don't return early! Continue to expansion loop.
                // This is the key fix - unrar doesn't return here either.
            }
        } else {
            // First time seeing this symbol in this context chain
            self.write_state_successor(self.found_state as usize, successor);
            // fs.Successor = MinContext (for the final assignment)
            fs_successor_new = self.min_context;
        }

        // Add symbol to contexts from max_context to min_context
        let ns = self.read_context_num_stats(self.min_context as usize) as u32;
        let summ_freq = self.read_context_summ_freq(self.min_context as usize) as u32;
        let s0 = summ_freq
            .saturating_sub(ns)
            .saturating_sub(fs_freq as u32)
            .saturating_add(1);

        let mut pc = self.max_context;
        while pc != self.min_context {
            let ns1 = self.read_context_num_stats(pc as usize);

            if ns1 != 1 {
                // Multi-symbol context - expand if needed
                if (ns1 & 1) == 0 {
                    // Need to expand stats array
                    let old_stats = self.read_context_stats(pc as usize);
                    let new_stats = self
                        .sub_alloc
                        .expand_units(old_stats as usize, (ns1 >> 1) as usize);
                    if new_stats.is_none() {
                        self.restart_model();
                        return;
                    }
                    self.write_context_stats(pc as usize, new_stats.unwrap() as u32);
                }

                // Update summ_freq based on symbol distribution
                let mut sf_inc = 0u16;
                if 2 * ns1 < ns as u16 {
                    sf_inc += 1;
                }
                let summ = self.read_context_summ_freq(pc as usize);
                if 4 * ns1 as u32 <= ns && summ <= 8 * ns1 {
                    sf_inc += 2;
                }
                self.write_context_summ_freq(pc as usize, summ + sf_inc);
            } else {
                // Binary context - convert to multi-symbol
                let new_stats = self.sub_alloc.alloc_units(1);
                if new_stats.is_none() {
                    self.restart_model();
                    return;
                }
                let new_stats = new_stats.unwrap();

                // Copy OneState (at offset 2) to new stats
                let one_state_sym = self.read_state_symbol(pc as usize + 2);
                let one_state_freq = self.read_state_freq(pc as usize + 2);
                let one_state_succ = self.read_state_successor(pc as usize + 2);
                self.write_state(new_stats, one_state_sym, one_state_freq, one_state_succ);

                self.write_context_stats(pc as usize, new_stats as u32);

                // Update freq
                let freq = self.read_state_freq(new_stats);
                let new_freq = if (freq as u32) < MAX_FREQ / 4 - 1 {
                    freq * 2
                } else {
                    (MAX_FREQ - 4) as u8
                };
                self.write_state_freq(new_stats, new_freq);

                // Set summ_freq - use self.init_esc (dynamic, set during binary escape)
                let init_esc_extra = if ns > 3 { 1 } else { 0 };
                let new_summ = new_freq as u16 + self.init_esc as u16 + init_esc_extra as u16;
                self.write_context_summ_freq(pc as usize, new_summ);
                #[cfg(test)]
                if pc == 15728580 {
                    eprintln!("[UPDATE_MODEL pos={}] context {} promoted: new_freq={} init_esc={} init_esc_extra={} → SummFreq={}", 
                             self.debug_count, pc, new_freq, self.init_esc, init_esc_extra, new_summ);
                }
            }

            // Calculate new symbol's frequency
            let summ = self.read_context_summ_freq(pc as usize) as u32;
            let cf = 2 * fs_freq as u32 * (summ + 6);
            let sf = s0 + summ;

            let sym_freq: u8;
            if cf < 6 * sf {
                sym_freq = 1 + (cf > sf) as u8 + (cf >= 4 * sf) as u8;
                let summ = self.read_context_summ_freq(pc as usize);
                self.write_context_summ_freq(pc as usize, summ + 3);
                #[cfg(test)]
                if pc == 15728580 {
                    eprintln!(
                        "[UPDATE_MODEL pos={}] context {} SummFreq: {} → {} (branch1, sym_freq={})",
                        self.debug_count,
                        pc,
                        summ,
                        summ + 3,
                        sym_freq
                    );
                }
            } else {
                sym_freq = 4 + (cf >= 9 * sf) as u8 + (cf >= 12 * sf) as u8 + (cf >= 15 * sf) as u8;
                let summ = self.read_context_summ_freq(pc as usize);
                self.write_context_summ_freq(pc as usize, summ + sym_freq as u16);
                #[cfg(test)]
                if pc == 15728580 {
                    eprintln!(
                        "[UPDATE_MODEL pos={}] context {} SummFreq: {} → {} (branch2, sym_freq={})",
                        self.debug_count,
                        pc,
                        summ,
                        summ + sym_freq as u16,
                        sym_freq
                    );
                }
            }

            // Add new symbol at end of stats
            let stats = self.read_context_stats(pc as usize);
            let new_state_ptr = stats as usize + (ns1 as usize) * 6;

            #[cfg(test)]
            if ns1 >= 256 {
                eprintln!(
                    "[UPDATE] ERROR: Adding state at ns1={} to context {} - exceeds 256!",
                    ns1, pc
                );
            }

            self.write_state(new_state_ptr, fs_symbol, sym_freq, successor);

            // Increment NumStats
            self.write_context_num_stats(pc as usize, ns1 + 1);

            // Move to suffix
            pc = self.read_context_suffix(pc as usize);
        }

        // Update context pointers to fs.Successor
        self.max_context = fs_successor_new;
        self.min_context = fs_successor_new;

        #[cfg(test)]
        if self.debug_count == 11 || self.debug_count == 12 {
            eprintln!(
                "[UPDATE pos={}] After: min_context={} max_context={}",
                self.debug_count, self.min_context, self.max_context
            );
        }
    }

    /// Clear the character mask.
    fn clear_mask(&mut self) {
        // Increment esc_count instead of zeroing array - much faster
        // Skip 0 since that's the uninitialized state
        self.esc_count = self.esc_count.wrapping_add(1);
        if self.esc_count == 0 {
            // Wrapped around - must zero the array and restart at 1
            self.esc_count = 1;
            self.char_mask = [0; 256];
        }
    }

    // Helper methods for reading/writing context and state structures

    #[inline]
    fn read_context_num_stats(&self, offset: usize) -> u16 {
        self.sub_alloc.read_u16(offset)
    }

    #[inline]
    fn write_context_num_stats(&mut self, offset: usize, val: u16) {
        self.sub_alloc.write_u16(offset, val);
    }

    #[inline]
    fn read_context_summ_freq(&self, offset: usize) -> u16 {
        self.sub_alloc.read_u16(offset + 2)
    }

    #[inline]
    fn write_context_summ_freq(&mut self, offset: usize, val: u16) {
        self.sub_alloc.write_u16(offset + 2, val);
    }

    #[inline]
    fn read_context_stats(&self, offset: usize) -> u32 {
        self.sub_alloc.read_u32(offset + 4)
    }

    #[inline]
    fn write_context_stats(&mut self, offset: usize, val: u32) {
        self.sub_alloc.write_u32(offset + 4, val);
    }

    #[inline]
    fn read_context_suffix(&self, offset: usize) -> u32 {
        self.sub_alloc.read_u32(offset + 8)
    }

    #[inline]
    fn write_context_suffix(&mut self, offset: usize, val: u32) {
        self.sub_alloc.write_u32(offset + 8, val);
    }

    #[inline]
    fn read_context_one_state(&self, offset: usize) -> State {
        // OneState is at offset 2 (in the union with SummFreq+Stats)
        // Layout: Symbol(1) + Freq(1) + Successor(4) = 6 bytes at offset 2-7
        State {
            symbol: self.sub_alloc.read_byte(offset + 2),
            freq: self.sub_alloc.read_byte(offset + 3),
            successor: self.sub_alloc.read_u32(offset + 4),
        }
    }

    #[inline]
    fn write_context_one_state_freq(&mut self, offset: usize, freq: u8) {
        // OneState.Freq is at offset+3 (Symbol at +2, Freq at +3)
        self.sub_alloc.write_byte(offset + 3, freq);
    }

    #[inline]
    fn read_state_symbol(&self, offset: usize) -> u8 {
        self.sub_alloc.read_byte(offset)
    }

    #[inline]
    fn read_state_freq(&self, offset: usize) -> u8 {
        self.sub_alloc.read_byte(offset + 1)
    }

    #[inline]
    fn write_state_freq(&mut self, offset: usize, freq: u8) {
        self.sub_alloc.write_byte(offset + 1, freq);
    }

    #[inline]
    fn read_state_successor(&self, offset: usize) -> u32 {
        self.sub_alloc.read_u32(offset + 2)
    }

    #[inline]
    fn write_state_successor(&mut self, offset: usize, successor: u32) {
        self.sub_alloc.write_u32(offset + 2, successor);
    }

    fn write_state(&mut self, offset: usize, symbol: u8, freq: u8, successor: u32) {
        self.sub_alloc.write_byte(offset, symbol);
        self.sub_alloc.write_byte(offset + 1, freq);
        self.sub_alloc.write_byte(offset + 2, successor as u8);
        self.sub_alloc
            .write_byte(offset + 3, (successor >> 8) as u8);
        self.sub_alloc
            .write_byte(offset + 4, (successor >> 16) as u8);
        self.sub_alloc
            .write_byte(offset + 5, (successor >> 24) as u8);
    }
}

impl Default for PpmModel {
    fn default() -> Self {
        Self::new()
    }
}
