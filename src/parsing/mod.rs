//! RAR header parsing.
//!
//! This module provides parsers for RAR archive headers. Both RAR4 and RAR5
//! formats are supported, with automatic format detection.
//!
//! ## RAR Archive Structure
//!
//! A RAR archive consists of a sequence of headers:
//!
//! ```text
//! ┌─────────────────┐
//! │ Marker Header   │ ← RAR signature (7-8 bytes)
//! ├─────────────────┤
//! │ Archive Header  │ ← Archive flags (solid, multi-volume, etc.)
//! ├─────────────────┤
//! │ File Header 1   │ ← File metadata (name, size, method)
//! │ [File Data 1]   │ ← Compressed/encrypted content
//! ├─────────────────┤
//! │ File Header 2   │
//! │ [File Data 2]   │
//! ├─────────────────┤
//! │ ...             │
//! ├─────────────────┤
//! │ End Header      │ ← Archive terminator
//! └─────────────────┘
//! ```
//!
//! ## Format Differences
//!
//! | Feature | RAR4 | RAR5 |
//! |---------|------|------|
//! | Signature | 7 bytes | 8 bytes |
//! | Header size | Fixed fields | Variable-length integers |
//! | Max file size | 8 EB (64-bit) | Unlimited (vint) |
//! | Encryption header | No | Yes (type 4) |
//! | Extra area | No | Yes (extensible) |
//!
//! ## Header Types
//!
//! ### RAR4 Header Types
//!
//! | Type | Value | Description |
//! |------|-------|-------------|
//! | Marker | `0x72` | Archive signature |
//! | Archive | `0x73` | Archive-level flags |
//! | File | `0x74` | File entry |
//! | Comment | `0x75` | Archive comment |
//! | Extra | `0x76` | Extra info |
//! | Subblock | `0x77` | Subblock |
//! | Recovery | `0x78` | Recovery record |
//! | End | `0x7B` | End of archive |
//!
//! ### RAR5 Header Types
//!
//! | Type | Value | Description |
//! |------|-------|-------------|
//! | Archive | 1 | Main archive header |
//! | File | 2 | File entry |
//! | Service | 3 | Service data (comments, etc.) |
//! | Encryption | 4 | Archive encryption header |
//! | End | 5 | End of archive |
//!
//! ## Example
//!
//! ```rust,ignore
//! use rar_stream::parsing::{MarkerHeaderParser, RarVersion};
//!
//! let data = std::fs::read("archive.rar")?;
//! let (version, consumed) = MarkerHeaderParser::parse(&data)?;
//!
//! match version {
//!     RarVersion::Rar4 => println!("RAR 4.x format"),
//!     RarVersion::Rar5 => println!("RAR 5.x format"),
//! }
//! ```

pub mod archive_header;
pub mod file_header;
pub mod marker_header;
pub mod rar5;
pub mod terminator_header;

pub use archive_header::ArchiveHeaderParser;
pub use file_header::FileHeaderParser;
pub use marker_header::{MarkerHeaderParser, RarVersion};
pub use rar5::{Rar5ArchiveHeader, Rar5EndHeader, Rar5FileHeader};
pub use terminator_header::TerminatorHeaderParser;
