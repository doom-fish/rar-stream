//! RarFilesPackage - multi-volume RAR archive parser.

use crate::error::{RarError, Result};
use crate::file_media::{FileMedia, ReadInterval};
use crate::inner_file::InnerFile;
use crate::parsing::{
    ArchiveHeaderParser, FileHeaderParser, MarkerHeaderParser, RarVersion, TerminatorHeaderParser,
    rar5::{Rar5ArchiveHeaderParser, Rar5FileHeaderParser},
};
use crate::rar_file_chunk::RarFileChunk;
use std::collections::HashMap;
use std::sync::Arc;

/// Filter options for parsing.
#[derive(Default)]
pub struct ParseOptions {
    /// Filter function: return true to include a file.
    pub filter: Option<Box<dyn Fn(&str, usize) -> bool + Send + Sync>>,
    /// Maximum number of files to return.
    pub max_files: Option<usize>,
}

/// Parsed file chunk with metadata.
struct ParsedChunk {
    name: String,
    chunk: RarFileChunk,
    continues_in_next: bool,
    unpacked_size: u64,
    chunk_size: u64,
    method: u8,
}

/// Multi-volume RAR archive parser.
pub struct RarFilesPackage {
    files: Vec<Arc<dyn FileMedia>>,
}

impl RarFilesPackage {
    pub fn new(files: Vec<Arc<dyn FileMedia>>) -> Self {
        // Sort files by name to ensure correct order (.rar, .r00, .r01, ...)
        let mut files = files;
        files.sort_by(|a, b| Self::volume_order(a.name()).cmp(&Self::volume_order(b.name())));
        Self { files }
    }

    /// Get sort order for volume names.
    fn volume_order(name: &str) -> (u32, String) {
        let lower = name.to_lowercase();
        if lower.ends_with(".rar") {
            (0, lower) // .rar comes first
        } else {
            // Try to extract number from extension like .r00, .r01
            let ext = lower.rsplit('.').next().unwrap_or("");
            if ext.starts_with('r') && ext.len() == 3 {
                ext[1..]
                    .parse::<u32>()
                    .map(|n| (n + 1, lower.clone()))
                    .unwrap_or((1000, lower))
            } else {
                (1000, lower)
            }
        }
    }

    /// Parse a single RAR file and extract file chunks.
    async fn parse_file(
        &self,
        rar_file: &Arc<dyn FileMedia>,
        opts: &ParseOptions,
    ) -> Result<Vec<ParsedChunk>> {
        let mut offset = 0u64;

        // Read enough for both RAR4 and RAR5 signatures
        let marker_buf = rar_file
            .read_range(ReadInterval {
                start: offset,
                end: offset + 8 - 1, // RAR5 signature is 8 bytes
            })
            .await?;
        
        let marker = MarkerHeaderParser::parse(&marker_buf)?;
        
        // Dispatch based on version
        match marker.version {
            RarVersion::Rar4 => self.parse_rar4_file(rar_file, opts, marker.size as u64).await,
            RarVersion::Rar5 => self.parse_rar5_file(rar_file, opts).await,
        }
    }

    /// Parse a RAR4 format file.
    async fn parse_rar4_file(
        &self,
        rar_file: &Arc<dyn FileMedia>,
        opts: &ParseOptions,
        marker_size: u64,
    ) -> Result<Vec<ParsedChunk>> {
        let mut chunks = Vec::new();
        let mut offset = marker_size;

        // Parse archive header
        let archive_buf = rar_file
            .read_range(ReadInterval {
                start: offset,
                end: offset + ArchiveHeaderParser::HEADER_SIZE as u64 - 1,
            })
            .await?;
        let archive = ArchiveHeaderParser::parse(&archive_buf)?;
        offset += archive.size as u64;

        let mut file_count = 0usize;
        let mut retrieved_count = 0usize;
        let terminator_size = TerminatorHeaderParser::HEADER_SIZE as u64;

        // Parse file headers
        while offset < rar_file.length().saturating_sub(terminator_size) {
            // Read enough bytes for header (but not more than available)
            let bytes_available = rar_file.length().saturating_sub(offset);
            let read_size = (FileHeaderParser::HEADER_SIZE as u64).min(bytes_available);

            if read_size < 32 {
                // Not enough for minimum header
                break;
            }

            let header_buf = rar_file
                .read_range(ReadInterval {
                    start: offset,
                    end: offset + read_size - 1,
                })
                .await?;

            let file_header = match FileHeaderParser::parse(&header_buf) {
                Ok(h) => h,
                Err(_) => break,
            };

            // Check if this is a file header (type 0x74 = 116)
            if file_header.header_type != 0x74 {
                break;
            }

            // Check encryption
            if file_header.is_encrypted {
                return Err(RarError::EncryptedNotSupported);
            }

            let data_start = offset + file_header.head_size as u64;
            let data_end = data_start + file_header.packed_size - 1;

            // Apply filter
            let include = match &opts.filter {
                Some(f) => f(&file_header.name, file_count),
                None => true,
            };

            if include {
                let chunk = RarFileChunk::new(rar_file.clone(), data_start, data_end);
                let chunk_size = chunk.length();

                chunks.push(ParsedChunk {
                    name: file_header.name.clone(),
                    chunk,
                    continues_in_next: file_header.continues_in_next,
                    unpacked_size: file_header.unpacked_size,
                    chunk_size,
                    method: file_header.method,
                });
                retrieved_count += 1;

                // Check max files limit
                if let Some(max) = opts.max_files {
                    if retrieved_count >= max {
                        break;
                    }
                }
            }

            offset = data_end + 1;
            file_count += 1;
        }

        Ok(chunks)
    }

