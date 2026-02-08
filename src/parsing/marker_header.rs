//! Marker header parser - RAR signature.
//!
//! The marker header is the first 7 bytes of a RAR file.
//! RAR4: 0x52 0x61 0x72 0x21 0x1A 0x07 0x00
//! RAR5: 0x52 0x61 0x72 0x21 0x1A 0x07 0x01 0x00

use crate::error::{RarError, Result};

/// RAR4 magic signature: `Rar!\x1a\x07\x00`.
///
/// ```
/// use rar_stream::parsing::marker_header::RAR4_SIGNATURE;
/// assert_eq!(&RAR4_SIGNATURE[..4], b"Rar!");
/// ```
pub const RAR4_SIGNATURE: [u8; 7] = [0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00];

/// RAR5 magic signature: `Rar!\x1a\x07\x01\x00`.
///
/// ```
/// use rar_stream::parsing::marker_header::RAR5_SIGNATURE;
/// assert_eq!(&RAR5_SIGNATURE[..4], b"Rar!");
/// assert_eq!(RAR5_SIGNATURE[6], 0x01); // distinguishes RAR5 from RAR4
/// ```
pub const RAR5_SIGNATURE: [u8; 8] = [0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x01, 0x00];

/// RAR archive version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RarVersion {
    /// RAR 4.x format (also known as RAR 2.9)
    #[default]
    Rar4,
    /// RAR 5.x format
    Rar5,
}

impl RarVersion {
    /// Returns the signature size for this version.
    pub fn signature_size(&self) -> usize {
        match self {
            Self::Rar4 => 7,
            Self::Rar5 => 8,
        }
    }
}

/// Parsed RAR marker header (the first header in every RAR archive).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkerHeader {
    /// Header CRC (RAR4 only; always 0x6152 for valid archives).
    pub crc: u16,
    /// Header type byte.
    pub header_type: u8,
    /// Header flags.
    pub flags: u16,
    /// Header size in bytes.
    pub size: u32,
    /// Detected RAR format version.
    pub version: RarVersion,
}

/// Parser for the RAR archive marker (signature) header.
pub struct MarkerHeaderParser;

impl MarkerHeaderParser {
    /// Size of the RAR4 marker header in bytes.
    pub const HEADER_SIZE: usize = 7;

    /// Detect RAR version from buffer without full parsing.
    ///
    /// Returns [`RarVersion::Rar4`] or [`RarVersion::Rar5`] based on the
    /// signature bytes, or [`RarError::InvalidSignature`] if the buffer
    /// doesn't start with a valid RAR signature.
    ///
    /// ```
    /// use rar_stream::parsing::marker_header::MarkerHeaderParser;
    /// use rar_stream::parsing::marker_header::RarVersion;
    ///
    /// let rar4 = [0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00];
    /// assert_eq!(MarkerHeaderParser::detect_version(&rar4).unwrap(), RarVersion::Rar4);
    ///
    /// let rar5 = [0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x01, 0x00];
    /// assert_eq!(MarkerHeaderParser::detect_version(&rar5).unwrap(), RarVersion::Rar5);
    ///
    /// assert!(MarkerHeaderParser::detect_version(b"not rar").is_err());
    /// ```
    pub fn detect_version(buffer: &[u8]) -> Result<RarVersion> {
        if buffer.len() >= 8 && buffer[..8] == RAR5_SIGNATURE {
            return Ok(RarVersion::Rar5);
        }
        if buffer.len() >= 7 && buffer[..7] == RAR4_SIGNATURE {
            return Ok(RarVersion::Rar4);
        }
        Err(RarError::InvalidSignature)
    }

    /// Parse marker header from buffer.
    /// The marker header is actually parsed as a generic RAR header.
    /// The "size" field tells us how many bytes this header consumes.
    pub fn parse(buffer: &[u8]) -> Result<MarkerHeader> {
        if buffer.len() < Self::HEADER_SIZE {
            return Err(RarError::BufferTooSmall {
                needed: Self::HEADER_SIZE,
                have: buffer.len(),
            });
        }

        // Check for RAR5 first (longer signature)
        if buffer.len() >= 8 && buffer[..8] == RAR5_SIGNATURE {
            return Ok(MarkerHeader {
                crc: 0,
                header_type: 0,
                flags: 0,
                size: 8,
                version: RarVersion::Rar5,
            });
        }

        // Verify RAR4 signature (first 7 bytes)
        if buffer[..7] != RAR4_SIGNATURE {
            return Err(RarError::InvalidSignature);
        }

        // Parse as generic header structure (RAR4)
        let crc = u16::from_le_bytes([buffer[0], buffer[1]]);
        let header_type = buffer[2];
        let flags = u16::from_le_bytes([buffer[3], buffer[4]]);
        let size = u16::from_le_bytes([buffer[5], buffer[6]]) as u32;

        Ok(MarkerHeader {
            crc,
            header_type,
            flags,
            size,
            version: RarVersion::Rar4,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rar4_marker() {
        // RAR4 signature + header
        let buffer = [
            0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00, // RAR4 signature
            0x00, 0x00, 0x00, 0x00, // padding for HEADER_SIZE
        ];
        let header = MarkerHeaderParser::parse(&buffer).unwrap();
        assert_eq!(header.header_type, b'r'); // 0x72
    }

    #[test]
    fn test_invalid_signature() {
        let buffer = [0x00; 11];
        assert!(matches!(
            MarkerHeaderParser::parse(&buffer),
            Err(RarError::InvalidSignature)
        ));
    }

    #[test]
    fn test_buffer_too_small() {
        let buffer = [0x52, 0x61, 0x72];
        assert!(matches!(
            MarkerHeaderParser::parse(&buffer),
            Err(RarError::BufferTooSmall { .. })
        ));
    }
}
