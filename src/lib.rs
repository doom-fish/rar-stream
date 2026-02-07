//! # rar-stream
//!
//! A high-performance RAR archive streaming library for Rust, Node.js, and browsers.
//!
//! This library provides streaming access to files within RAR archives, optimized for
//! video streaming with fast seeking via binary search. It supports both RAR4 (1.5-4.x)
//! and RAR5 (5.0+) formats with full decompression and optional encryption support.
//!
//! ## Features
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `crypto` | No | Encrypted archive support (AES-256 for RAR5, AES-128 for RAR4) |
//! | `napi` | No | Node.js native bindings via napi-rs (includes async I/O) |
//! | `wasm` | No | Browser WebAssembly bindings |
//!
//! ## Supported Formats
//!
//! | Format | Versions | Compression | Encryption |
//! |--------|----------|-------------|------------|
//! | RAR4 | 1.5-4.x | LZSS, PPMd | AES-128-CBC (SHA-1 KDF) |
//! | RAR5 | 5.0+ | LZSS + filters | AES-256-CBC (PBKDF2-HMAC-SHA256) |
//!
//! ## Architecture
//!
//! The library is organized into layers:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │  Application Layer (async feature)                  │
//! │  RarFilesPackage → InnerFile → read_to_end()        │
//! ├─────────────────────────────────────────────────────┤
//! │  Parsing Layer                                      │
//! │  MarkerHeader → ArchiveHeader → FileHeader          │
//! ├─────────────────────────────────────────────────────┤
//! │  Decompression Layer                                │
//! │  Rar29Decoder (RAR4) / Rar5Decoder (RAR5)           │
//! ├─────────────────────────────────────────────────────┤
//! │  Crypto Layer (crypto feature)                      │
//! │  Rar4Crypto (AES-128) / Rar5Crypto (AES-256)        │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ### High-Level API (requires `napi` feature for Node.js)
//!
//! ```rust,ignore
//! use rar_stream::{RarFilesPackage, ParseOptions, LocalFileMedia, FileMedia};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Open a RAR archive
//!     let file: Arc<dyn FileMedia> = Arc::new(LocalFileMedia::new("archive.rar")?);
//!     let package = RarFilesPackage::new(vec![file]);
//!
//!     // Parse and list files
//!     let files = package.parse(ParseOptions::default()).await?;
//!     for f in &files {
//!         println!("{}: {} bytes", f.name, f.length);
//!     }
//!
//!     // Read file content (automatically decompresses)
//!     let content = files[0].read_to_end().await?;
//!     Ok(())
//! }
//! ```
//!
//! ### Low-Level Decompression (no features required)
//!
//! ```rust
//! use rar_stream::Rar29Decoder;
//!
//! // Create a decoder for RAR4 LZSS data
//! let mut decoder = Rar29Decoder::new();
//!
//! // Decompress raw compressed data (obtained from file header)
//! // let decompressed = decoder.decompress(&compressed_data, expected_size)?;
//! ```
//!
//! ## Encrypted Archives
//!
//! With the `crypto` feature, you can read encrypted archives:
//!
//! ```rust,ignore
//! use rar_stream::{RarFilesPackage, ParseOptions};
//!
//! let opts = ParseOptions {
//!     password: Some("secret".to_string()),
//!     ..Default::default()
//! };
//! let files = package.parse(opts).await?;
//!
//! // Content is automatically decrypted and decompressed
//! let content = files[0].read_decompressed().await?;
//! ```
//!
//! ## Error Handling
//!
//! All operations return [`Result<T, RarError>`]. Common errors include:
//!
//! - [`RarError::InvalidSignature`] - Not a valid RAR file
//! - [`RarError::PasswordRequired`] - Encrypted archive, no password provided
//! - [`RarError::DecryptionFailed`] - Wrong password or corrupt data
//! - [`RarError::DecompressionNotSupported`] - Unsupported compression method
//!
//! ## Module Overview
//!
//! - [`error`] - Error types for all operations
//! - [`parsing`] - RAR header parsing (both RAR4 and RAR5)
//! - [`decompress`] - Decompression algorithms (LZSS, PPMd, filters)
//! - [`crypto`] - Encryption/decryption (requires `crypto` feature)
//! - [`formats`] - Low-level format constants and utilities
//!
//! ## Performance Notes
//!
//! - **Streaming**: Files are read on-demand, not loaded entirely into memory
//! - **Binary search**: Chunk lookup uses binary search for O(log n) seeking
//! - **Zero-copy parsing**: Headers are parsed without unnecessary allocations
//! - **Cached decompression**: Decompressed data is cached for repeated reads
//!
//! ## Browser/WASM Usage
//!
//! With the `wasm` feature, the library compiles to WebAssembly for browser use.
//! See the npm package documentation for JavaScript API details.

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

// Async modules (require 'napi' feature)
#[cfg(feature = "napi")]
mod inner_file;
#[cfg(feature = "napi")]
mod rar_file_chunk;
#[cfg(feature = "napi")]
mod rar_files_package;

#[cfg(feature = "napi")]
mod napi_bindings;

#[cfg(feature = "wasm")]
mod wasm_bindings;

pub use error::RarError;
pub use file_media::{LocalFileMedia, ReadInterval};

#[cfg(feature = "napi")]
pub use file_media::FileMedia;
#[cfg(feature = "napi")]
pub use inner_file::{ChunkMapEntry, InnerFile, InnerFileStream, StreamChunkInfo};
#[cfg(feature = "napi")]
pub use rar_file_chunk::RarFileChunk;
#[cfg(feature = "napi")]
pub use rar_files_package::{ParseOptions, RarFilesPackage};

// Re-export decompression types
pub use decompress::{CompressionMethod, DecompressError, Rar29Decoder};

// Re-export NAPI bindings when feature is enabled
#[cfg(all(feature = "napi", not(feature = "wasm")))]
pub use napi_bindings::*;

// Re-export WASM bindings when feature is enabled
#[cfg(all(feature = "wasm", not(feature = "napi")))]
pub use wasm_bindings::*;
