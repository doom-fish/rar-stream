//! NAPI bindings for rar-stream compatibility.
//!
//! Exposes Rust types to Node.js with the same API as the original rar-stream.

#![allow(missing_docs)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::file_media::{FileMedia, LocalFileMedia as RustLocalFileMedia, ReadInterval};
use crate::inner_file::InnerFile as RustInnerFile;
use crate::rar_files_package::{ParseOptions, RarFilesPackage as RustRarFilesPackage};

/// LocalFileMedia - reads from local filesystem.
#[napi]
pub struct LocalFileMedia {
    inner: Arc<RustLocalFileMedia>,
}

#[napi]
impl LocalFileMedia {
    #[napi(constructor)]
    pub fn new(path: String) -> Result<Self> {
        let inner = RustLocalFileMedia::new(&path)
            .map_err(|e| Error::from_reason(format!("Failed to open file: {}", e)))?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    #[napi(getter)]
    pub fn name(&self) -> String {
        self.inner.name().to_string()
    }

    #[napi(getter)]
    pub fn length(&self) -> i64 {
        self.inner.length() as i64
    }

    #[napi]
    pub async fn create_read_stream(&self, opts: ReadIntervalJs) -> Result<Buffer> {
        let interval = ReadInterval {
            start: opts.start as u64,
            end: opts.end as u64,
        };
        let data = self
            .inner
            .read_range(interval)
            .await
            .map_err(|e| Error::from_reason(format!("{}", e)))?;
        Ok(Buffer::from(data))
    }
}

/// Read interval options.
#[napi(object)]
pub struct ReadIntervalJs {
    pub start: i64,
    pub end: i64,
}

/// InnerFile - a file inside the RAR archive.
/// Note: We use the name "NapiInnerFile" in Rust to avoid conflict with RustInnerFile,
/// but it's exported to JS as "InnerFile".
#[napi]
pub struct InnerFile {
    inner: Arc<Mutex<RustInnerFile>>,
}

#[napi]
impl InnerFile {
    #[napi(getter)]
    pub fn name(&self) -> String {
        // Need to block to get the name - it's just a string copy
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let inner = self.inner.lock().await;
            inner.name.clone()
        })
    }

    #[napi(getter)]
    pub fn length(&self) -> i64 {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let inner = self.inner.lock().await;
            inner.length as i64
        })
    }

    #[napi]
    pub async fn create_read_stream(&self, opts: ReadIntervalJs) -> Result<Buffer> {
        let inner = self.inner.lock().await;
        let interval = ReadInterval {
            start: opts.start as u64,
            end: opts.end as u64,
        };
        let data = inner
            .read_range(interval)
            .await
            .map_err(|e| Error::from_reason(format!("{}", e)))?;
        Ok(Buffer::from(data))
    }

    #[napi]
    pub async fn read_to_end(&self) -> Result<Buffer> {
        let inner = self.inner.lock().await;
        let data = inner
            .read_to_end()
            .await
            .map_err(|e| Error::from_reason(format!("{}", e)))?;
        Ok(Buffer::from(data))
    }
}

/// FileMedia wrapper for NAPI - wraps any FileMedia implementation.
/// Currently unused but kept for potential future JS FileMedia implementations.
#[allow(dead_code)]
struct JsFileMediaWrapper {
    name: String,
    length: u64,
    // Store a reference to the JS LocalFileMedia
    local: Option<Arc<RustLocalFileMedia>>,
}

impl FileMedia for JsFileMediaWrapper {
    fn length(&self) -> u64 {
        self.length
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn read_range(
        &self,
        interval: ReadInterval,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::error::Result<Vec<u8>>> + Send + '_>>
    {
        let local = self.local.clone();
        Box::pin(async move {
            if let Some(local) = local {
                local.read_range(interval).await
            } else {
                Err(crate::error::RarError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "No file media available",
                )))
            }
        })
    }
}

