//! Cryptographic support for encrypted RAR archives.
//!
//! This module provides decryption support for both RAR4 and RAR5 encrypted archives.
//!
//! ## RAR5 Encryption
//!
//! RAR5 uses AES-256-CBC encryption with PBKDF2-HMAC-SHA256 for key derivation.
//! The iteration count is configurable (default 2^15 = 32768 rounds).
//!
//! ```rust,ignore
//! use rar_stream::crypto::{Rar5Crypto, Rar5EncryptionInfo};
//!
//! let info = Rar5EncryptionInfo::parse(&encryption_data)?;
//! let crypto = Rar5Crypto::derive_key("password", &info.salt, info.lg2_count);
//! crypto.decrypt(&info.init_v, &mut encrypted_data)?;
//! ```
//!
//! ## RAR4 Encryption
//!
//! RAR4 uses AES-128-CBC encryption with a custom SHA-1 based key derivation
//! (262,144 iterations). The IV is derived alongside the key.
//!
//! ```rust,ignore
//! use rar_stream::crypto::Rar4Crypto;
//!
//! let crypto = Rar4Crypto::derive_key("password", &salt);
//! crypto.decrypt(&mut encrypted_data)?;
//! ```

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
#[derive(Debug, Clone, PartialEq, Eq)]
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
