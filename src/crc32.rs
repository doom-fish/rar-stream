//! CRC32 calculation for RAR archives.
//!
//! Uses slicing-by-8 for ~8x throughput over naive byte-at-a-time,
//! with no external dependencies.

/// CRC32 lookup table (polynomial 0xEDB88320)
const CRC32_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
};

/// Extended CRC32 tables for slicing-by-8 (8 × 256 entries).
/// Each table[k] is derived by feeding k additional zero bytes through the CRC.
const CRC32_TABLES: [[u32; 256]; 8] = {
    let mut tables = [[0u32; 256]; 8];
    // Table 0 is the standard byte table
    let mut i = 0;
    while i < 256 {
        tables[0][i] = CRC32_TABLE[i];
        i += 1;
    }
    // Tables 1..7: each entry = crc32 of (table[k-1][i] >> 8) ^ table[0][table[k-1][i] & 0xFF]
    let mut k = 1;
    while k < 8 {
        let mut i = 0;
        while i < 256 {
            let prev = tables[k - 1][i];
            tables[k][i] = (prev >> 8) ^ tables[0][(prev & 0xFF) as usize];
            i += 1;
        }
        k += 1;
    }
    tables
};

/// Calculate CRC32 of data using slicing-by-8 for high throughput.
#[inline]
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF_u32;
    let mut pos = 0;
    let len = data.len();

    // Process 8 bytes at a time
    while pos + 8 <= len {
        let b = u64::from_le_bytes([
            data[pos],
            data[pos + 1],
            data[pos + 2],
            data[pos + 3],
            data[pos + 4],
            data[pos + 5],
            data[pos + 6],
            data[pos + 7],
        ]);
        let v = b ^ crc as u64;
        crc = CRC32_TABLES[7][(v & 0xFF) as usize]
            ^ CRC32_TABLES[6][((v >> 8) & 0xFF) as usize]
            ^ CRC32_TABLES[5][((v >> 16) & 0xFF) as usize]
            ^ CRC32_TABLES[4][((v >> 24) & 0xFF) as usize]
            ^ CRC32_TABLES[3][((v >> 32) & 0xFF) as usize]
            ^ CRC32_TABLES[2][((v >> 40) & 0xFF) as usize]
            ^ CRC32_TABLES[1][((v >> 48) & 0xFF) as usize]
            ^ CRC32_TABLES[0][((v >> 56) & 0xFF) as usize];
        pos += 8;
    }

    // Process remaining bytes
    while pos < len {
        let index = ((crc ^ data[pos] as u32) & 0xFF) as usize;
        crc = (crc >> 8) ^ CRC32_TABLES[0][index];
        pos += 1;
    }

    crc ^ 0xFFFF_FFFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32() {
        // Test with known values
        assert_eq!(crc32(b""), 0x00000000);
        assert_eq!(crc32(b"123456789"), 0xCBF43926);
    }

    #[test]
    fn test_crc32_various_lengths() {
        // Test alignment edge cases (1-7 byte remainders)
        assert_eq!(crc32(b"a"), 0xE8B7BE43);
        assert_eq!(crc32(b"ab"), 0x9E83486D);
        assert_eq!(
            crc32(b"The quick brown fox jumps over the lazy dog"),
            0x414FA339
        );
    }
}
