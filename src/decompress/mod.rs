//! RAR decompression algorithms.
//!
//! Implements LZSS + Huffman decompression for RAR archives.
//! Pure Rust implementation for WASM compatibility.

// Work-in-progress: PPM and RAR29 decompression not fully integrated yet
#![allow(dead_code)]

mod bit_reader;
mod huffman;
mod lzss;
mod ppm;
mod rar29;
pub mod rar5;
mod vm;

#[cfg(test)]
mod tests;

pub use bit_reader::BitReader;
pub use huffman::{HuffmanDecoder, HuffmanTable};
pub use lzss::LzssDecoder;
pub use ppm::PpmModel;
pub use rar29::Rar29Decoder;
pub use rar5::Rar5Decoder;
pub use vm::RarVM;

use std::fmt;
use std::io;

/// Decompression errors.
#[derive(Debug)]
pub enum DecompressError {
    UnexpectedEof,
    InvalidHuffmanCode,
    InvalidBackReference { offset: u32, position: u32 },
    BufferOverflow,
    UnsupportedMethod(u8),
    IncompleteData,
    Io(io::Error),
}

impl fmt::Display for DecompressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "Unexpected end of data"),
            Self::InvalidHuffmanCode => write!(f, "Invalid Huffman code"),
            Self::InvalidBackReference { offset, position } => {
                write!(
                    f,
                    "Invalid back reference: offset {} exceeds window position {}",
                    offset, position
                )
            }
            Self::BufferOverflow => write!(f, "Decompression buffer overflow"),
            Self::UnsupportedMethod(m) => write!(f, "Unsupported compression method: {}", m),
            Self::IncompleteData => write!(f, "Incomplete compressed data"),
            Self::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for DecompressError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for DecompressError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, DecompressError>;

/// Compression methods used in RAR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CompressionMethod {
    /// Store (no compression)
    Store = 0x30,
    /// Fastest compression
    Fastest = 0x31,
    /// Fast compression  
    Fast = 0x32,
    /// Normal compression
    Normal = 0x33,
    /// Good compression
    Good = 0x34,
    /// Best compression
    Best = 0x35,
}

impl CompressionMethod {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x30 => Some(Self::Store),
            0x31 => Some(Self::Fastest),
            0x32 => Some(Self::Fast),
            0x33 => Some(Self::Normal),
            0x34 => Some(Self::Good),
            0x35 => Some(Self::Best),
            _ => None,
        }
    }

    /// Whether this method requires decompression.
    pub fn needs_decompression(&self) -> bool {
        *self != Self::Store
    }
}
