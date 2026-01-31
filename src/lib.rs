//! # rar-stream
//!
//! A high-performance RAR archive streaming library for Rust, Node.js, and browsers.
//!
//! This library provides streaming access to files within RAR archives, optimized for
//! video streaming with fast seeking via binary search. It supports both RAR4 (1.5-4.x)
//! and RAR5 (5.0+) formats.
//!
//! ## Features
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `async` | Async file reading with tokio (enables [`RarFilesPackage`], [`InnerFile`]) |
//! | `crypto` | Encrypted archive support (AES-256 for RAR5, AES-128 for RAR4) |
//! | `napi` | Node.js native bindings |
//! | `wasm` | Browser WebAssembly bindings |
//!
//! ## Supported Formats
//!
//! - **RAR4** (versions 1.5-4.x): LZSS, PPMd compression, AES-128-CBC encryption
//! - **RAR5** (version 5.0+): LZSS with filters, AES-256-CBC encryption with PBKDF2
//!
//! ## Quick Start
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
//!     // Read file content
//!     let content = files[0].read_to_end().await?;
//!     Ok(())
//! }
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
//! let decrypted = files[0].read_decompressed().await?;
//! ```
//!
//! ## Decompression Only
//!
//! For low-level decompression without async I/O:
//!
//! ```rust
//! use rar_stream::{Rar29Decoder, CompressionMethod};
//!
//! // Decompress RAR4 LZSS data
//! let mut decoder = Rar29Decoder::new();
//! // decoder.decompress(&compressed_data, expected_size)?;
//! ```
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
