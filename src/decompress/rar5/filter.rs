//! RAR5 filter implementation.
//!
//! RAR5 uses simplified filters compared to RAR4's VM-based system.
//! Only 4 filter types are supported: Delta, E8, E8E9, and ARM.

/// Maximum filter block size (4MB).
const MAX_FILTER_BLOCK_SIZE: usize = 0x400000;

/// RAR5 filter types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FilterType {
    /// Delta filter - byte-wise diff per channel
    Delta = 0,
    /// E8 filter - x86 CALL instruction preprocessing
    E8 = 1,
    /// E8E9 filter - x86 CALL/JMP preprocessing
    E8E9 = 2,
    /// ARM filter - ARM branch preprocessing
    Arm = 3,
}

impl FilterType {
    /// Parse filter type from 3-bit value.
    pub fn from_bits(bits: u8) -> Option<Self> {
        match bits {
            0 => Some(FilterType::Delta),
            1 => Some(FilterType::E8),
            2 => Some(FilterType::E8E9),
            3 => Some(FilterType::Arm),
            _ => None,
        }
    }
}

/// A pending filter to be applied after decompression.
#[derive(Debug, Clone)]
pub struct UnpackFilter {
    /// Filter type
    pub filter_type: FilterType,
    /// Start position in decompressed data (relative to window)
    pub block_start: usize,
    /// Length of data to filter
    pub block_length: usize,
    /// Number of channels (for delta filter, 1-32)
    pub channels: u8,
}

impl UnpackFilter {
    /// Create a new filter.
    pub fn new(filter_type: FilterType, block_start: usize, block_length: usize, channels: u8) -> Self {
        Self {
            filter_type,
            block_start,
            block_length: block_length.min(MAX_FILTER_BLOCK_SIZE),
            channels,
        }
    }
}

/// Apply a filter to decompressed data in-place.
/// Returns the filtered data.
pub fn apply_filter(data: &mut [u8], filter: &UnpackFilter, file_offset: u64) -> Vec<u8> {
    match filter.filter_type {
        FilterType::Delta => apply_delta_filter(data, filter.channels as usize),
        FilterType::E8 => apply_e8_filter(data, file_offset as u32, false),
        FilterType::E8E9 => apply_e8_filter(data, file_offset as u32, true),
        FilterType::Arm => apply_arm_filter(data, file_offset as u32),
    }
}

/// Apply delta filter.
/// Bytes from same channels are grouped, need to interleave them back.
fn apply_delta_filter(data: &[u8], channels: usize) -> Vec<u8> {
    if channels == 0 || channels > 32 || data.is_empty() {
        return data.to_vec();
    }

    let data_size = data.len();
    let mut output = vec![0u8; data_size];
    let mut src_pos = 0;

    // Bytes from same channels are grouped to continual data blocks,
    // so we need to place them back to their interleaving positions.
    for cur_channel in 0..channels {
        let mut prev_byte: u8 = 0;
        let mut dest_pos = cur_channel;
        while dest_pos < data_size {
            // Delta decode: output[i] = prev - data[src]
            prev_byte = prev_byte.wrapping_sub(data[src_pos]);
            output[dest_pos] = prev_byte;
            src_pos += 1;
            dest_pos += channels;
        }
    }

    output
}

/// Apply E8/E8E9 filter for x86 executables.
/// Converts absolute addresses back to relative.
fn apply_e8_filter(data: &mut [u8], file_offset: u32, include_e9: bool) -> Vec<u8> {
    const FILE_SIZE: u32 = 0x1000000; // 16MB

    if data.len() < 5 {
        return data.to_vec();
    }

    let mut cur_pos: usize = 0;
    while cur_pos + 4 < data.len() {
        let cur_byte = data[cur_pos];
        cur_pos += 1;

        if cur_byte == 0xE8 || (include_e9 && cur_byte == 0xE9) {
            let offset = (cur_pos as u32).wrapping_add(file_offset);
            let addr = u32::from_le_bytes([
                data[cur_pos],
                data[cur_pos + 1],
                data[cur_pos + 2],
                data[cur_pos + 3],
            ]);

            // Check high bit for sign
            if (addr & 0x80000000) != 0 {
                // addr < 0
                if (addr.wrapping_add(offset) & 0x80000000) == 0 {
                    // addr + offset >= 0
                    let new_addr = addr.wrapping_add(FILE_SIZE);
                    data[cur_pos..cur_pos + 4].copy_from_slice(&new_addr.to_le_bytes());
                }
            } else {
                // addr >= 0
                if (addr.wrapping_sub(FILE_SIZE) & 0x80000000) != 0 {
                    // addr < FILE_SIZE
                    let new_addr = addr.wrapping_sub(offset);
                    data[cur_pos..cur_pos + 4].copy_from_slice(&new_addr.to_le_bytes());
                }
            }
            cur_pos += 4;
        }
    }

    data.to_vec()
}

/// Apply ARM filter for ARM executables.
/// Converts BL instruction addresses.
fn apply_arm_filter(data: &mut [u8], file_offset: u32) -> Vec<u8> {
    if data.len() < 4 {
        return data.to_vec();
    }

    let mut cur_pos: usize = 0;
    while cur_pos + 3 < data.len() {
        // Check for BL instruction (0xEB in high byte with condition 1110 = Always)
        if data[cur_pos + 3] == 0xEB {
            let offset = u32::from_le_bytes([
                data[cur_pos],
                data[cur_pos + 1],
                data[cur_pos + 2],
                0,
            ]);
            let new_offset = offset.wrapping_sub((file_offset.wrapping_add(cur_pos as u32)) / 4);
            data[cur_pos] = (new_offset & 0xFF) as u8;
            data[cur_pos + 1] = ((new_offset >> 8) & 0xFF) as u8;
            data[cur_pos + 2] = ((new_offset >> 16) & 0xFF) as u8;
        }
        cur_pos += 4;
    }

    data.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_filter_single_channel() {
        // Single channel: just delta decode
        let data = [5, 3, 2, 1]; // Deltas
        let result = apply_delta_filter(&data, 1);
        // prev = 0 - 5 = 251 (wrapping)
        // prev = 251 - 3 = 248
        // prev = 248 - 2 = 246
        // prev = 246 - 1 = 245
        assert_eq!(result, vec![251, 248, 246, 245]);
    }

    #[test]
    fn test_delta_filter_two_channels() {
        // Two channels: interleave L0, R0, L1, R1, L2, R2 from L0,L1,L2,R0,R1,R2
        let data = [0, 0, 0, 0, 0, 0]; // All zeros = no change
        let result = apply_delta_filter(&data, 2);
        assert_eq!(result, vec![0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_filter_type_from_bits() {
        assert_eq!(FilterType::from_bits(0), Some(FilterType::Delta));
        assert_eq!(FilterType::from_bits(1), Some(FilterType::E8));
        assert_eq!(FilterType::from_bits(2), Some(FilterType::E8E9));
        assert_eq!(FilterType::from_bits(3), Some(FilterType::Arm));
        assert_eq!(FilterType::from_bits(4), None);
    }
}
