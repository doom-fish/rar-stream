//! RAR5 main archive header parser.
//!
//! The main archive header appears once after the signature and contains
//! archive-level flags and optional locator information.

use super::{Rar5HeaderFlags, VintReader};
use crate::error::{RarError, Result};

/// RAR5 archive flags (specific to main header).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rar5ArchiveFlags {
    /// Archive is part of multi-volume set
    pub is_volume: bool,
    /// Volume number field is present
    pub has_volume_number: bool,
    /// Solid archive
    pub is_solid: bool,
    /// Recovery record present
    pub has_recovery_record: bool,
    /// Archive is locked
    pub is_locked: bool,
}

impl From<u64> for Rar5ArchiveFlags {
    fn from(flags: u64) -> Self {
        Self {
            is_volume: flags & 0x0001 != 0,
            has_volume_number: flags & 0x0002 != 0,
            is_solid: flags & 0x0004 != 0,
            has_recovery_record: flags & 0x0008 != 0,
            is_locked: flags & 0x0010 != 0,
        }
    }
}

/// Parsed RAR5 main archive header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rar5ArchiveHeader {
    /// Header CRC32
    pub crc32: u32,
    /// Total header size in bytes
    pub header_size: u64,
    /// Common header flags
    pub header_flags: Rar5HeaderFlags,
    /// Archive-specific flags
    pub archive_flags: Rar5ArchiveFlags,
    /// Volume number (if multi-volume)
    pub volume_number: Option<u64>,
    /// Extra area data (if present)
    pub extra_area: Option<Vec<u8>>,
}

pub struct Rar5ArchiveHeaderParser;

impl Rar5ArchiveHeaderParser {
    /// Parse RAR5 main archive header from buffer.
    /// Buffer should start after the 8-byte RAR5 signature.
    pub fn parse(buffer: &[u8]) -> Result<(Rar5ArchiveHeader, usize)> {
        if buffer.len() < 8 {
            return Err(RarError::BufferTooSmall {
                needed: 8,
                have: buffer.len(),
            });
        }

        let mut reader = VintReader::new(buffer);

        // Read CRC32 (4 bytes, not vint)
        let crc32 = reader.read_u32_le().ok_or(RarError::InvalidHeader)?;

        // Read header size (vint) - this is the size of header content AFTER this vint
        let header_size = reader.read().ok_or(RarError::InvalidHeader)?;

        // Record position after reading header_size vint
        // Total consumed = this position + header_size
        let header_content_start = reader.position();

        // Read header type (vint) - should be 1 for main header
        let header_type = reader.read().ok_or(RarError::InvalidHeader)?;
        if header_type != 1 {
            return Err(RarError::InvalidHeader);
        }

        // Read header flags (vint)
        let header_flags_raw = reader.read().ok_or(RarError::InvalidHeader)?;
        let header_flags = Rar5HeaderFlags::from(header_flags_raw);

        // Read extra area size if present
        let extra_area_size = if header_flags.has_extra_area {
            reader.read().ok_or(RarError::InvalidHeader)?
        } else {
            0
        };

        // Read archive flags (vint)
        let archive_flags_raw = reader.read().ok_or(RarError::InvalidHeader)?;
        let archive_flags = Rar5ArchiveFlags::from(archive_flags_raw);

        // Read volume number if present
        let volume_number = if archive_flags.has_volume_number {
            Some(reader.read().ok_or(RarError::InvalidHeader)?)
        } else {
            None
        };

        // Read extra area if present
        let extra_area = if extra_area_size > 0 {
            let extra_bytes = reader
                .read_bytes(extra_area_size as usize)
                .ok_or(RarError::InvalidHeader)?;
            Some(extra_bytes.to_vec())
        } else {
            None
        };

        // Calculate total bytes consumed
        // header_size indicates bytes after the header_size vint itself
        let total_consumed = header_content_start + header_size as usize;

        Ok((
            Rar5ArchiveHeader {
                crc32,
                header_size,
                header_flags,
                archive_flags,
                volume_number,
                extra_area,
            },
            total_consumed,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_header() {
        // Minimal RAR5 main header:
        // CRC32 (4 bytes) + header_size (vint) + type (vint=1) + header_flags (vint) + archive_flags (vint)
        let header = [
            0x00, 0x00, 0x00, 0x00, // CRC32 (placeholder)
            0x05, // header_size = 5
            0x01, // type = 1 (main)
            0x00, // header_flags = 0
            0x00, // archive_flags = 0
            0x00, // padding
        ];

        let (parsed, consumed) = Rar5ArchiveHeaderParser::parse(&header).unwrap();
        assert_eq!(parsed.header_size, 5);
        assert!(!parsed.header_flags.has_extra_area);
        assert!(!parsed.archive_flags.is_volume);
        assert_eq!(consumed, 10); // 4 (CRC) + 1 (header_size vint) + 5 (content)
    }

    #[test]
    fn test_parse_real_rar5_archive_header() {
        // Real RAR5 archive header from test fixture (after 8-byte signature)
        // Signature: 52 61 72 21 1a 07 01 00
        // Header starts at offset 8: 33 92 b5 e5 0a 01 05 06 00 05 01 01 80 80 00
        let header = [
            0x33, 0x92, 0xb5, 0xe5, // CRC32
            0x0a, // header_size = 10 (vint)
            0x01, // type = 1 (main archive header)
            0x05, // header_flags = 5 (has_extra_area | skip_if_unknown)
            0x06, // extra_area_size = 6
            0x00, // archive_flags = 0
            0x05, 0x01, 0x01, 0x80, 0x80, 0x00, // extra area (6 bytes)
        ];

        let (parsed, consumed) = Rar5ArchiveHeaderParser::parse(&header).unwrap();
        assert_eq!(parsed.crc32, 0xe5b59233);
        assert_eq!(parsed.header_size, 10);
        assert!(parsed.header_flags.has_extra_area);
        assert!(!parsed.archive_flags.is_volume);
        assert_eq!(consumed, 15); // 4 (CRC) + 1 (header_size vint) + 10 (header content)
    }
}
