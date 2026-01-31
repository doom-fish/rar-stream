//! RAR5 end of archive header parser.
//!
//! The end header marks the end of the archive and contains
//! optional flags about the archive state.

use super::{Rar5HeaderFlags, VintReader};
use crate::error::{RarError, Result};

/// RAR5 end header flags.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rar5EndFlags {
    /// Archive continues in next volume
    pub has_next_volume: bool,
}

impl From<u64> for Rar5EndFlags {
    fn from(flags: u64) -> Self {
        Self {
            has_next_volume: flags & 0x0001 != 0,
        }
    }
}

/// Parsed RAR5 end of archive header.
#[derive(Debug, Clone)]
pub struct Rar5EndHeader {
    /// Header CRC32
    pub crc32: u32,
    /// Total header size in bytes
    pub header_size: u64,
    /// Common header flags
    pub header_flags: Rar5HeaderFlags,
    /// End-specific flags
    pub end_flags: Rar5EndFlags,
}

pub struct Rar5EndHeaderParser;

impl Rar5EndHeaderParser {
    /// Parse RAR5 end of archive header from buffer.
    pub fn parse(buffer: &[u8]) -> Result<(Rar5EndHeader, usize)> {
        if buffer.len() < 6 {
            return Err(RarError::BufferTooSmall {
                needed: 6,
                have: buffer.len(),
            });
        }

        let mut reader = VintReader::new(buffer);

        // Read CRC32 (4 bytes, not vint)
        let crc32 = reader.read_u32_le().ok_or(RarError::InvalidHeader)?;

        // Read header size (vint) - this is the size of header content AFTER this vint
        let header_size = reader.read().ok_or(RarError::InvalidHeader)?;

        // Record position after reading header_size vint
        let header_content_start = reader.position();

        // Read header type (vint) - should be 5 for end header
        let header_type = reader.read().ok_or(RarError::InvalidHeader)?;
        if header_type != 5 {
            return Err(RarError::InvalidHeader);
        }

        // Read header flags (vint)
        let header_flags_raw = reader.read().ok_or(RarError::InvalidHeader)?;
        let header_flags = Rar5HeaderFlags::from(header_flags_raw);

        // Read end flags (vint)
        let end_flags_raw = reader.read().ok_or(RarError::InvalidHeader)?;
        let end_flags = Rar5EndFlags::from(end_flags_raw);

        // Calculate total bytes consumed
        // header_size indicates bytes after the header_size vint itself
        let total_consumed = header_content_start + header_size as usize;

        Ok((
            Rar5EndHeader {
                crc32,
                header_size,
                header_flags,
                end_flags,
            },
            total_consumed,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_end_header() {
        // Minimal RAR5 end header
        let header = [
            0x00, 0x00, 0x00, 0x00, // CRC32 (placeholder)
            0x04, // header_size = 4
            0x05, // type = 5 (end)
            0x00, // header_flags = 0
            0x00, // end_flags = 0
        ];

        let (parsed, consumed) = Rar5EndHeaderParser::parse(&header).unwrap();
        assert_eq!(parsed.header_size, 4);
        assert!(!parsed.end_flags.has_next_volume);
        assert_eq!(consumed, 9); // 4 (CRC) + 1 (header_size vint) + 4 (header content)
    }
}
