//! Error types for RAR parsing and decompression.
//!
//! This module provides the [`RarError`] type which covers all possible errors
//! that can occur when parsing, decompressing, or decrypting RAR archives.
//!
//! ## Error Categories
//!
//! | Category | Errors | Description |
//! |----------|--------|-------------|
//! | Format | [`InvalidSignature`], [`InvalidHeader`] | File is not a valid RAR archive |
//! | Encryption | [`PasswordRequired`], [`DecryptionFailed`] | Encryption-related errors |
//! | Decompression | [`DecompressionNotSupported`] | Unsupported compression method |
//! | I/O | [`Io`], [`BufferTooSmall`] | Read/write errors |
//!
//! ## Example
//!
//! ```rust,ignore
//! use rar_stream::{RarFilesPackage, RarError};
//!
//! match package.parse(opts).await {
//!     Ok(files) => println!("Found {} files", files.len()),
//!     Err(RarError::InvalidSignature) => eprintln!("Not a RAR file"),
//!     Err(RarError::PasswordRequired) => eprintln!("Archive is encrypted"),
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```
//!
//! [`InvalidSignature`]: RarError::InvalidSignature
//! [`InvalidHeader`]: RarError::InvalidHeader
//! [`PasswordRequired`]: RarError::PasswordRequired
//! [`DecryptionFailed`]: RarError::DecryptionFailed
//! [`DecompressionNotSupported`]: RarError::DecompressionNotSupported
//! [`Io`]: RarError::Io
//! [`BufferTooSmall`]: RarError::BufferTooSmall

use std::fmt;
use std::io;

/// Error type for RAR operations.
///
/// This enum covers all possible errors that can occur when parsing,
/// decompressing, or decrypting RAR archives. It implements [`std::error::Error`]
/// for integration with the Rust error handling ecosystem.
///
/// # Example
///
/// ```rust,ignore
/// use rar_stream::RarError;
///
/// fn handle_error(err: RarError) {
///     match err {
///         RarError::InvalidSignature => {
///             // File doesn't start with RAR magic bytes
///         }
///         RarError::PasswordRequired => {
///             // Need to provide password in ParseOptions
///         }
///         RarError::Io(io_err) => {
///             // Underlying I/O error (file not found, permission denied, etc.)
///         }
///         _ => {}
///     }
/// }
/// ```
#[derive(Debug)]
pub enum RarError {
    /// The file does not have a valid RAR signature.
    ///
    /// RAR files must start with either:
    /// - RAR4: `Rar!\x1a\x07\x00` (7 bytes)
    /// - RAR5: `Rar!\x1a\x07\x01\x00` (8 bytes)
    InvalidSignature,

    /// A header in the archive is malformed or corrupt.
    ///
    /// This usually indicates file corruption or an incomplete download.
    InvalidHeader,

    /// An unknown or unsupported header type was encountered.
    ///
    /// The `u8` value is the header type byte. Standard types are:
    /// - `0x72` (114): Marker header
    /// - `0x73` (115): Archive header
    /// - `0x74` (116): File header
    /// - `0x7B` (123): End of archive
    InvalidHeaderType(u8),

    /// The compression method is not supported.
    ///
    /// The `u8` value is the method byte:
    /// - `0x30`: Store (no compression) - always supported
    /// - `0x31`-`0x35`: LZSS variants - supported
    /// - `0x36`+: Future methods - may not be supported
    DecompressionNotSupported(u8),

    /// The archive is encrypted but the `crypto` feature is not enabled.
    ///
    /// Enable the `crypto` feature in Cargo.toml:
    /// ```toml
    /// rar-stream = { version = "4", features = ["async", "crypto"] }
    /// ```
    EncryptedNotSupported,

    /// The archive is encrypted but no password was provided.
    ///
    /// Provide a password in [`ParseOptions`]:
    /// ```rust,ignore
    /// let opts = ParseOptions {
    ///     password: Some("secret".to_string()),
    ///     ..Default::default()
    /// };
    /// ```
    ///
    /// [`ParseOptions`]: crate::ParseOptions
    PasswordRequired,

    /// Decryption failed (wrong password or corrupt data).
    ///
    /// The `String` contains additional context about the failure.
    /// Common causes:
    /// - Incorrect password
    /// - Corrupt encrypted data
    /// - Truncated archive
    DecryptionFailed(String),

    /// The provided buffer is too small.
    ///
    /// This occurs when reading into a fixed-size buffer that cannot
    /// hold the required data.
    BufferTooSmall {
        /// Number of bytes needed.
        needed: usize,
        /// Number of bytes available.
        have: usize,
    },

    /// An invalid file offset was requested.
    ///
    /// This occurs when seeking beyond the end of a file or archive.
    InvalidOffset {
        /// The requested offset.
        offset: u64,
        /// The actual file length.
        length: u64,
    },

    /// An I/O error occurred.
    ///
    /// Wraps [`std::io::Error`] for file system operations.
    Io(io::Error),

    /// No files were found in the archive.
    ///
    /// The archive may be empty, or all files may have been filtered out.
    NoFilesFound,

    /// RAR5 format detected but a specific feature is not supported.
    ///
    /// This is a legacy error that should rarely occur with current versions.
    Rar5NotFullySupported,

    /// The archive has encrypted headers and requires a password to list files.
    ///
    /// RAR5 archives created with `rar -hp` encrypt both file data and headers.
    /// Without the correct password, even file names cannot be read.
    #[cfg(feature = "crypto")]
    EncryptedHeaders,
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
            #[cfg(feature = "crypto")]
            Self::EncryptedHeaders => {
                write!(f, "Archive has encrypted headers, password required to list files")
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
