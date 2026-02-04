//! RAR5 decompression module.
//!
//! RAR5 uses a different compression algorithm than RAR4.
//! This module provides decompression support for RAR5 archives.

#[cfg(feature = "parallel")]
pub mod bit_decoder;
#[cfg(not(feature = "parallel"))]
mod bit_decoder;

mod filter;
mod range_coder;

#[cfg(feature = "parallel")]
pub mod block_decoder;
#[cfg(not(feature = "parallel"))]
mod block_decoder;

mod decoder;

pub use decoder::Rar5Decoder;

#[cfg(feature = "parallel")]
pub use block_decoder::{ParallelConfig, Rar5BlockDecoder};
