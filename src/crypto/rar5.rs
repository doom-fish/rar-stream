//! RAR5 encryption implementation.
//!
//! RAR5 uses:
//! - AES-256-CBC for encryption
//! - PBKDF2-HMAC-SHA256 for key derivation
//! - 16-byte salt
//! - Configurable iteration count (2^lg2_count)
//! - 8-byte password check value for fast verification

use aes::Aes256;
use cbc::cipher::{BlockDecryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

type Aes256CbcDec = cbc::Decryptor<Aes256>;

/// Size constants for RAR5 encryption.
pub const SIZE_SALT50: usize = 16;
pub const SIZE_INITV: usize = 16;
pub const SIZE_PSWCHECK: usize = 8;
pub const SIZE_PSWCHECK_CSUM: usize = 4;
pub const CRYPT_BLOCK_SIZE: usize = 16;

/// Default PBKDF2 iteration count (log2).
/// Actual iterations = 2^15 = 32768
pub const CRYPT5_KDF_LG2_COUNT: u32 = 15;

/// Maximum allowed PBKDF2 iteration count (log2).
pub const CRYPT5_KDF_LG2_COUNT_MAX: u32 = 24;

/// RAR5 encryption information parsed from file header.
#[derive(Debug, Clone)]
pub struct Rar5EncryptionInfo {
    /// Encryption version (must be 0)
    pub version: u8,
    /// Flags (0x01 = password check present, 0x02 = use MAC for checksums)
    pub flags: u8,
    /// Log2 of PBKDF2 iteration count
    pub lg2_count: u8,
    /// 16-byte salt
    pub salt: [u8; SIZE_SALT50],
    /// 16-byte initialization vector
    pub init_v: [u8; SIZE_INITV],
    /// Optional 8-byte password check value
    pub psw_check: Option<[u8; SIZE_PSWCHECK]>,
    /// Optional 4-byte password check sum
    pub psw_check_sum: Option<[u8; SIZE_PSWCHECK_CSUM]>,
}

impl Rar5EncryptionInfo {
    /// Parse encryption info from extra data.
    /// Format:
    /// - 1 byte: version
    /// - 1 byte: flags
    /// - 1 byte: lg2_count
    /// - 16 bytes: salt
    /// - 16 bytes: init_v
    /// - if flags & 0x01:
    ///   - 8 bytes: psw_check
    ///   - 4 bytes: psw_check_sum (CRC32 of first 3 bytes)
    pub fn parse(data: &[u8]) -> Result<Self, super::CryptoError> {
        if data.len() < 35 {
            return Err(super::CryptoError::InvalidHeader);
        }

        let version = data[0];
        if version != 0 {
            return Err(super::CryptoError::UnsupportedVersion(version));
        }

        let flags = data[1];
        let lg2_count = data[2];

        let mut salt = [0u8; SIZE_SALT50];
        salt.copy_from_slice(&data[3..19]);

        let mut init_v = [0u8; SIZE_INITV];
        init_v.copy_from_slice(&data[19..35]);

        let (psw_check, psw_check_sum) = if flags & 0x01 != 0 {
            if data.len() < 47 {
                return Err(super::CryptoError::InvalidHeader);
            }
            let mut check = [0u8; SIZE_PSWCHECK];
            check.copy_from_slice(&data[35..43]);
            let mut sum = [0u8; SIZE_PSWCHECK_CSUM];
            sum.copy_from_slice(&data[43..47]);
            (Some(check), Some(sum))
        } else {
            (None, None)
        };

        Ok(Self {
            version,
            flags,
            lg2_count,
            salt,
            init_v,
            psw_check,
            psw_check_sum,
        })
    }
}

/// RAR5 cryptographic operations.
pub struct Rar5Crypto {
    /// Derived AES-256 key
    key: [u8; 32],
    /// Password check value (XOR of hash key iterations)
    psw_check_value: [u8; 32],
}

impl Rar5Crypto {
    /// Derive key from password using PBKDF2-HMAC-SHA256.
    ///
    /// RAR5 derives 3 values:
    /// 1. Key (32 bytes) - for AES-256 encryption
    /// 2. Hash key (32 bytes) - for MAC checksums (iterations + 16)
    /// 3. Password check (32 bytes) - for password verification (iterations + 32)
    pub fn derive_key(password: &str, salt: &[u8; SIZE_SALT50], lg2_count: u8) -> Self {
        let iterations = 1u32 << lg2_count;

        // Derive key material: 32 bytes key + 32 bytes hash_key + 32 bytes psw_check
        // RAR uses a modified PBKDF2 that outputs these at different iteration counts
        // For simplicity, we compute the standard PBKDF2 and then the additional values

        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, iterations, &mut key);

        // Password check value - computed at iterations + 32
        // This is used to verify the password without decrypting
        let mut psw_check_value = [0u8; 32];
        pbkdf2_hmac::<Sha256>(
            password.as_bytes(),
            salt,
            iterations + 32,
            &mut psw_check_value,
        );

        Self {
            key,
            psw_check_value,
        }
    }

    /// Verify password using the stored check value.
    pub fn verify_password(&self, expected: &[u8; SIZE_PSWCHECK]) -> bool {
        // The check value is XOR of all bytes in psw_check_value,
        // folded into 8 bytes
        let mut check = [0u8; SIZE_PSWCHECK];
        for (i, &byte) in self.psw_check_value.iter().enumerate() {
            check[i % SIZE_PSWCHECK] ^= byte;
        }
        check == *expected
    }

    /// Decrypt data in-place using AES-256-CBC.
    pub fn decrypt(&self, iv: &[u8; SIZE_INITV], data: &mut [u8]) -> Result<(), super::CryptoError> {
        // Data must be a multiple of block size
        if data.len() % CRYPT_BLOCK_SIZE != 0 {
            return Err(super::CryptoError::DecryptionFailed);
        }

        let decryptor = Aes256CbcDec::new_from_slices(&self.key, iv)
            .map_err(|_| super::CryptoError::DecryptionFailed)?;

        decryptor
            .decrypt_padded_mut::<cbc::cipher::block_padding::NoPadding>(data)
            .map_err(|_| super::CryptoError::DecryptionFailed)?;

        Ok(())
    }

    /// Decrypt data to a new buffer.
    pub fn decrypt_to_vec(
        &self,
        iv: &[u8; SIZE_INITV],
        data: &[u8],
    ) -> Result<Vec<u8>, super::CryptoError> {
        let mut output = data.to_vec();
        self.decrypt(iv, &mut output)?;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key() {
        // Test that key derivation works
        let salt = [0u8; SIZE_SALT50];
        let crypto = Rar5Crypto::derive_key("password", &salt, 15);

        // Just verify it produces deterministic output
        let crypto2 = Rar5Crypto::derive_key("password", &salt, 15);
        assert_eq!(crypto.key, crypto2.key);
        assert_eq!(crypto.psw_check_value, crypto2.psw_check_value);

        // Different password should produce different key
        let crypto3 = Rar5Crypto::derive_key("different", &salt, 15);
        assert_ne!(crypto.key, crypto3.key);
    }

    #[test]
    fn test_parse_encryption_info() {
        // Minimal header: version=0, flags=0, lg2_count=15, salt, iv
        let mut data = vec![0u8; 35];
        data[0] = 0; // version
        data[1] = 0; // flags
        data[2] = 15; // lg2_count
        // salt and iv are zeros

        let info = Rar5EncryptionInfo::parse(&data).unwrap();
        assert_eq!(info.version, 0);
        assert_eq!(info.flags, 0);
        assert_eq!(info.lg2_count, 15);
        assert!(info.psw_check.is_none());
    }

    #[test]
    fn test_parse_encryption_info_with_check() {
        // Header with password check: version=0, flags=1, lg2_count=15, salt, iv, check, sum
        let mut data = vec![0u8; 47];
        data[0] = 0; // version
        data[1] = 1; // flags - password check present
        data[2] = 15; // lg2_count
        // Fill check value
        for i in 35..43 {
            data[i] = i as u8;
        }

        let info = Rar5EncryptionInfo::parse(&data).unwrap();
        assert_eq!(info.flags, 1);
        assert!(info.psw_check.is_some());
        assert!(info.psw_check_sum.is_some());
    }
}
