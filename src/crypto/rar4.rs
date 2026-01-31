//! RAR 3.x/4.x encryption support.
//!
//! RAR 3.x/4.x uses:
//! - AES-128-CBC encryption
//! - Custom SHA-1 based key derivation with 2^18 (262,144) iterations
//! - 8-byte salt
//! - No password verification (wrong password produces garbage)

use aes::cipher::{block_padding::NoPadding, BlockDecryptMut, KeyIvInit};
use sha1::{Digest, Sha1};

use super::CryptoError;

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

/// RAR 3.x/4.x encryption info from file header.
#[derive(Debug, Clone)]
pub struct Rar4EncryptionInfo {
    /// 8-byte salt from file header
    pub salt: [u8; 8],
}

/// RAR 3.x/4.x crypto handler.
#[derive(Clone)]
pub struct Rar4Crypto {
    /// Derived AES-256 key (32 bytes)
    key: [u8; 32],
    /// Derived IV (16 bytes)
    iv: [u8; 16],
}

impl Rar4Crypto {
    /// Derive key and IV from password and salt.
    ///
    /// The RAR4 KDF uses 2^18 iterations of SHA-1:
    /// - Password is encoded as UTF-16LE, concatenated with salt as seed
    /// - Each iteration: `SHA1.update(seed + counter[0..3])`
    /// - Counter is a 3-byte little-endian integer
    /// - Every 16384 (0x4000) iterations at j=0, extract byte 19 of digest as IV byte
    /// - Final SHA-1 digest (first 16 bytes) is the AES-128 key (with endian swap)
    pub fn derive_key(password: &str, salt: &[u8; 8]) -> Self {
        // Convert password to UTF-16LE and concatenate with salt
        let seed: Vec<u8> = password
            .encode_utf16()
            .flat_map(|c| c.to_le_bytes())
            .chain(salt.iter().copied())
            .collect();

        let mut hasher = Sha1::new();
        let mut iv = [0u8; 16];

        // 16 outer iterations (for IV bytes), each with 0x4000 inner iterations
        for i in 0..16 {
            for j in 0..0x4000u32 {
                let cnt = i * 0x4000 + j;
                let cnt_bytes = [cnt as u8, (cnt >> 8) as u8, (cnt >> 16) as u8];

                hasher.update(&seed);
                hasher.update(&cnt_bytes);

                // At the start of each outer iteration, extract IV byte
                if j == 0 {
                    let temp_digest = hasher.clone().finalize();
                    iv[i as usize] = temp_digest[19];
                }
            }
        }

        // Final digest - first 16 bytes become the key (with endian swap)
        let digest = hasher.finalize();
        let key_be: [u8; 16] = digest[..16].try_into().unwrap();

        // Swap endianness: convert from big-endian to little-endian
        // pack("<LLLL", *unpack(">LLLL", key_be))
        let key16 = Self::swap_key_endianness(&key_be);

        // Store as 32-byte key for AES-256 compatibility (but only use first 16 for AES-128)
        let mut key = [0u8; 32];
        key[..16].copy_from_slice(&key16);
        key[16..32].copy_from_slice(&key16);

        Self { key, iv }
    }

    /// Swap key bytes from big-endian to little-endian (4 x 32-bit words).
    fn swap_key_endianness(key_be: &[u8; 16]) -> [u8; 16] {
        let mut key_le = [0u8; 16];
        for i in 0..4 {
            // Each 4-byte word is swapped
            key_le[i * 4] = key_be[i * 4 + 3];
            key_le[i * 4 + 1] = key_be[i * 4 + 2];
            key_le[i * 4 + 2] = key_be[i * 4 + 1];
            key_le[i * 4 + 3] = key_be[i * 4];
        }
        key_le
    }

    /// Decrypt data in place using AES-128-CBC (using first 16 bytes of key).
    pub fn decrypt(&self, data: &mut [u8]) -> Result<(), CryptoError> {
        if data.is_empty() {
            return Ok(());
        }

        // Data must be multiple of 16 bytes
        if !data.len().is_multiple_of(16) {
            return Err(CryptoError::DecryptionFailed);
        }

        // Use only first 16 bytes of key for AES-128
        let decryptor = Aes128CbcDec::new_from_slices(&self.key[..16], &self.iv)
            .map_err(|_| CryptoError::DecryptionFailed)?;

        decryptor
            .decrypt_padded_mut::<NoPadding>(data)
            .map_err(|_| CryptoError::DecryptionFailed)?;

        Ok(())
    }

    /// Decrypt data returning a new Vec.
    pub fn decrypt_to_vec(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut decrypted = data.to_vec();
        self.decrypt(&mut decrypted)?;
        Ok(decrypted)
    }

    /// Get the derived IV.
    pub fn iv(&self) -> &[u8; 16] {
        &self.iv
    }

    /// Get the derived key.
    pub fn key(&self) -> &[u8; 32] {
        &self.key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key() {
        // Test key derivation with known password and salt
        let salt = [0x72, 0x8b, 0xe5, 0x8c, 0x22, 0x7f, 0x8d, 0xb4];
        let crypto = Rar4Crypto::derive_key("hello", &salt);

        // The key and IV should be deterministic
        assert_eq!(crypto.iv.len(), 16);
        assert_eq!(crypto.key.len(), 32);

        // Verify same password/salt produces same result
        let crypto2 = Rar4Crypto::derive_key("hello", &salt);
        assert_eq!(crypto.key, crypto2.key);
        assert_eq!(crypto.iv, crypto2.iv);
    }

    #[test]
    fn test_different_passwords_different_keys() {
        let salt = [0x72, 0x8b, 0xe5, 0x8c, 0x22, 0x7f, 0x8d, 0xb4];
        let crypto1 = Rar4Crypto::derive_key("hello", &salt);
        let crypto2 = Rar4Crypto::derive_key("world", &salt);

        assert_ne!(crypto1.key, crypto2.key);
        assert_ne!(crypto1.iv, crypto2.iv);
    }

    #[test]
    fn test_different_salts_different_keys() {
        let salt1 = [0x72, 0x8b, 0xe5, 0x8c, 0x22, 0x7f, 0x8d, 0xb4];
        let salt2 = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        let crypto1 = Rar4Crypto::derive_key("hello", &salt1);
        let crypto2 = Rar4Crypto::derive_key("hello", &salt2);

        assert_ne!(crypto1.key, crypto2.key);
        assert_ne!(crypto1.iv, crypto2.iv);
    }
}
