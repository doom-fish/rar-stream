//! WASM bindings for rar-stream.
//!
//! Provides browser-compatible API for RAR parsing and decompression.

#![allow(missing_docs)]

use wasm_bindgen::prelude::*;

use crate::decompress::rar5::Rar5Decoder;
use crate::decompress::Rar29Decoder;
use crate::formats::Signature;
use crate::parsing::rar5::{Rar5ArchiveHeaderParser, Rar5FileHeaderParser};
use crate::parsing::{ArchiveHeaderParser, FileHeaderParser, MarkerHeaderParser};

/// Extract the first file from a RAR archive buffer.
/// Auto-detects RAR4/RAR5 format, parses headers, and decompresses.
/// Returns a JS object with `name`, `data` (Uint8Array), and `size`.
#[wasm_bindgen]
pub fn extract_file(archive: &[u8]) -> Result<JsValue, JsError> {
    match Signature::from_bytes(archive) {
        Some(Signature::Rar15) => extract_rar4_file(archive),
        Some(Signature::Rar50) => extract_rar5_file(archive),
        None => Err(JsError::new("Not a RAR archive")),
    }
}

fn extract_rar4_file(archive: &[u8]) -> Result<JsValue, JsError> {
    let marker = MarkerHeaderParser::parse(archive)
        .map_err(|e| JsError::new(&format!("Invalid marker: {e}")))?;
    let mut offset = marker.size as usize;

    let arch = ArchiveHeaderParser::parse(&archive[offset..])
        .map_err(|e| JsError::new(&format!("Invalid archive header: {e}")))?;
    offset += arch.size as usize;

    let fh = FileHeaderParser::parse(&archive[offset..])
        .map_err(|e| JsError::new(&format!("Invalid file header: {e}")))?;
    let data_offset = offset + fh.head_size as usize;
    let compressed = &archive[data_offset..data_offset + fh.packed_size as usize];

    let decompressed = if fh.method == 0x30 {
        compressed.to_vec()
    } else {
        let mut decoder = Rar29Decoder::new();
        decoder
            .decompress(compressed, fh.unpacked_size)
            .map_err(|e| JsError::new(&e.to_string()))?
    };

    build_extract_result(&fh.name, &decompressed)
}

fn extract_rar5_file(archive: &[u8]) -> Result<JsValue, JsError> {
    let mut offset = 8usize;

    let (_arch, arch_consumed) = Rar5ArchiveHeaderParser::parse(&archive[offset..])
        .map_err(|e| JsError::new(&format!("Invalid archive header: {e}")))?;
    offset += arch_consumed;

    let (fh, file_consumed) = Rar5FileHeaderParser::parse(&archive[offset..])
        .map_err(|e| JsError::new(&format!("Invalid file header: {e}")))?;
    let data_offset = offset + file_consumed;
    let compressed = &archive[data_offset..data_offset + fh.packed_size as usize];

    let decompressed = if fh.compression.is_stored() {
        compressed[..fh.unpacked_size as usize].to_vec()
    } else {
        let mut decoder = Rar5Decoder::with_dict_size(fh.compression.dict_size_log);
        decoder
            .decompress(compressed, fh.unpacked_size, fh.compression.method, false)
            .map_err(|e| JsError::new(&e.to_string()))?
    };

    build_extract_result(&fh.name, &decompressed)
}

fn build_extract_result(name: &str, data: &[u8]) -> Result<JsValue, JsError> {
    let obj = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&obj, &"name".into(), &JsValue::from_str(name));
    let _ = js_sys::Reflect::set(
        &obj,
        &"data".into(),
        &js_sys::Uint8Array::from(data).into(),
    );
    let _ = js_sys::Reflect::set(
        &obj,
        &"size".into(),
        &JsValue::from_f64(data.len() as f64),
    );
    Ok(obj.into())
}

/// Check if a buffer contains a RAR signature.
#[wasm_bindgen]
pub fn is_rar_archive(data: &[u8]) -> bool {
    Signature::from_bytes(data).is_some()
}

/// Get the RAR format version from a buffer.
/// Returns 15 for RAR 1.5-4.x, 50 for RAR 5.0+, or 0 if not a RAR archive.
#[wasm_bindgen]
pub fn get_rar_version(data: &[u8]) -> u8 {
    match Signature::from_bytes(data) {
        Some(Signature::Rar15) => 15,
        Some(Signature::Rar50) => 50,
        None => 0,
    }
}

