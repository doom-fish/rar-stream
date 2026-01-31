//! RAR5 archive encryption header parser.
//!
//! The archive encryption header (type 4) appears in archives with encrypted headers.
//! All headers after this one are encrypted with AES-256-CBC.

use super::VintReader;
use crate::error::{RarError, Result};

/// Archive encryption header (type 4).
#[derive(Debug, Clone)]
pub struct Rar5EncryptionHeader {
    /// Encryption version (currently 0 for AES-256)
    pub version: u8,
    /// Encryption flags
    pub flags: u8,
    /// Log2 of PBKDF2 iteration count
    pub lg2_count: u8,
    /// 16-byte salt for key derivation
    pub salt: [u8; 16],
    /// Password check value (if FLAG_CHECK_PRESENT)
    pub check_value: Option<[u8; 12]>,
}

impl Rar5EncryptionHeader {
    /// Flag indicating password check value is present
    pub const FLAG_CHECK_PRESENT: u8 = 0x01;
}

pub struct Rar5EncryptionHeaderParser;

impl Rar5EncryptionHeaderParser {
    /// Parse an encryption header.
    /// Returns the header and number of bytes consumed.
    pub fn parse(data: &[u8]) -> Result<(Rar5EncryptionHeader, usize)> {
        if data.len() < 4 {
            return Err(RarError::InvalidHeader);
        }

        let mut pos = 0;

        // CRC32 (4 bytes)
        let _crc = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        pos += 4;

        // Header size (vint)
        let mut reader = VintReader::new(&data[pos..]);
        let header_size = reader.read().ok_or(RarError::InvalidHeader)?;
        pos += reader.position();

        // Header type (vint)
        let mut reader = VintReader::new(&data[pos..]);
        let header_type = reader.read().ok_or(RarError::InvalidHeader)?;
        pos += reader.position();

        if header_type != 4 {
            return Err(RarError::InvalidHeaderType(header_type as u8));
        }

        // Header flags (vint)
        let mut reader = VintReader::new(&data[pos..]);
        let _header_flags = reader.read().ok_or(RarError::InvalidHeader)?;
        pos += reader.position();

        // Encryption version (vint)
        let mut reader = VintReader::new(&data[pos..]);
        let version = reader.read().ok_or(RarError::InvalidHeader)? as u8;
        pos += reader.position();

        // Encryption flags (vint)
        let mut reader = VintReader::new(&data[pos..]);
        let flags = reader.read().ok_or(RarError::InvalidHeader)? as u8;
        pos += reader.position();

        // KDF count (1 byte)
        if pos >= data.len() {
            return Err(RarError::InvalidHeader);
        }
        let lg2_count = data[pos];
        pos += 1;

        // Salt (16 bytes)
        if pos + 16 > data.len() {
            return Err(RarError::InvalidHeader);
        }
        let mut salt = [0u8; 16];
        salt.copy_from_slice(&data[pos..pos + 16]);
        pos += 16;

        // Check value (12 bytes, optional)
        let check_value = if flags & Rar5EncryptionHeader::FLAG_CHECK_PRESENT != 0 {
            if pos + 12 > data.len() {
                return Err(RarError::InvalidHeader);
            }
            let mut check = [0u8; 12];
            check.copy_from_slice(&data[pos..pos + 12]);
            // pos += 12; // Not needed - last use of pos
            Some(check)
        } else {
            None
        };

        let total_consumed = 4 + 1 + header_size as usize; // CRC + size vint + content

        Ok((
            Rar5EncryptionHeader {
                version,
                flags,
                lg2_count,
                salt,
                check_value,
            },
            total_consumed,
        ))
    }

    /// Check if data starts with an encryption header.
    pub fn is_encryption_header(data: &[u8]) -> bool {
        if data.len() < 7 {
            return false;
        }

        // Skip CRC32 (4 bytes) and header size (1-3 bytes typically)
        let mut pos = 4;
        let mut reader = VintReader::new(&data[pos..]);
        if reader.read().is_none() {
            return false;
        }
        pos += reader.position();

        // Read header type
        let mut reader = VintReader::new(&data[pos..]);
        matches!(reader.read(), Some(4))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_encryption_header() {
        // Minimal encryption header: CRC + size(1) + type(4) + flags
        let mut data = vec![0u8; 20];
        data[4] = 5; // size = 5
        data[5] = 4; // type = 4 (encryption header)

        assert!(Rar5EncryptionHeaderParser::is_encryption_header(&data));
    }

    #[test]
    fn test_is_not_encryption_header() {
        // File header (type 2)
        let mut data = vec![0u8; 20];
        data[4] = 5; // size = 5
        data[5] = 2; // type = 2 (file header)

        assert!(!Rar5EncryptionHeaderParser::is_encryption_header(&data));
    }
}
