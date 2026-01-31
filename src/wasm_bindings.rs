//! WASM bindings for rar-stream.
//!
//! Provides browser-compatible API for RAR parsing and decompression.

use wasm_bindgen::prelude::*;

use crate::decompress::Rar29Decoder;
use crate::formats::Signature;
use crate::parsing::{MarkerHeaderParser, ArchiveHeaderParser, FileHeaderParser};

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

    // Build result object
    let obj = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&obj, &"name".into(), &file_header.name.into());
    let _ = js_sys::Reflect::set(&obj, &"packedSize".into(), &JsValue::from_f64(file_header.packed_size as f64));
    let _ = js_sys::Reflect::set(&obj, &"unpackedSize".into(), &JsValue::from_f64(file_header.unpacked_size as f64));
    let _ = js_sys::Reflect::set(&obj, &"method".into(), &JsValue::from_f64(file_header.method as f64));
    let _ = js_sys::Reflect::set(&obj, &"isCompressed".into(), &JsValue::from_bool(file_header.method != 0x30));

    Ok(obj.into())
}
