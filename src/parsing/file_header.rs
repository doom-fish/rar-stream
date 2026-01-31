//! File header parser.
//!
//! Each file in a RAR archive has a file header that describes
//! the file's name, size, compression method, etc.

use crate::error::{RarError, Result};

/// File header type constant.
pub const FILE_HEADER_TYPE: u8 = 0x74; // 116

#[derive(Debug, Clone)]
pub struct FileHeader {
    pub crc: u16,
    pub header_type: u8,
    pub flags: u16,
    pub head_size: u16,
    pub packed_size: u64,
    pub unpacked_size: u64,
    pub host_os: u8,
    pub file_crc: u32,
    pub timestamp: u32,
    pub version: u8,
    pub method: u8,
    pub name_size: u16,
    pub attributes: u32,
    pub name: String,
    // Parsed flags
    pub continues_from_previous: bool,
    pub continues_in_next: bool,
    pub is_encrypted: bool,
    pub has_comment: bool,
    pub has_info_from_previous: bool,
    pub has_high_size: bool,
    pub has_special_name: bool,
    pub has_salt: bool,
    pub is_old_version: bool,
    pub has_extended_time: bool,
}

pub struct FileHeaderParser;

impl FileHeaderParser {
    /// Maximum header size to read (includes variable-length filename).
    pub const HEADER_SIZE: usize = 280;
    /// Minimum fixed header size before filename.
    const MIN_HEADER_SIZE: usize = 32;

    pub fn parse(buffer: &[u8]) -> Result<FileHeader> {
        if buffer.len() < Self::MIN_HEADER_SIZE {
            return Err(RarError::BufferTooSmall {
                needed: Self::MIN_HEADER_SIZE,
                have: buffer.len(),
            });
        }

        let mut offset = 0;

        let crc = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]);
        offset += 2;

        let header_type = buffer[offset];
        offset += 1;

        let flags = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]);
        offset += 2;

        let head_size = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]);
        offset += 2;

        let mut packed_size = u32::from_le_bytes([
            buffer[offset],
            buffer[offset + 1],
            buffer[offset + 2],
            buffer[offset + 3],
        ]) as u64;
        offset += 4;

        let mut unpacked_size = u32::from_le_bytes([
            buffer[offset],
            buffer[offset + 1],
            buffer[offset + 2],
            buffer[offset + 3],
        ]) as u64;
        offset += 4;

        let host_os = buffer[offset];
        offset += 1;

        let file_crc = u32::from_le_bytes([
            buffer[offset],
            buffer[offset + 1],
            buffer[offset + 2],
            buffer[offset + 3],
        ]);
        offset += 4;

        let timestamp = u32::from_le_bytes([
            buffer[offset],
            buffer[offset + 1],
            buffer[offset + 2],
            buffer[offset + 3],
        ]);
        offset += 4;

        let version = buffer[offset];
        offset += 1;

        let method = buffer[offset];
        offset += 1;

        let name_size = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]);
        offset += 2;

        let attributes = u32::from_le_bytes([
            buffer[offset],
            buffer[offset + 1],
            buffer[offset + 2],
            buffer[offset + 3],
        ]);
        offset += 4;

        // Parse flags
        let continues_from_previous = (flags & 0x01) != 0;
        let continues_in_next = (flags & 0x02) != 0;
        let is_encrypted = (flags & 0x04) != 0;
        let has_comment = (flags & 0x08) != 0;
        let has_info_from_previous = (flags & 0x10) != 0;
        let has_high_size = (flags & 0x100) != 0;
        let has_special_name = (flags & 0x200) != 0;
        let has_salt = (flags & 0x400) != 0;
        let is_old_version = (flags & 0x800) != 0;
        let has_extended_time = (flags & 0x1000) != 0;

        // Handle 64-bit sizes
        if has_high_size && buffer.len() >= offset + 8 {
            let high_packed = u32::from_le_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]) as u64;
            offset += 4;
            let high_unpacked = u32::from_le_bytes([
                buffer[offset],
                buffer[offset + 1],
                buffer[offset + 2],
                buffer[offset + 3],
            ]) as u64;
            offset += 4;

            packed_size |= high_packed << 32;
            unpacked_size |= high_unpacked << 32;
        }

        // Parse filename
        let name_end = offset + name_size as usize;
        if buffer.len() < name_end {
            return Err(RarError::BufferTooSmall {
                needed: name_end,
                have: buffer.len(),
            });
        }
        let name = String::from_utf8_lossy(&buffer[offset..name_end]).to_string();

        Ok(FileHeader {
            crc,
            header_type,
            flags,
            head_size,
            packed_size,
            unpacked_size,
            host_os,
            file_crc,
            timestamp,
            version,
            method,
            name_size,
            attributes,
            name,
            continues_from_previous,
            continues_in_next,
            is_encrypted,
            has_comment,
            has_info_from_previous,
            has_high_size,
            has_special_name,
            has_salt,
            is_old_version,
            has_extended_time,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file_header() {
        // Minimal file header with a 4-byte filename "test"
        let mut buffer = vec![0u8; 36];
        buffer[2] = FILE_HEADER_TYPE; // type
        buffer[5] = 36; // head_size low byte
        buffer[26] = 4; // name_size = 4
        buffer[32] = b't';
        buffer[33] = b'e';
        buffer[34] = b's';
        buffer[35] = b't';

        let header = FileHeaderParser::parse(&buffer).unwrap();
        assert_eq!(header.header_type, FILE_HEADER_TYPE);
        assert_eq!(header.name, "test");
    }

    #[test]
    fn test_compression_method() {
        let mut buffer = vec![0u8; 36];
        buffer[2] = FILE_HEADER_TYPE;
        buffer[5] = 36;
        buffer[25] = 0x30; // method = Store (no compression) - at offset 25
        buffer[26] = 4;    // name_size low byte
        buffer[32..36].copy_from_slice(b"test");

        let header = FileHeaderParser::parse(&buffer).unwrap();
        assert_eq!(header.method, 0x30); // Store method
    }
}