/// WASM-compatible RAR5 decryptor.
#[cfg(feature = "crypto")]
#[wasm_bindgen]
pub struct WasmRar5Crypto {
    crypto: crate::crypto::Rar5Crypto,
}

#[cfg(feature = "crypto")]
#[wasm_bindgen]
impl WasmRar5Crypto {
    /// Create a new RAR5 decryptor with the given password, salt, and iteration count.
    /// The salt must be 16 bytes, lg2_count is the log2 of iteration count (typically 15).
    #[wasm_bindgen(constructor)]
    pub fn new(password: &str, salt: &[u8], lg2_count: u8) -> Result<WasmRar5Crypto, JsError> {
        if salt.len() != 16 {
            return Err(JsError::new("Salt must be exactly 16 bytes"));
        }
        let mut salt_arr = [0u8; 16];
        salt_arr.copy_from_slice(salt);
        Ok(Self {
            crypto: crate::crypto::Rar5Crypto::derive_key(password, &salt_arr, lg2_count),
        })
    }

    /// Decrypt data in place. The IV must be 16 bytes.
    /// Returns the decrypted data (same length as input, may include padding).
    #[wasm_bindgen]
    pub fn decrypt(&self, iv: &[u8], data: &[u8]) -> Result<Vec<u8>, JsError> {
        if iv.len() != 16 {
            return Err(JsError::new("IV must be exactly 16 bytes"));
        }
        let mut iv_arr = [0u8; 16];
        iv_arr.copy_from_slice(iv);
        self.crypto
            .decrypt_to_vec(&iv_arr, data)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Verify password using the check value from the encryption header.
    /// The check value is 12 bytes (8-byte check + 4-byte checksum).
    #[wasm_bindgen]
    pub fn verify_password(&self, check_value: &[u8]) -> bool {
        if check_value.len() < 8 {
            return false;
        }
        let expected: [u8; 8] = check_value[..8].try_into().unwrap();
        self.crypto.verify_password(&expected)
    }
}

/// WASM-compatible RAR decompressor.
#[wasm_bindgen]
pub struct WasmRarDecoder {
    decoder: Rar29Decoder,
    unpacked_size: u64,
}

#[wasm_bindgen]
impl WasmRarDecoder {
    /// Create a new decoder for the specified unpacked size.
    #[wasm_bindgen(constructor)]
    pub fn new(unpacked_size: u64) -> Self {
        Self {
            decoder: Rar29Decoder::new(),
            unpacked_size,
        }
    }

    /// Decompress a chunk of data.
    #[wasm_bindgen]
    pub fn decompress(&mut self, data: &[u8]) -> Result<Vec<u8>, JsError> {
        self.decoder
            .decompress(data, self.unpacked_size)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Get total bytes decompressed so far.
    #[wasm_bindgen]
    pub fn bytes_written(&self) -> u64 {
        self.decoder.bytes_written()
    }

    /// Check if decompression is complete.
    #[wasm_bindgen]
    pub fn is_complete(&self) -> bool {
        self.decoder.bytes_written() >= self.unpacked_size
    }

    /// Reset the decoder for a new file.
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.decoder.reset();
    }
}

/// WASM-compatible RAR5 decompressor.
#[wasm_bindgen]
pub struct WasmRar5Decoder {
    decoder: Rar5Decoder,
    unpacked_size: u64,
    method: u8,
    is_solid: bool,
}

#[wasm_bindgen]
impl WasmRar5Decoder {
    /// Create a new RAR5 decoder.
    /// `dict_size_log` is the dictionary size as log2 (e.g., 22 = 4MB, 25 = 32MB).
    /// `method` is the compression method (0 = stored, 1-5 = compression levels).
    #[wasm_bindgen(constructor)]
    pub fn new(unpacked_size: u64, dict_size_log: u8, method: u8, is_solid: bool) -> Self {
        Self {
            decoder: Rar5Decoder::with_dict_size(dict_size_log),
            unpacked_size,
            method,
            is_solid,
        }
    }

