//! RAR streaming library with NAPI bindings.
//!
//! Rust port of rar-stream for streaming files from RAR archives.
//! Optimized for video streaming with fast seeking via binary search.
//!
//! Supports RAR15 (RAR 1.5-4.x) and RAR50 (RAR 5.0+) formats.
//!
//! ## Features
//! - Core library has **zero dependencies**
//! - `async` - Async file reading with tokio
//! - `napi` - Node.js bindings
//! - `wasm` - Browser WASM bindings
//! - `crypto` - Encrypted archive support

// Note: unsafe_code = "warn" in Cargo.toml allows targeted unsafe for performance
// All unsafe blocks should be minimal and well-documented with SAFETY comments

mod crc32;
#[cfg(feature = "crypto")]
pub mod crypto;
pub mod decompress;
pub mod error;
mod file_media;
pub mod formats;
pub mod parsing;

// Async modules (require 'async' feature)
#[cfg(feature = "async")]
mod inner_file;
#[cfg(feature = "async")]
mod rar_file_chunk;
#[cfg(feature = "async")]
mod rar_files_package;

#[cfg(feature = "napi")]
mod napi_bindings;

#[cfg(feature = "wasm")]
mod wasm_bindings;

pub use error::RarError;
pub use file_media::{LocalFileMedia, ReadInterval};

#[cfg(feature = "async")]
pub use file_media::FileMedia;
#[cfg(feature = "async")]
pub use inner_file::{ChunkMapEntry, InnerFile, InnerFileStream, StreamChunkInfo};
#[cfg(feature = "async")]
pub use rar_file_chunk::RarFileChunk;
#[cfg(feature = "async")]
pub use rar_files_package::{ParseOptions, RarFilesPackage};

// Re-export decompression types
pub use decompress::{CompressionMethod, DecompressError, Rar29Decoder};

// Re-export NAPI bindings when feature is enabled
#[cfg(all(feature = "napi", not(feature = "wasm")))]
pub use napi_bindings::*;

// Re-export WASM bindings when feature is enabled
#[cfg(all(feature = "wasm", not(feature = "napi")))]
pub use wasm_bindings::*;
