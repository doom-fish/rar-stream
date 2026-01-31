//! RAR5 decompression module.
//!
//! RAR5 uses a different compression algorithm than RAR4.
//! This module provides decompression support for RAR5 archives.

mod bit_decoder;
mod block_decoder;
mod decoder;
mod range_coder;

pub use decoder::Rar5Decoder;
