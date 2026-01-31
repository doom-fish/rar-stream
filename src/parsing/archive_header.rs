//! Archive header parser.
//!
//! The archive header follows the marker header and contains
//! archive-level flags and metadata.

use crate::error::{RarError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveHeader {
    pub crc: u16,
    pub header_type: u8,
    pub flags: u16,
    pub size: u16,
    pub reserved1: u16,
    pub reserved2: u32,
    // Parsed flags
    pub has_volume_attributes: bool,
    pub has_comment: bool,
    pub is_locked: bool,
    pub has_solid_attributes: bool,
    pub is_new_name_scheme: bool,
    pub has_auth_info: bool,
    pub has_recovery: bool,
    pub is_block_encoded: bool,
    pub is_first_volume: bool,
}

pub struct ArchiveHeaderParser;

impl ArchiveHeaderParser {
    pub const HEADER_SIZE: usize = 13;

    pub fn parse(buffer: &[u8]) -> Result<ArchiveHeader> {
        if buffer.len() < Self::HEADER_SIZE {
            return Err(RarError::BufferTooSmall {
                needed: Self::HEADER_SIZE,
                have: buffer.len(),
            });
        }

        let crc = u16::from_le_bytes([buffer[0], buffer[1]]);
        let header_type = buffer[2];
        let flags = u16::from_le_bytes([buffer[3], buffer[4]]);
        let size = u16::from_le_bytes([buffer[5], buffer[6]]);
        let reserved1 = u16::from_le_bytes([buffer[7], buffer[8]]);
        let reserved2 = u32::from_le_bytes([buffer[9], buffer[10], buffer[11], buffer[12]]);

        Ok(ArchiveHeader {
            crc,
            header_type,
            flags,
            size,
            reserved1,
            reserved2,
            has_volume_attributes: (flags & 0x0001) != 0,
            has_comment: (flags & 0x0002) != 0,
            is_locked: (flags & 0x0004) != 0,
            has_solid_attributes: (flags & 0x0008) != 0,
            is_new_name_scheme: (flags & 0x0010) != 0,
            has_auth_info: (flags & 0x0020) != 0,
            has_recovery: (flags & 0x0040) != 0,
            is_block_encoded: (flags & 0x0080) != 0,
            is_first_volume: (flags & 0x0100) != 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_archive_header() {
        let buffer = [
            0x00, 0x00, // crc
            0x73, // type (ARCHIVE_HEADER = 0x73)
            0x01, 0x00, // flags (has_volume_attributes)
            0x0D, 0x00, // size = 13
            0x00, 0x00, // reserved1
            0x00, 0x00, 0x00, 0x00, // reserved2
        ];
        let header = ArchiveHeaderParser::parse(&buffer).unwrap();
        assert_eq!(header.header_type, 0x73);
        assert!(header.has_volume_attributes);
        assert!(!header.has_comment);
    }
}
