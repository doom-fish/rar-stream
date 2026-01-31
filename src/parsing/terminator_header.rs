//! Terminator header parser.
//!
//! The terminator header marks the end of a RAR archive.

use crate::error::{RarError, Result};

pub struct TerminatorHeaderParser;

impl TerminatorHeaderParser {
    pub const HEADER_SIZE: usize = 7;

    pub fn parse(buffer: &[u8]) -> Result<()> {
        if buffer.len() < Self::HEADER_SIZE {
            return Err(RarError::BufferTooSmall {
                needed: Self::HEADER_SIZE,
                have: buffer.len(),
            });
        }
        // Just validate we have enough bytes - actual terminator is 0x7B type
        Ok(())
    }
}
