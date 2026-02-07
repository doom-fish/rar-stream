//! Fast byte search using platform SIMD where available, SWAR fallback elsewhere.
//! Zero-dependency replacement for `memchr`.

/// Find first occurrence of `needle` in `haystack`.
#[inline]
pub fn find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: SSE2 is guaranteed on x86_64. All pointer accesses are bounds-checked.
        unsafe { sse2::find_byte(haystack, needle) }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        swar::find_byte(haystack, needle)
    }
}

/// Find first occurrence of either `a` or `b` in `haystack`.
#[inline]
pub fn find_byte2(haystack: &[u8], a: u8, b: u8) -> Option<usize> {
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: SSE2 is guaranteed on x86_64. All pointer accesses are bounds-checked.
        unsafe { sse2::find_byte2(haystack, a, b) }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        swar::find_byte2(haystack, a, b)
    }
}

/// SSE2 implementation — 16 bytes per cycle, guaranteed available on x86_64.
#[cfg(target_arch = "x86_64")]
mod sse2 {
    use core::arch::x86_64::{
        __m128i, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_or_si128, _mm_set1_epi8,
    };

    #[inline]
    #[allow(clippy::cast_possible_wrap)]
    pub unsafe fn find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
        let needle_v = _mm_set1_epi8(needle as i8);
        let len = haystack.len();
        let ptr = haystack.as_ptr();
        let mut i = 0;

        while i + 16 <= len {
            let chunk = _mm_loadu_si128(ptr.add(i).cast::<__m128i>());
            let cmp = _mm_cmpeq_epi8(chunk, needle_v);
            let mask = _mm_movemask_epi8(cmp) as u32;
            if mask != 0 {
                return Some(i + mask.trailing_zeros() as usize);
            }
            i += 16;
        }

        while i < len {
            if *ptr.add(i) == needle {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    #[inline]
    #[allow(clippy::cast_possible_wrap)]
    pub unsafe fn find_byte2(haystack: &[u8], a: u8, b: u8) -> Option<usize> {
        let va = _mm_set1_epi8(a as i8);
        let vb = _mm_set1_epi8(b as i8);
        let len = haystack.len();
        let ptr = haystack.as_ptr();
        let mut i = 0;

        while i + 16 <= len {
            let chunk = _mm_loadu_si128(ptr.add(i).cast::<__m128i>());
            let cmp = _mm_or_si128(_mm_cmpeq_epi8(chunk, va), _mm_cmpeq_epi8(chunk, vb));
            let mask = _mm_movemask_epi8(cmp) as u32;
            if mask != 0 {
                return Some(i + mask.trailing_zeros() as usize);
            }
            i += 16;
        }

        while i < len {
            let byte = *ptr.add(i);
            if byte == a || byte == b {
                return Some(i);
            }
            i += 1;
        }
        None
    }
}

/// SWAR fallback — 8 bytes per cycle using u64 arithmetic.
#[cfg(not(target_arch = "x86_64"))]
mod swar {
    const LO: u64 = 0x0101_0101_0101_0101;
    const HI: u64 = 0x8080_8080_8080_8080;

    #[inline]
    pub fn find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
        let broadcast = LO.wrapping_mul(u64::from(needle));
        // SAFETY: align_to reinterprets aligned bytes as u64
        let (prefix, chunks, suffix) = unsafe { haystack.align_to::<u64>() };

        for (i, &b) in prefix.iter().enumerate() {
            if b == needle {
                return Some(i);
            }
        }
        let prefix_len = prefix.len();
        for (ci, &chunk) in chunks.iter().enumerate() {
            let xored = chunk ^ broadcast;
            let has_match = xored.wrapping_sub(LO) & !xored & HI;
            if has_match != 0 {
                return Some(prefix_len + ci * 8 + (has_match.trailing_zeros() as usize / 8));
            }
        }
        let suffix_start = prefix_len + chunks.len() * 8;
        for (i, &b) in suffix.iter().enumerate() {
            if b == needle {
                return Some(suffix_start + i);
            }
        }
        None
    }

    #[inline]
    pub fn find_byte2(haystack: &[u8], a: u8, b: u8) -> Option<usize> {
        let ba = LO.wrapping_mul(u64::from(a));
        let bb = LO.wrapping_mul(u64::from(b));
        // SAFETY: align_to reinterprets aligned bytes as u64
        let (prefix, chunks, suffix) = unsafe { haystack.align_to::<u64>() };

        for (i, &byte) in prefix.iter().enumerate() {
            if byte == a || byte == b {
                return Some(i);
            }
        }
        let prefix_len = prefix.len();
        for (ci, &chunk) in chunks.iter().enumerate() {
            let x1 = chunk ^ ba;
            let x2 = chunk ^ bb;
            let has_match = (x1.wrapping_sub(LO) & !x1 & HI) | (x2.wrapping_sub(LO) & !x2 & HI);
            if has_match != 0 {
                return Some(prefix_len + ci * 8 + (has_match.trailing_zeros() as usize / 8));
            }
        }
        let suffix_start = prefix_len + chunks.len() * 8;
        for (i, &byte) in suffix.iter().enumerate() {
            if byte == a || byte == b {
                return Some(suffix_start + i);
            }
        }
        None
    }
}
