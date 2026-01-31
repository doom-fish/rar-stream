//! Error types for RAR parsing.

use std::fmt;
use std::io;

#[derive(Debug)]
pub enum RarError {
    InvalidSignature,
    InvalidHeaderType(u8),
    DecompressionNotSupported(u8),
    EncryptedNotSupported,
    BufferTooSmall { needed: usize, have: usize },
    InvalidOffset { offset: u64, length: u64 },
    Io(io::Error),
    NoFilesFound,
}

impl fmt::Display for RarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSignature => write!(f, "Invalid RAR signature"),
            Self::InvalidHeaderType(t) => write!(f, "Invalid header type: {}", t),
            Self::DecompressionNotSupported(m) => write!(f, "Decompression not supported (method: 0x{:02x})", m),
            Self::EncryptedNotSupported => write!(f, "Encrypted archives not supported"),
            Self::BufferTooSmall { needed, have } => write!(f, "Buffer too small: need {} bytes, have {}", needed, have),
            Self::InvalidOffset { offset, length } => write!(f, "Invalid offset: {} (file length: {})", offset, length),
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::NoFilesFound => write!(f, "No files found in archive"),
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
            crate::decompress::DecompressError::UnsupportedMethod(m) => Self::DecompressionNotSupported(m),
            _ => Self::Io(io::Error::new(io::ErrorKind::InvalidData, e.to_string())),
        }
    }
}

pub type Result<T> = std::result::Result<T, RarError>;
