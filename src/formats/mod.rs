//! RAR format detection and signatures.
//!
//! Zero dependencies.

/// RAR file signature detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signature {
    /// RAR 1.5 to 4.x
    Rar15,
    /// RAR 5.0+
    Rar50,
}

impl Signature {
    pub const RAR15: &[u8; 7] = b"Rar!\x1a\x07\x00";
    pub const RAR50: &[u8; 8] = b"Rar!\x1a\x07\x01\x00";

    pub fn size(&self) -> u64 {
        match self {
            Self::Rar15 => 7,
            Self::Rar50 => 8,
        }
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() >= 8 && data.starts_with(Self::RAR50) {
            Some(Self::Rar50)
        } else if data.len() >= 7 && data.starts_with(Self::RAR15) {
            Some(Self::Rar15)
        } else {
            None
        }
    }
}

/// Raw timestamp value (Unix nanoseconds).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RawTimestamp {
    pub nanos: i64,
}

impl RawTimestamp {
    pub fn from_unix_nanos(nanos: i64) -> Self {
        Self { nanos }
    }

    pub fn from_dos(dos_time: u32) -> Self {
        let second = ((dos_time & 0x1f) * 2) as i64;
        let minute = ((dos_time >> 5) & 0x3f) as i64;
        let hour = ((dos_time >> 11) & 0x1f) as i64;
        let day = ((dos_time >> 16) & 0x1f) as i64;
        let month = ((dos_time >> 21) & 0x0f) as i64;
        let year = ((dos_time >> 25) + 1980) as i64;

        // Count days from epoch (1970-01-01) to the given date
        let mut days: i64 = 0;
        for y in 1970..year {
            days += if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
                366
            } else {
                365
            };
        }
        let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let month_days = [
            31,
            if is_leap { 29 } else { 28 },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ];
        for m in 0..(month - 1).min(11) as usize {
            days += month_days[m] as i64;
        }
        days += day - 1;

        let secs = days * 86400 + hour * 3600 + minute * 60 + second;
        Self {
            nanos: secs * 1_000_000_000,
        }
    }

    pub fn saturating_add(self, add_nanos: i64) -> Self {
        Self {
            nanos: self.nanos.saturating_add(add_nanos),
        }
    }
}

pub fn parse_dos_datetime(dos_time: u32) -> RawTimestamp {
    RawTimestamp::from_dos(dos_time)
}

pub fn parse_windows_filetime(filetime: u64) -> RawTimestamp {
    const WINDOWS_TICK_NS: i128 = 100;
    const EPOCH_DIFF: i128 = 11_644_473_600_000_000_000;
    let unix_ns = (filetime as i128) * WINDOWS_TICK_NS - EPOCH_DIFF;
    RawTimestamp::from_unix_nanos(unix_ns as i64)
}
