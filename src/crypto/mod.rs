//! Cryptographic support for encrypted RAR archives.
//!
//! RAR5 uses AES-256-CBC with PBKDF2-HMAC-SHA256 key derivation.
//! RAR4 uses AES-256-CBC with a custom SHA-1 based KDF.

mod rar4;
mod rar5;

pub use rar4::{Rar4Crypto, Rar4EncryptionInfo};
pub use rar5::{Rar5Crypto, Rar5EncryptionInfo};

/// Encryption method used by the archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionMethod {
    /// RAR 5.0 encryption (AES-256-CBC, PBKDF2-HMAC-SHA256)
    Rar50,
    /// RAR 3.0/4.0 encryption (AES-128-CBC, custom SHA-1 KDF)
    Rar30,
    /// Unknown encryption method
    Unknown,
}

/// Error type for cryptographic operations.
#[derive(Debug, Clone)]
pub enum CryptoError {
    /// Wrong password provided
    WrongPassword,
    /// Invalid encryption header
    InvalidHeader,
    /// Decryption failed
    DecryptionFailed,
    /// Unsupported encryption version
    UnsupportedVersion(u8),
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::WrongPassword => write!(f, "Wrong password"),
            CryptoError::InvalidHeader => write!(f, "Invalid encryption header"),
            CryptoError::DecryptionFailed => write!(f, "Decryption failed"),
            CryptoError::UnsupportedVersion(v) => write!(f, "Unsupported encryption version: {}", v),
        }
    }
}

impl std::error::Error for CryptoError {}
