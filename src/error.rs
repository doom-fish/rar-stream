//! Error types for RAR parsing and decompression.
//!
//! The main error type is [`RarError`], which covers all possible errors
//! that can occur when parsing or extracting RAR archives.

use std::fmt;
use std::io;

/// Error type for RAR operations.
///
/// This enum covers all possible errors that can occur when parsing,
/// decompressing, or decrypting RAR archives.
#[derive(Debug)]
pub enum RarError {
    /// The file does not have a valid RAR signature.
    InvalidSignature,
    /// A header in the archive is malformed or corrupt.
    InvalidHeader,
    /// An unknown or unsupported header type was encountered.
    InvalidHeaderType(u8),
    /// The compression method is not supported.
    DecompressionNotSupported(u8),
    /// The archive is encrypted but the `crypto` feature is not enabled.
    EncryptedNotSupported,
    /// The archive is encrypted but no password was provided.
    PasswordRequired,
    /// Decryption failed (wrong password or corrupt data).
    DecryptionFailed(String),
    /// The provided buffer is too small.
    BufferTooSmall {
        /// Bytes needed
        needed: usize,
        /// Bytes available
        have: usize,
    },
    /// An invalid file offset was requested.
    InvalidOffset {
        /// Requested offset
        offset: u64,
        /// File length
        length: u64,
    },
    /// An I/O error occurred.
    Io(io::Error),
    /// No files were found in the archive.
    NoFilesFound,
    /// RAR5 format detected but a specific feature is not supported.
    Rar5NotFullySupported,
}

impl fmt::Display for RarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSignature => write!(f, "Invalid RAR signature"),
            Self::InvalidHeader => write!(f, "Invalid or malformed header"),
            Self::InvalidHeaderType(t) => write!(f, "Invalid header type: {}", t),
            Self::DecompressionNotSupported(m) => {
                write!(f, "Decompression not supported (method: 0x{:02x})", m)
            }
            Self::EncryptedNotSupported => write!(f, "Encrypted archives not supported"),
            Self::PasswordRequired => write!(f, "Password required for encrypted file"),
            Self::DecryptionFailed(msg) => write!(f, "Decryption failed: {}", msg),
            Self::BufferTooSmall { needed, have } => {
                write!(f, "Buffer too small: need {} bytes, have {}", needed, have)
            }
            Self::InvalidOffset { offset, length } => {
                write!(f, "Invalid offset: {} (file length: {})", offset, length)
            }
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::NoFilesFound => write!(f, "No files found in archive"),
            Self::Rar5NotFullySupported => {
                write!(
                    f,
                    "RAR5 format detected but decompression not yet supported"
                )
            }
        }
    }
}

impl std::error::Error for RarError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for RarError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<crate::decompress::DecompressError> for RarError {
    fn from(e: crate::decompress::DecompressError) -> Self {
        match e {
            crate::decompress::DecompressError::UnsupportedMethod(m) => {
                Self::DecompressionNotSupported(m)
            }
            _ => Self::Io(io::Error::new(io::ErrorKind::InvalidData, e.to_string())),
        }
    }
}

pub type Result<T> = std::result::Result<T, RarError>;