    /// Parse a RAR5 format file.
    async fn parse_rar5_file(
        &self,
        rar_file: &Arc<dyn FileMedia>,
        opts: &ParseOptions,
    ) -> Result<Vec<ParsedChunk>> {
        let mut chunks = Vec::new();
        let mut offset = 8u64; // RAR5 signature is 8 bytes

        // Read archive header (variable size, read enough for typical header)
        let header_buf = rar_file
            .read_range(ReadInterval {
                start: offset,
                end: (offset + 256 - 1).min(rar_file.length() - 1),
            })
            .await?;

        let (archive_header, consumed) = Rar5ArchiveHeaderParser::parse(&header_buf)?;
        offset += consumed as u64;

        let mut file_count = 0usize;
        let mut retrieved_count = 0usize;

        // Parse file headers
        while offset < rar_file.length().saturating_sub(16) {
            // Read header data (variable size)
            let bytes_available = rar_file.length().saturating_sub(offset);
            let read_size = 512u64.min(bytes_available);

            if read_size < 16 {
                break;
            }

            let header_buf = rar_file
                .read_range(ReadInterval {
                    start: offset,
                    end: offset + read_size - 1,
                })
                .await?;

            // Try to parse as file header
            let (file_header, header_consumed) = match Rar5FileHeaderParser::parse(&header_buf) {
                Ok(h) => h,
                Err(_) => break,
            };

            let data_start = offset + header_consumed as u64;
            let data_end = data_start + file_header.packed_size - 1;

            // Apply filter
            let include = match &opts.filter {
                Some(f) => f(&file_header.name, file_count),
                None => true,
            };

            if include {
                let chunk = RarFileChunk::new(rar_file.clone(), data_start, data_end);
                let chunk_size = file_header.packed_size;

                // Convert RAR5 method to RAR4-compatible format
                // RAR5 method 0 = stored, 1-5 = compression
                let method = if file_header.compression.is_stored() {
                    0x30 // Store
                } else {
                    0x30 + file_header.compression.method
                };

                chunks.push(ParsedChunk {
                    name: file_header.name.clone(),
                    chunk,
                    continues_in_next: file_header.continues_in_next(),
                    unpacked_size: file_header.unpacked_size,
                    chunk_size,
                    method,
                });
                retrieved_count += 1;

                if let Some(max) = opts.max_files {
                    if retrieved_count >= max {
                        break;
                    }
                }
            }

            offset = data_end + 1;
            file_count += 1;
        }

        Ok(chunks)
    }

    /// Parse all volumes and return inner files.
    pub async fn parse(&self, opts: ParseOptions) -> Result<Vec<InnerFile>> {
        if self.files.is_empty() {
            return Err(RarError::NoFilesFound);
        }

        let mut all_parsed: Vec<Vec<ParsedChunk>> = Vec::new();

        let mut i = 0;
        while i < self.files.len() {
            let file = &self.files[i];
            let chunks = self.parse_file(file, &opts).await?;

            if chunks.is_empty() {
                i += 1;
                continue;
            }

            // Get info from last chunk
            let last = chunks.last().unwrap();
            let continues = last.continues_in_next;
            let chunk_size = last.chunk_size;
            let unpacked_size = last.unpacked_size;
            let chunk_start = last.chunk.start_offset;
            let chunk_end = last.chunk.end_offset;
            let name = last.name.clone();

            all_parsed.push(chunks);

            // Handle continuation - simplified approach matching original rar-stream
            if continues {
                let mut remaining = unpacked_size.saturating_sub(chunk_size);
                while remaining >= chunk_size && i + 1 < self.files.len() {
                    i += 1;
                    let next_file = &self.files[i];

                    // Create chunk at same offsets in next volume
                    let chunk = RarFileChunk::new(next_file.clone(), chunk_start, chunk_end);
                    all_parsed.push(vec![ParsedChunk {
                        name: name.clone(),
                        chunk,
                        continues_in_next: false,
                        unpacked_size,
                        chunk_size,
                        method: 0x30, // Continue chunks are always raw data
                    }]);
                    remaining = remaining.saturating_sub(chunk_size);
                }
            }

            i += 1;
        }

        // Flatten and group chunks by filename, keeping method info
        let all_chunks: Vec<ParsedChunk> = all_parsed.into_iter().flatten().collect();

        let mut grouped: HashMap<String, (Vec<RarFileChunk>, u8, u64)> = HashMap::new();
        for chunk in all_chunks {
            let entry = grouped
                .entry(chunk.name)
                .or_insert_with(|| (Vec::new(), chunk.method, chunk.unpacked_size));
            entry.0.push(chunk.chunk);
        }

        // Create InnerFile for each group
        let inner_files: Vec<InnerFile> = grouped
            .into_iter()
            .map(|(name, (chunks, method, unpacked_size))| {
                InnerFile::new(name, chunks, method, unpacked_size)
            })
            .collect();

        Ok(inner_files)
    }
}
