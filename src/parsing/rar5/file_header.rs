//! RAR5 file header parser.
//!
//! The file header contains information about each file in the archive,
//! including name, size, compression method, and timestamps.

use super::{Rar5HeaderFlags, VintReader};
use crate::error::{RarError, Result};

/// Safely cast u64 to usize, returning an error on 32-bit overflow.
#[inline]
fn safe_usize(value: u64) -> Result<usize> {
    usize::try_from(value).map_err(|_| RarError::InvalidHeader)
}

/// RAR5 file flags (specific to file header).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rar5FileFlags {
    /// File is a directory
    pub is_directory: bool,
    /// File modification time is present
    pub has_mtime: bool,
    /// File CRC32 is present
    pub has_crc32: bool,
    /// Unpacked size is unknown
    pub unpacked_size_unknown: bool,
}

impl From<u64> for Rar5FileFlags {
    fn from(flags: u64) -> Self {
        Self {
            is_directory: flags & 0x0001 != 0,
            has_mtime: flags & 0x0002 != 0,
            has_crc32: flags & 0x0004 != 0,
            unpacked_size_unknown: flags & 0x0008 != 0,
        }
    }
}

/// RAR5 compression information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rar5CompressionInfo {
    /// Compression algorithm version
    pub version: u8,
    /// Solid flag
    pub is_solid: bool,
    /// Compression method (0 = store, 1-5 = compression levels)
    pub method: u8,
    /// Dictionary size as power of 2 (minimum 17 = 128KB)
    pub dict_size_log: u8,
}

impl From<u64> for Rar5CompressionInfo {
    fn from(info: u64) -> Self {
        Self {
            version: (info & 0x3F) as u8,
            is_solid: (info >> 6) & 1 != 0,
            method: ((info >> 7) & 0x07) as u8,
            dict_size_log: ((info >> 10) & 0x0F) as u8 + 17,
        }
    }
}

impl Rar5CompressionInfo {
    /// Get dictionary size in bytes.
    pub fn dict_size(&self) -> u64 {
        1u64 << self.dict_size_log
    }

    /// Check if file is stored (not compressed).
    pub fn is_stored(&self) -> bool {
        self.method == 0
    }
}

/// RAR5 host OS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Rar5HostOs {
    Windows = 0,
    Unix = 1,
}

impl TryFrom<u64> for Rar5HostOs {
    type Error = ();

    fn try_from(value: u64) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Windows),
            1 => Ok(Self::Unix),
            _ => Err(()),
        }
    }
}

/// Parsed RAR5 file header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rar5FileHeader {
    /// Header CRC32
    pub crc32: u32,
    /// Total header size in bytes
    pub header_size: u64,
    /// Common header flags
    pub header_flags: Rar5HeaderFlags,
    /// File-specific flags
    pub file_flags: Rar5FileFlags,
    /// Unpacked (original) file size
    pub unpacked_size: u64,
    /// File attributes
    pub attributes: u64,
    /// Modification time (if present)
    pub mtime: Option<u32>,
    /// File CRC32 (if present)
    pub file_crc32: Option<u32>,
    /// Compression info
    pub compression: Rar5CompressionInfo,
    /// Host OS
    pub host_os: Rar5HostOs,
    /// File name (UTF-8)
    pub name: String,
    /// Packed (compressed) data size
    pub packed_size: u64,
    /// Extra area data (if present)
    pub extra_area: Option<Vec<u8>>,
}

impl Rar5FileHeader {
    /// Check if file continues from previous volume.
    pub fn continues_from_previous(&self) -> bool {
        self.header_flags.split_before
    }

    /// Check if file continues in next volume.
    pub fn continues_in_next(&self) -> bool {
        self.header_flags.split_after
    }

    /// Check if file is stored (not compressed).
    pub fn is_stored(&self) -> bool {
        self.compression.is_stored()
    }

    /// Check if file is a directory.
    pub fn is_directory(&self) -> bool {
        self.file_flags.is_directory
    }