    /// Decompress RAR5 compressed data.
    #[wasm_bindgen]
    pub fn decompress(&mut self, data: &[u8]) -> Result<Vec<u8>, JsError> {
        self.decoder
            .decompress(data, self.unpacked_size, self.method, self.is_solid)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Reset the decoder for a new file.
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.decoder.reset();
    }
}

/// Parse all RAR4 file headers from a buffer.
/// Returns an array of file info objects for every file in the archive.
#[wasm_bindgen]
pub fn parse_rar_headers(data: &[u8]) -> Result<JsValue, JsError> {
    if data.len() < 50 {
        return Err(JsError::new("Buffer too small"));
    }

    let marker = MarkerHeaderParser::parse(data)
        .map_err(|e| JsError::new(&format!("Invalid marker: {}", e)))?;
    let mut offset = marker.size as usize;

    if data.len() < offset + ArchiveHeaderParser::HEADER_SIZE {
        return Err(JsError::new("Buffer too small for archive header"));
    }
    let archive = ArchiveHeaderParser::parse(&data[offset..])
        .map_err(|e| JsError::new(&format!("Invalid archive header: {}", e)))?;
    offset += archive.size as usize;

    let arr = js_sys::Array::new();

    while offset + 32 <= data.len() {
        let header_buf = &data[offset..];
        let file_header = match FileHeaderParser::parse(header_buf) {
            Ok(h) if h.header_type == 0x74 => h,
            _ => break,
        };

        let data_offset = offset + file_header.head_size as usize;

        let obj = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&obj, &"name".into(), &file_header.name.into());
        let _ = js_sys::Reflect::set(
            &obj,
            &"packedSize".into(),
            &JsValue::from_f64(file_header.packed_size as f64),
        );
        let _ = js_sys::Reflect::set(
            &obj,
            &"unpackedSize".into(),
            &JsValue::from_f64(file_header.unpacked_size as f64),
        );
        let _ = js_sys::Reflect::set(
            &obj,
            &"method".into(),
            &JsValue::from_f64(file_header.method as f64),
        );
        let _ = js_sys::Reflect::set(
            &obj,
            &"isCompressed".into(),
            &JsValue::from_bool(file_header.method != 0x30),
        );
        let _ = js_sys::Reflect::set(
            &obj,
            &"dataOffset".into(),
            &JsValue::from_f64(data_offset as f64),
        );

        arr.push(&obj);
        offset = data_offset + file_header.packed_size as usize;
    }

    Ok(arr.into())
}

/// Parse all RAR5 file headers from a buffer.
/// Returns an array of file info objects for every file in the archive.
#[wasm_bindgen]
pub fn parse_rar5_headers(data: &[u8]) -> Result<JsValue, JsError> {
    let sig = Signature::from_bytes(data);
    if sig != Some(Signature::Rar50) {
        return Err(JsError::new("Not a RAR5 archive"));
    }
    let mut offset = 8usize; // RAR5 signature length

    if offset + 4 >= data.len() {
        return Err(JsError::new("Buffer too small"));
    }
    let (_archive, archive_consumed) = Rar5ArchiveHeaderParser::parse(&data[offset..])
        .map_err(|e| JsError::new(&format!("Invalid archive header: {}", e)))?;
    offset += archive_consumed;

    let arr = js_sys::Array::new();

    while offset + 12 <= data.len() {
        let (file_header, file_consumed) = match Rar5FileHeaderParser::parse(&data[offset..]) {
            Ok(result) => result,
            Err(_) => break,
        };

        let data_offset = offset + file_consumed;

        let obj = js_sys::Object::new();
        let _ =
            js_sys::Reflect::set(&obj, &"name".into(), &JsValue::from_str(&file_header.name));
        let _ = js_sys::Reflect::set(
            &obj,
            &"packedSize".into(),
            &JsValue::from_f64(file_header.packed_size as f64),
        );
        let _ = js_sys::Reflect::set(
            &obj,
            &"unpackedSize".into(),
            &JsValue::from_f64(file_header.unpacked_size as f64),
        );
        let _ = js_sys::Reflect::set(
            &obj,
            &"method".into(),
            &JsValue::from_f64(file_header.compression.method as f64),
        );
        let _ = js_sys::Reflect::set(
            &obj,
            &"dictSizeLog".into(),
            &JsValue::from_f64(file_header.compression.dict_size_log as f64),
        );
        let _ = js_sys::Reflect::set(
            &obj,
            &"isCompressed".into(),
            &JsValue::from_bool(!file_header.compression.is_stored()),
        );
        let _ = js_sys::Reflect::set(
            &obj,
            &"isDirectory".into(),
            &JsValue::from_bool(file_header.is_directory()),
        );
        let _ = js_sys::Reflect::set(
            &obj,
            &"dataOffset".into(),
            &JsValue::from_f64(data_offset as f64),
        );

        arr.push(&obj);
        offset = data_offset + file_header.packed_size as usize;
    }

    Ok(arr.into())
}