/// RarFilesPackage - parses multi-volume RAR archives.
#[napi]
pub struct RarFilesPackage {
    files: Vec<Arc<dyn FileMedia>>,
}

#[napi]
impl RarFilesPackage {
    #[napi(constructor)]
    pub fn new(files: Vec<&LocalFileMedia>) -> Self {
        let files: Vec<Arc<dyn FileMedia>> = files
            .into_iter()
            .map(|f| f.inner.clone() as Arc<dyn FileMedia>)
            .collect();
        Self { files }
    }

    #[napi]
    pub async fn parse(&self, opts: Option<ParseOptionsJs>) -> Result<Vec<InnerFile>> {
        let parse_opts = match opts {
            Some(js_opts) => {
                let filter: Option<Box<dyn Fn(&str, usize) -> bool + Send + Sync>> =
                    js_opts.max_files.map(|_| {
                        // Simple filter that accepts all - real filtering done via max_files
                        Box::new(|_name: &str, _idx: usize| true)
                            as Box<dyn Fn(&str, usize) -> bool + Send + Sync>
                    });
                ParseOptions {
                    filter,
                    max_files: js_opts.max_files.map(|n| n as usize),
                }
            }
            None => ParseOptions::default(),
        };

        let package = RustRarFilesPackage::new(self.files.clone());
        let inner_files = package
            .parse(parse_opts)
            .await
            .map_err(|e| Error::from_reason(format!("{}", e)))?;

        Ok(inner_files
            .into_iter()
            .map(|f| InnerFile {
                inner: Arc::new(Mutex::new(f)),
            })
            .collect())
    }
}

/// Parse options for filtering results.
#[napi(object)]
pub struct ParseOptionsJs {
    pub max_files: Option<i32>,
}

/// Parsed file info from RAR header.
#[napi(object)]
pub struct RarFileInfo {
    pub name: String,
    pub packed_size: i64,
    pub unpacked_size: i64,
    pub method: u8,
    pub continues_in_next: bool,
}

/// Parse RAR file header from a buffer.
/// This can be used to detect RAR archives and get inner file info
/// without downloading the entire archive.
///
/// The buffer should contain at least the first ~300 bytes of a .rar file.
#[napi]
pub fn parse_rar_header(buffer: Buffer) -> Result<Option<RarFileInfo>> {
    use crate::parsing::{ArchiveHeaderParser, FileHeaderParser, MarkerHeaderParser};

    let data: &[u8] = &buffer;

    // Need at least marker + archive + some file header bytes
    if data.len() < 50 {
        return Ok(None);
    }

    // Parse marker header
    let marker = match MarkerHeaderParser::parse(data) {
        Ok(m) => m,
        Err(_) => return Ok(None),
    };
    let mut offset = marker.size as usize;

    // Parse archive header
    if data.len() < offset + ArchiveHeaderParser::HEADER_SIZE {
        return Ok(None);
    }
    let archive = match ArchiveHeaderParser::parse(&data[offset..]) {
        Ok(a) => a,
        Err(_) => return Ok(None),
    };
    offset += archive.size as usize;

    // Parse first file header
    if data.len() < offset + 32 {
        return Ok(None);
    }
    let file_header = match FileHeaderParser::parse(&data[offset..]) {
        Ok(h) => h,
        Err(_) => return Ok(None),
    };

    Ok(Some(RarFileInfo {
        name: file_header.name,
        packed_size: file_header.packed_size as i64,
        unpacked_size: file_header.unpacked_size as i64,
        method: file_header.method,
        continues_in_next: file_header.continues_in_next,
    }))
}

/// Check if a buffer starts with a RAR signature.
#[napi]
pub fn is_rar_archive(buffer: Buffer) -> bool {
    use crate::parsing::marker_header::RAR4_SIGNATURE;
    let data: &[u8] = &buffer;
    data.len() >= 7 && data[..7] == RAR4_SIGNATURE
}