    /// Check if file is encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.header_flags.has_extra_area && self.encryption_info().is_some()
    }

    /// Get encryption info from extra area if present.
    /// Returns (encryption_data, flags) where flags indicate password check presence.
    pub fn encryption_info(&self) -> Option<&[u8]> {
        let extra = self.extra_area.as_ref()?;
        Self::find_extra_field(extra, 0x01) // FHEXTRA_CRYPT = 0x01
    }

    /// Find a specific extra field by type.
    fn find_extra_field(extra: &[u8], field_type: u64) -> Option<&[u8]> {
        let mut pos = 0;
        while pos < extra.len() {
            // Each extra field: size (vint), type (vint), data
            // size = total size of type + data (does NOT include the size vint itself)
            let mut reader = super::VintReader::new(&extra[pos..]);
            let size = reader.read()?;
            let size_vint_len = reader.position();
            let ftype = reader.read()?;
            let header_consumed = reader.position();

            if ftype == field_type {
                // Return the data after the type field
                let data_start = pos + header_consumed;
                let size_usize = size as usize;
                if size_usize as u64 != size {
                    return None; // Overflow on 32-bit
                }
                let data_end = pos + size_vint_len + size_usize;
                if data_end <= extra.len() {
                    return Some(&extra[data_start..data_end]);
                }
            }

            let size_usize = size as usize;
            if size_usize as u64 != size {
                return None;
            }
            pos += size_vint_len + size_usize;
        }
        None
    }
}

pub struct Rar5FileHeaderParser;

impl Rar5FileHeaderParser {
    /// Parse RAR5 file header from buffer.
    pub fn parse(buffer: &[u8]) -> Result<(Rar5FileHeader, usize)> {
        if buffer.len() < 12 {
            return Err(RarError::BufferTooSmall {
                needed: 12,
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

        // Read header type (vint) - should be 2 for file header
        let header_type = reader.read().ok_or(RarError::InvalidHeader)?;
        if header_type != 2 {
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

        // Read data size if present (packed size)
        let packed_size = if header_flags.has_data_area {
            reader.read().ok_or(RarError::InvalidHeader)?
        } else {
            0
        };

        // File-specific fields
        let file_flags_raw = reader.read().ok_or(RarError::InvalidHeader)?;
        let file_flags = Rar5FileFlags::from(file_flags_raw);

        let unpacked_size = reader.read().ok_or(RarError::InvalidHeader)?;
        let attributes = reader.read().ok_or(RarError::InvalidHeader)?;

        // Modification time (if present)
        let mtime = if file_flags.has_mtime {
            Some(reader.read_u32_le().ok_or(RarError::InvalidHeader)?)
        } else {
            None
        };

        // File CRC32 (if present)
        let file_crc32 = if file_flags.has_crc32 {
            Some(reader.read_u32_le().ok_or(RarError::InvalidHeader)?)
        } else {
            None
        };

        // Compression info
        let compression_raw = reader.read().ok_or(RarError::InvalidHeader)?;
        let compression = Rar5CompressionInfo::from(compression_raw);

        // Host OS
        let host_os_raw = reader.read().ok_or(RarError::InvalidHeader)?;
        let host_os = Rar5HostOs::try_from(host_os_raw).map_err(|()| RarError::InvalidHeader)?;

        // Name length and name
        let name_len = reader.read().ok_or(RarError::InvalidHeader)?;
        let name_bytes = reader
            .read_bytes(safe_usize(name_len)?)
            .ok_or(RarError::InvalidHeader)?;
        let name = String::from_utf8_lossy(name_bytes).into_owned();

        // Read extra area if present
        let extra_area = if extra_area_size > 0 {
            let extra_bytes = reader
                .read_bytes(safe_usize(extra_area_size)?)
                .ok_or(RarError::InvalidHeader)?;
            Some(extra_bytes.to_vec())
        } else {
            None
        };

        // Calculate total bytes consumed
        // header_size indicates bytes after the header_size vint itself
        let total_consumed = header_content_start + safe_usize(header_size)?;

        Ok((
            Rar5FileHeader {
                crc32,
                header_size,
                header_flags,
                file_flags,
                unpacked_size,
                attributes,
                mtime,
                file_crc32,
                compression,
                host_os,
                name,
                packed_size,
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
    fn test_compression_info() {
        // Layout: bits 0-5 = version, bit 6 = solid, bits 7-9 = method, bits 10-13 = dict_size
        // value 0 = version 0, not solid, method 0, dict_size 0 (+17 = 17)
        let info = Rar5CompressionInfo::from(0);
        assert_eq!(info.version, 0);
        assert_eq!(info.method, 0);
        assert_eq!(info.dict_size_log, 17);
        assert!(!info.is_solid);
        assert!(info.is_stored());
    }

    #[test]
    fn test_compression_with_method() {
        // method=3 at bits 7-9: 0b011 << 7 = 0x180
        let info = Rar5CompressionInfo::from(0x180);
        assert_eq!(info.method, 3);
    }

    #[test]
    fn test_stored_file() {
        let info = Rar5CompressionInfo::from(0);
        assert!(info.is_stored());
    }
}
