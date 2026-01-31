//! Cryptographic support for encrypted RAR archives.
//!
//! This module provides decryption support for both RAR4 and RAR5 encrypted archives.
//! RAR uses AES in CBC mode with format-specific key derivation functions.
//!
//! ## Security Overview
//!
//! | Format | Cipher | Key Size | KDF | Iterations |
//! |--------|--------|----------|-----|------------|
//! | RAR4 | AES-128-CBC | 128-bit | SHA-1 based | 262,144 (2^18) |
//! | RAR5 | AES-256-CBC | 256-bit | PBKDF2-HMAC-SHA256 | Configurable (default 2^15) |
//!
//! RAR5 is significantly more secure due to the larger key size and standard KDF.
//!
//! ## RAR5 Encryption
//!
//! RAR5 uses AES-256-CBC encryption with PBKDF2-HMAC-SHA256 for key derivation.
//! The iteration count is stored as log2 (e.g., 15 means 32,768 iterations).
//!
//! ```rust,ignore
//! use rar_stream::crypto::{Rar5Crypto, Rar5EncryptionInfo};
//!
//! // Parse encryption info from file header's extra area
//! let info = Rar5EncryptionInfo::parse(&encryption_data)?;
//!
//! // Derive key from password (slow due to PBKDF2)
//! let crypto = Rar5Crypto::derive_key("password", &info.salt, info.lg2_count);
//!
//! // Verify password (optional, if check value present)
//! if let Some(ref check) = info.psw_check {
//!     if !crypto.verify_password(check) {
//!         return Err("Wrong password");
//!     }
//! }
//!
//! // Decrypt data in-place
//! crypto.decrypt(&info.init_v, &mut encrypted_data)?;
//! ```
//!
//! ## RAR4 Encryption
//!
//! RAR4 uses AES-128-CBC encryption with a custom SHA-1 based key derivation.
//! The IV is derived alongside the key from the password and salt.
//!
//! ```rust,ignore
//! use rar_stream::crypto::Rar4Crypto;
//!
//! // Derive key and IV from password and 8-byte salt
//! let crypto = Rar4Crypto::derive_key("password", &salt);
//!
//! // Decrypt data in-place (uses derived IV)
//! crypto.decrypt(&mut encrypted_data)?;
//! ```
//!
//! ## Encrypted Headers (RAR5 only)
//!
//! RAR5 supports encrypting file headers with `rar -hp`. When headers are encrypted:
//!
//! 1. An encryption header appears right after the archive signature
//! 2. All subsequent headers are encrypted with AES-256-CBC
//! 3. File names and metadata cannot be read without the password
//!
//! ## Security Notes
//!
//! - **Password strength**: RAR's KDFs are computationally expensive, but strong
//!   passwords are still essential for security.
//! - **No authentication**: RAR encryption provides confidentiality but not integrity.
//!   Corrupt or tampered data may decrypt without error but produce garbage output.
//! - **Salt uniqueness**: Each file uses a unique random salt, preventing rainbow
//!   table attacks and ensuring identical files encrypt differently.

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
            CryptoError::UnsupportedVersion(v) => {
                write!(f, "Unsupported encryption version: {}", v)
            }
        }
    }
}

impl std::error::Error for CryptoError {}
