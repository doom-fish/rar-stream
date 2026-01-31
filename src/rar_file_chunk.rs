//! RarFileChunk - a byte range within a single RAR volume.

use crate::error::Result;
use crate::file_media::{FileMedia, ReadInterval};
use std::fmt;
use std::sync::Arc;

/// A byte range within a single FileMedia (RAR volume).
#[derive(Clone)]
pub struct RarFileChunk {
    file_media: Arc<dyn FileMedia>,
    pub start_offset: u64,
    pub end_offset: u64,
}

impl fmt::Debug for RarFileChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RarFileChunk")
            .field("file_name", &self.file_media.name())
            .field("start_offset", &self.start_offset)
            .field("end_offset", &self.end_offset)
            .finish()
    }
}

impl RarFileChunk {
    pub fn new(file_media: Arc<dyn FileMedia>, start_offset: u64, end_offset: u64) -> Self {
        Self {
            file_media,
            start_offset,
            end_offset,
        }
    }

    /// Length of this chunk in bytes.
    pub fn length(&self) -> u64 {
        if self.end_offset >= self.start_offset {
            self.end_offset - self.start_offset + 1
        } else {
            0
        }
    }

    /// Create a new chunk with the start offset moved forward.
    pub fn pad_start(&self, padding: u64) -> Self {
        Self {
            file_media: self.file_media.clone(),
            start_offset: self.start_offset + padding,
            end_offset: self.end_offset,
        }
    }

    /// Create a new chunk with the end offset moved backward.
    pub fn pad_end(&self, padding: u64) -> Self {
        Self {
            file_media: self.file_media.clone(),
            start_offset: self.start_offset,
            end_offset: self.end_offset.saturating_sub(padding),
        }
    }

    /// Read the entire chunk.
    pub async fn read(&self) -> Result<Vec<u8>> {
        self.file_media
            .read_range(ReadInterval {
                start: self.start_offset,
                end: self.end_offset,
            })
            .await
    }

    /// Read a portion of the chunk.
    pub async fn read_range(&self, start: u64, end: u64) -> Result<Vec<u8>> {
        self.file_media
            .read_range(ReadInterval {
                start: self.start_offset + start,
                end: self.start_offset + end,
            })
            .await
    }

    /// Get the name of the file media (volume) this chunk belongs to.
    pub fn volume_name(&self) -> &str {
        self.file_media.name()
    }
}