/// Parse RAR file header information.
#[wasm_bindgen]
pub fn parse_rar_header(data: &[u8]) -> Result<JsValue, JsError> {
    if data.len() < 50 {
        return Err(JsError::new("Buffer too small"));
    }

    // Parse marker
    let marker = MarkerHeaderParser::parse(data)
        .map_err(|e| JsError::new(&format!("Invalid marker: {}", e)))?;
    let mut offset = marker.size as usize;

    // Parse archive header
    if data.len() < offset + ArchiveHeaderParser::HEADER_SIZE {
        return Err(JsError::new("Buffer too small for archive header"));
    }
    let archive = ArchiveHeaderParser::parse(&data[offset..])
        .map_err(|e| JsError::new(&format!("Invalid archive header: {}", e)))?;
    offset += archive.size as usize;

    // Parse first file header
    if data.len() < offset + 32 {
        return Err(JsError::new("Buffer too small for file header"));
    }
    let file_header = FileHeaderParser::parse(&data[offset..])
        .map_err(|e| JsError::new(&format!("Invalid file header: {}", e)))?;

    let data_offset = offset + file_header.head_size as usize;

    // Build result object
    let obj = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&obj, &"name".into(), &file_header.name.into());
    let _ = js_sys::Reflect::set(
        &obj,
        &"packedSize".into(),
        &JsValue::from_f64(file_header.packed_size as f64),
    );
    let _ = js_sys::Reflect::set(
        &obj,
        &"unpackedSize".into(),
        &JsValue::from_f64(file_header.unpacked_size as f64),
    );
    let _ = js_sys::Reflect::set(
        &obj,
        &"method".into(),
        &JsValue::from_f64(file_header.method as f64),
    );
    let _ = js_sys::Reflect::set(
        &obj,
        &"isCompressed".into(),
        &JsValue::from_bool(file_header.method != 0x30),
    );
    let _ = js_sys::Reflect::set(
        &obj,
        &"dataOffset".into(),
        &JsValue::from_f64(data_offset as f64),
    );

    Ok(obj.into())
}

/// Parse RAR5 file header information from a buffer.
/// The buffer should start at the RAR5 signature (Rar!\x1a\x07\x01\x00).
#[wasm_bindgen]
pub fn parse_rar5_header(data: &[u8]) -> Result<JsValue, JsError> {
    let sig = Signature::from_bytes(data);
    if sig != Some(Signature::Rar50) {
        return Err(JsError::new("Not a RAR5 archive"));
    }
    let mut offset = 8; // RAR5 signature length

    // Skip archive header
    if offset + 4 >= data.len() {
        return Err(JsError::new("Buffer too small"));
    }
    let (_archive, archive_consumed) = Rar5ArchiveHeaderParser::parse(&data[offset..])
        .map_err(|e| JsError::new(&format!("Invalid archive header: {}", e)))?;
    offset += archive_consumed;

    // Parse first file header
    if offset + 4 >= data.len() {
        return Err(JsError::new("Buffer too small for file header"));
    }
    let (file_header, file_consumed) = Rar5FileHeaderParser::parse(&data[offset..])
        .map_err(|e| JsError::new(&format!("Invalid file header: {}", e)))?;
    let data_offset = offset + file_consumed;

    let obj = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&obj, &"name".into(), &JsValue::from_str(&file_header.name));
    let _ = js_sys::Reflect::set(
        &obj,
        &"packedSize".into(),
        &JsValue::from_f64(file_header.packed_size as f64),
    );
    let _ = js_sys::Reflect::set(
        &obj,
        &"unpackedSize".into(),
        &JsValue::from_f64(file_header.unpacked_size as f64),
    );
    let _ = js_sys::Reflect::set(
        &obj,
        &"method".into(),
        &JsValue::from_f64(file_header.compression.method as f64),
    );
    let _ = js_sys::Reflect::set(
        &obj,
        &"dictSizeLog".into(),
        &JsValue::from_f64(file_header.compression.dict_size_log as f64),
    );
    let _ = js_sys::Reflect::set(
        &obj,
        &"isCompressed".into(),
        &JsValue::from_bool(!file_header.compression.is_stored()),
    );
    let _ = js_sys::Reflect::set(
        &obj,
        &"isDirectory".into(),
        &JsValue::from_bool(file_header.is_directory()),
    );
    let _ = js_sys::Reflect::set(&obj, &"version".into(), &JsValue::from_f64(50.0));
    let _ = js_sys::Reflect::set(
        &obj,
        &"dataOffset".into(),
        &JsValue::from_f64(data_offset as f64),
    );

    Ok(obj.into())
}
