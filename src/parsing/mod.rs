//! RAR header parsing modules.

pub mod archive_header;
pub mod file_header;
pub mod marker_header;
pub mod rar5;
pub mod terminator_header;

pub use archive_header::ArchiveHeaderParser;
pub use file_header::FileHeaderParser;
pub use marker_header::{MarkerHeaderParser, RarVersion};
pub use terminator_header::TerminatorHeaderParser;
pub use rar5::{Rar5ArchiveHeader, Rar5FileHeader, Rar5EndHeader};
