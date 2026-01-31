//! RAR5 format parsing modules.
//!
//! RAR5 uses a completely different header format than RAR4:
//! - Variable-length integers (vint) for sizes
//! - CRC-32 instead of CRC-16
//! - Different header type codes
//! - Different compression algorithm

mod vint;

pub mod archive_header;
pub mod encryption_header;
pub mod end_header;
pub mod file_header;

pub use archive_header::{Rar5ArchiveHeader, Rar5ArchiveHeaderParser};
pub use encryption_header::{Rar5EncryptionHeader, Rar5EncryptionHeaderParser};
pub use end_header::{Rar5EndHeader, Rar5EndHeaderParser};
pub use file_header::{Rar5FileHeader, Rar5FileHeaderParser};
pub use vint::{read_vint, VintReader};

/// RAR5 header type codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Rar5HeaderType {
    /// Main archive header
    Main = 1,
    /// File header
    File = 2,
    /// Service header (e.g., NTFS streams, ACL)
    Service = 3,
    /// Encryption header
    Encryption = 4,
    /// End of archive header
    End = 5,
}

impl TryFrom<u64> for Rar5HeaderType {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Main),
            2 => Ok(Self::File),
            3 => Ok(Self::Service),
            4 => Ok(Self::Encryption),
            5 => Ok(Self::End),
            _ => Err(()),
        }
    }
}

/// RAR5 common header flags.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rar5HeaderFlags {
    /// Extra area is present after header
    pub has_extra_area: bool,
    /// Data area is present after header
    pub has_data_area: bool,
    /// Skip header if unknown type
    pub skip_if_unknown: bool,
    /// Data continues from previous volume
    pub split_before: bool,
    /// Data continues in next volume
    pub split_after: bool,
}

impl From<u64> for Rar5HeaderFlags {
    fn from(flags: u64) -> Self {
        Self {
            has_extra_area: flags & 0x0001 != 0,
            has_data_area: flags & 0x0002 != 0,
            skip_if_unknown: flags & 0x0004 != 0,
            split_before: flags & 0x0008 != 0,
            split_after: flags & 0x0010 != 0,
        }
    }
}
