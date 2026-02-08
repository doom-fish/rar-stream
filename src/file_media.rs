//! FileMedia trait - abstract byte source for RAR reading.

use crate::error::Result;
use std::io::{Read, Seek, SeekFrom};

/// Interval for reading a byte range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReadInterval {
    pub start: u64,
    pub end: u64,
}

/// Local file implementation.
#[derive(Debug, Clone)]
pub struct LocalFileMedia {
    path: String,
    name: String,
    length: u64,
}

impl LocalFileMedia {
    pub fn new(path: &str) -> std::io::Result<Self> {
        let metadata = std::fs::metadata(path)?;
        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Self {
            path: path.to_string(),
            name,
            length: metadata.len(),
        })
    }

    pub fn length(&self) -> u64 {
        self.length
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sync read
    pub fn read_range_sync(&self, interval: ReadInterval) -> Result<Vec<u8>> {
        let mut file = std::fs::File::open(&self.path)?;
        file.seek(SeekFrom::Start(interval.start))?;
        let len = (interval.end - interval.start + 1) as usize;
        let mut buffer = vec![0u8; len];
        file.read_exact(&mut buffer)?;
        Ok(buffer)
    }
}

// Async FileMedia trait (requires 'async' feature)
#[cfg(feature = "async")]
use std::future::Future;
#[cfg(feature = "async")]
use std::pin::Pin;

/// Abstract file source that can provide byte ranges asynchronously.
///
/// Implement this trait for custom byte sources (e.g., HTTP range requests).
/// The library provides [`LocalFileMedia`] for local files.
#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub trait FileMedia: Send + Sync {
    fn length(&self) -> u64;
    fn name(&self) -> &str;
    fn read_range(
        &self,
        interval: ReadInterval,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + '_>>;
}

#[cfg(feature = "async")]
impl FileMedia for LocalFileMedia {
    fn length(&self) -> u64 {
        self.length
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn read_range(
        &self,
        interval: ReadInterval,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + '_>> {
        let path = self.path.clone();
        Box::pin(async move {
            use tokio::io::{AsyncReadExt, AsyncSeekExt};
            let mut file = tokio::fs::File::open(&path).await?;
            file.seek(std::io::SeekFrom::Start(interval.start)).await?;
            let len = (interval.end - interval.start + 1) as usize;
            let mut buffer = vec![0u8; len];
            file.read_exact(&mut buffer).await?;
            Ok(buffer)
        })
    }
}
