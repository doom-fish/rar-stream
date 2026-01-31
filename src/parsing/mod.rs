//! RAR header parsing modules.

pub mod archive_header;
pub mod file_header;
pub mod marker_header;
pub mod terminator_header;

pub use archive_header::ArchiveHeaderParser;
pub use file_header::FileHeaderParser;
pub use marker_header::MarkerHeaderParser;
pub use terminator_header::TerminatorHeaderParser;
