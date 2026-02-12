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
#[allow(dead_code)]
pub const CRYPT5_KDF_LG2_COUNT: u32 = 15;

/// Maximum allowed PBKDF2 iteration count (log2).
#[allow(dead_code)]
pub const CRYPT5_KDF_LG2_COUNT_MAX: u32 = 24;

/// RAR5 encryption information parsed from file header.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// - vint: version
    /// - vint: flags
    /// - 1 byte: lg2_count
    /// - 16 bytes: salt
    /// - 16 bytes: init_v
    /// - if flags & 0x01:
    ///   - 8 bytes: psw_check
    ///   - 4 bytes: psw_check_sum (SHA-256 of psw_check, first 4 bytes)
    pub fn parse(data: &[u8]) -> Result<Self, super::CryptoError> {
        use crate::parsing::rar5::VintReader;

        let mut reader = VintReader::new(data);

        let version = reader.read().ok_or(super::CryptoError::InvalidHeader)? as u8;
        if version != 0 {
            return Err(super::CryptoError::UnsupportedVersion(version));
        }

        let flags = reader.read().ok_or(super::CryptoError::InvalidHeader)? as u8;

        let lg2_bytes = reader
            .read_bytes(1)
            .ok_or(super::CryptoError::InvalidHeader)?;
        let lg2_count = lg2_bytes[0];

        let salt_bytes = reader
            .read_bytes(SIZE_SALT50)
            .ok_or(super::CryptoError::InvalidHeader)?;
        let mut salt = [0u8; SIZE_SALT50];
        salt.copy_from_slice(salt_bytes);

        let iv_bytes = reader
            .read_bytes(SIZE_INITV)
            .ok_or(super::CryptoError::InvalidHeader)?;
        let mut init_v = [0u8; SIZE_INITV];
        init_v.copy_from_slice(iv_bytes);

        let (psw_check, psw_check_sum) = if flags & 0x01 != 0 {
            let check_bytes = reader
                .read_bytes(SIZE_PSWCHECK)
                .ok_or(super::CryptoError::InvalidHeader)?;
            let mut check = [0u8; SIZE_PSWCHECK];
            check.copy_from_slice(check_bytes);

            let sum_bytes = reader
                .read_bytes(SIZE_PSWCHECK_CSUM)
                .ok_or(super::CryptoError::InvalidHeader)?;
            let mut sum = [0u8; SIZE_PSWCHECK_CSUM];
            sum.copy_from_slice(sum_bytes);

            // Validate psw_check_sum = SHA-256(psw_check)[0..4]
            use sha2::{Digest, Sha256};
            let hash = Sha256::digest(check);
            if hash[..SIZE_PSWCHECK_CSUM] != sum {
                return Err(super::CryptoError::InvalidHeader);
            }

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
#[derive(Clone, Debug)]
pub struct Rar5Crypto {
    /// Derived AES-256 key
    key: [u8; 32],
    /// Password check value (XOR of hash key iterations)
    psw_check_value: [u8; 32],
}

impl Rar5Crypto {
    /// Derive key from password using PBKDF2-HMAC-SHA256.
    ///
    /// RAR5 derives these values using PBKDF2-HMAC-SHA256:
    /// 1. Key (32 bytes) - for AES-256 encryption (at `iterations`)
    /// 2. Password check (32 bytes) - for password verification (at `iterations + 32`)
    ///
    /// Note: RAR5 spec also defines a hash key (at `iterations + 16`) for MAC
    /// checksums, but this implementation does not compute it since we don't
    /// verify header MACs.
    pub fn derive_key(password: &str, salt: &[u8; SIZE_SALT50], lg2_count: u8) -> Self {
        // Clamp to valid range to prevent shift overflow in release builds.
        // CRYPT5_KDF_LG2_COUNT_MAX is 24 per RAR5 spec; values above 31 would
        // wrap the u32 shift, undermining key derivation security.
        let lg2_count = lg2_count.min(CRYPT5_KDF_LG2_COUNT_MAX as u8);
        let iterations = 1u32 << lg2_count;

        // Derive key material
        // RAR uses PBKDF2 at different iteration counts for different values

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
        // Constant-time comparison to prevent timing side-channel attacks
        let mut diff = 0u8;
        for (a, b) in check.iter().zip(expected.iter()) {
            diff |= a ^ b;
        }
        diff == 0
    }

    /// Decrypt data in-place using AES-256-CBC.
    pub fn decrypt(
        &self,
        iv: &[u8; SIZE_INITV],
        data: &mut [u8],
    ) -> Result<(), super::CryptoError> {
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

impl Drop for Rar5Crypto {
    fn drop(&mut self) {
        // Zero sensitive key material to reduce exposure window.
        // Use write_volatile to prevent the compiler from optimizing this out.
        for byte in &mut self.key {
            unsafe { std::ptr::write_volatile(byte, 0) };
        }
        for byte in &mut self.psw_check_value {
            unsafe { std::ptr::write_volatile(byte, 0) };
        }
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
        use sha2::{Digest, Sha256};

        // Header with password check: version=0, flags=1, lg2_count=15, salt, iv, check, sum
        let mut data = vec![0u8; 47];
        data[0] = 0; // version
        data[1] = 1; // flags - password check present
        data[2] = 15; // lg2_count
                      // Fill check value (bytes 35..43)
        for i in 35..43 {
            data[i] = i as u8;
        }
        // Compute correct psw_check_sum = SHA-256(psw_check)[0..4]
        let hash = Sha256::digest(&data[35..43]);
        data[43..47].copy_from_slice(&hash[..4]);

        let info = Rar5EncryptionInfo::parse(&data).unwrap();
        assert_eq!(info.flags, 1);
        assert!(info.psw_check.is_some());
        assert!(info.psw_check_sum.is_some());
    }

    #[test]
    fn test_decrypt_encrypted_rar5() {
        use crate::parsing::rar5::file_header::Rar5FileHeaderParser;
        use crate::parsing::rar5::VintReader;

        // Read the encrypted RAR5 file (created with rar -ma5 -p"testpass")
        let data = std::fs::read("__fixtures__/encrypted/rar5-encrypted-v5.rar").unwrap();

        // Skip the 8-byte RAR5 signature
        let _after_sig = &data[8..];

        // Find the file header (type 2)
        let mut pos = 8; // After signature
        loop {
            assert!(pos + 7 <= data.len(), "Could not find file header");

            // Read header: CRC32 (4) + header_size (vint) + header_type (vint)
            let mut reader = VintReader::new(&data[pos + 4..]);
            let header_size = reader.read().unwrap();
            let header_type = reader.read().unwrap();

            if header_type == 2 {
                // File header found
                let (file_header, _) = Rar5FileHeaderParser::parse(&data[pos..]).unwrap();

                if file_header.is_encrypted() {
                    let enc_data = file_header.encryption_info().unwrap();

                    let enc_info = Rar5EncryptionInfo::parse(enc_data).unwrap();

                    // Derive key with correct password
                    let crypto =
                        Rar5Crypto::derive_key("testpass", &enc_info.salt, enc_info.lg2_count);

                    // Verify password if check value is present
                    if let Some(ref check) = enc_info.psw_check {
                        let valid = crypto.verify_password(check);
                        assert!(valid, "Password verification failed");
                    }

                    // Now decrypt the actual file data
                    // The packed data follows the file header
                    let header_total_size = 4 + 1 + header_size as usize; // CRC + size vint + content
                    let data_start = pos + header_total_size;
                    let data_end = data_start + file_header.packed_size as usize;

                    if data_end <= data.len() {
                        let encrypted_data = &data[data_start..data_end];

                        // Decrypt the data
                        let decrypted = crypto
                            .decrypt_to_vec(&enc_info.init_v, encrypted_data)
                            .unwrap();

                        // The decrypted data should be compressed - we can't verify the content
                        // directly without decompressing, but we can verify decryption succeeded
                        assert_eq!(decrypted.len(), encrypted_data.len());

                        // For stored files (method 0), we could verify content directly
                        // For compressed files, the decrypted data is still compressed
                    }
                }
                break;
            }

            // Move to next header: 4 bytes CRC + size vint length + header content
            let size_vint_len = {
                let mut r = VintReader::new(&data[pos + 4..]);
                r.read().unwrap();
                r.position()
            };
            pos += 4 + size_vint_len + header_size as usize;
        }
    }

    #[test]
    fn test_decrypt_stored_file() {
        use crate::parsing::rar5::file_header::Rar5FileHeaderParser;
        use crate::parsing::rar5::VintReader;

        // Read the stored encrypted RAR5 file (created with rar -ma5 -m0 -p"testpass")
        let data = std::fs::read("__fixtures__/encrypted/rar5-encrypted-stored.rar").unwrap();

        // Find the file header (type 2)
        let mut pos = 8; // After signature
        loop {
            assert!(pos + 7 <= data.len(), "Could not find file header");

            let mut reader = VintReader::new(&data[pos + 4..]);
            let header_size = reader.read().unwrap();
            let header_type = reader.read().unwrap();

            if header_type == 2 {
                let (file_header, consumed) = Rar5FileHeaderParser::parse(&data[pos..]).unwrap();

                assert!(file_header.is_encrypted());
                assert!(
                    file_header.is_stored(),
                    "File should be stored (uncompressed)"
                );

                let enc_data = file_header.encryption_info().unwrap();
                let enc_info = Rar5EncryptionInfo::parse(enc_data).unwrap();

                let crypto = Rar5Crypto::derive_key("testpass", &enc_info.salt, enc_info.lg2_count);

                // Verify password
                if let Some(ref check) = enc_info.psw_check {
                    assert!(
                        crypto.verify_password(check),
                        "Password verification failed"
                    );
                }

                // Decrypt the file data
                let data_start = pos + consumed;
                let data_end = data_start + file_header.packed_size as usize;
                let encrypted_data = &data[data_start..data_end];

                let decrypted = crypto
                    .decrypt_to_vec(&enc_info.init_v, encrypted_data)
                    .unwrap();

                // For stored files, decrypted data IS the original content (with padding)
                // The original file is "Hello, encrypted world!\n" (24 bytes)
                // Padded to 32 bytes (next multiple of 16)
                let expected = b"Hello, encrypted world!\n";
                assert!(
                    decrypted.starts_with(expected),
                    "Decrypted content doesn't match. Got: {:?}",
                    String::from_utf8_lossy(&decrypted[..expected.len().min(decrypted.len())])
                );
                break;
            }

            let size_vint_len = {
                let mut r = VintReader::new(&data[pos + 4..]);
                r.read().unwrap();
                r.position()
            };
            pos += 4 + size_vint_len + header_size as usize;
        }
    }
}
