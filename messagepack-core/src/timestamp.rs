//! MessagePack timestamp extension values.

use crate::extension::{ExtensionRef, FixedExtension};

pub(crate) const TIMESTAMP_EXTENSION_TYPE: i8 = -1;

/// The error type returned when a checked extension conversion fails
pub enum TryFromTimeStampError {
    /// The format is not a valid timestamp type
    InvalidType,
    /// The data length is not valid for timestamp format
    InvalidDataLength,
}

/// Represents timestamp 32 extension type.
/// This stores 32bit unsigned seconds
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp32 {
    secs: u32,
}

impl Timestamp32 {
    /// Create a 32‑bit seconds timestamp.
    pub fn new(seconds: u32) -> Self {
        Self { secs: seconds }
    }

    /// Get seconds since the UNIX epoch.
    pub fn seconds(&self) -> u32 {
        self.secs
    }

    pub(crate) fn to_buf(self) -> [u8; 4] {
        self.secs.to_be_bytes()
    }

    pub(crate) fn from_buf(buf: [u8; 4]) -> Self {
        Self {
            secs: u32::from_be_bytes(buf),
        }
    }
}

impl TryFrom<ExtensionRef<'_>> for Timestamp32 {
    type Error = TryFromTimeStampError;

    fn try_from(value: ExtensionRef<'_>) -> Result<Self, Self::Error> {
        if value.r#type != TIMESTAMP_EXTENSION_TYPE {
            return Err(TryFromTimeStampError::InvalidType);
        }

        let data = value.data;
        let mut buf = [0u8; 4];
        if data.len() != buf.len() {
            return Err(TryFromTimeStampError::InvalidDataLength);
        }

        buf.copy_from_slice(data);
        Ok(Self::from_buf(buf))
    }
}

impl TryFrom<FixedExtension<4>> for Timestamp32 {
    type Error = TryFromTimeStampError;

    fn try_from(value: FixedExtension<4>) -> Result<Self, Self::Error> {
        value.as_ref().try_into()
    }
}

impl From<Timestamp32> for FixedExtension<4> {
    fn from(value: Timestamp32) -> Self {
        let buf = value.to_buf();
        FixedExtension::new_fixed(TIMESTAMP_EXTENSION_TYPE, buf.len(), buf)
    }
}

impl From<Timestamp32> for core::time::Duration {
    fn from(value: Timestamp32) -> Self {
        core::time::Duration::from_secs(value.seconds().into())
    }
}

impl TryFrom<core::time::Duration> for Timestamp32 {
    type Error = core::num::TryFromIntError;
    fn try_from(value: core::time::Duration) -> Result<Self, Self::Error> {
        let sec = value.as_secs();
        u32::try_from(sec).map(|v| Self::new(v))
    }
}

/// Represents timestamp 64 extension type.
/// This stores 34bit unsigned seconds and 30bit nanoseconds
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp64 {
    data: [u8; 8],
}

/// `seconds` or `nanos` cannot be represented
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp64Error {
    /// Requested seconds that exceeded the 34‑bit range.
    pub seconds: u64,
    /// Requested nanoseconds that exceeded the 30‑bit range.
    pub nanos: u32,
}

impl Timestamp64 {
    /// Create a 64‑bit timestamp storing 34‑bit seconds and 30‑bit nanoseconds.
    pub fn new(seconds: u64, nanos: u32) -> Result<Self, Timestamp64Error> {
        const SECONDS_MAX_LIMIT: u64 = 1 << 34;

        if seconds >= SECONDS_MAX_LIMIT {
            return Err(Timestamp64Error { seconds, nanos });
        }

        const NANOS_MAX_LIMIT: u32 = 1 << 30;
        if nanos >= NANOS_MAX_LIMIT {
            return Err(Timestamp64Error { seconds, nanos });
        }

        let mut buf = [0u8; 8];
        buf[..].copy_from_slice(&seconds.to_be_bytes());

        let nano = (nanos << 2).to_be_bytes();
        buf[..3].copy_from_slice(&nano[..3]);
        buf[3] |= nano[3];

        Ok(Self::from_buf(buf))
    }

    /// Get the nanoseconds component.
    pub fn nanos(&self) -> u32 {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&self.data[..4]);
        let nanosec = u32::from_be_bytes(buf);
        nanosec >> 2
    }

    /// Get the seconds component.
    pub fn seconds(&self) -> u64 {
        // 34bit mask
        const MASK: u64 = (1 << 34) - 1;
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&self.data[..]);
        let seconds = u64::from_be_bytes(buf);

        seconds & MASK
    }

    pub(crate) fn to_buf(self) -> [u8; 8] {
        self.data
    }

    pub(crate) fn from_buf(buf: [u8; 8]) -> Self {
        Self { data: buf }
    }
}

impl TryFrom<ExtensionRef<'_>> for Timestamp64 {
    type Error = TryFromTimeStampError;

    fn try_from(value: ExtensionRef<'_>) -> Result<Self, Self::Error> {
        if value.r#type != TIMESTAMP_EXTENSION_TYPE {
            return Err(TryFromTimeStampError::InvalidType);
        }

        let data = value.data;
        let mut buf = [0u8; 8];
        if data.len() != buf.len() {
            return Err(TryFromTimeStampError::InvalidDataLength);
        }

        buf.copy_from_slice(data);
        Ok(Self::from_buf(buf))
    }
}

impl TryFrom<FixedExtension<8>> for Timestamp64 {
    type Error = TryFromTimeStampError;

    fn try_from(value: FixedExtension<8>) -> Result<Self, Self::Error> {
        value.as_ref().try_into()
    }
}

impl From<Timestamp64> for FixedExtension<8> {
    fn from(value: Timestamp64) -> Self {
        let buf = value.to_buf();
        FixedExtension::new_fixed(TIMESTAMP_EXTENSION_TYPE, buf.len(), buf)
    }
}

impl From<Timestamp64> for core::time::Duration {
    fn from(value: Timestamp64) -> Self {
        let sec = value.seconds();
        let nano = value.nanos();
        core::time::Duration::from_secs(sec) + core::time::Duration::from_nanos(nano.into())
    }
}

impl TryFrom<core::time::Duration> for Timestamp64 {
    type Error = Timestamp64Error;

    fn try_from(value: core::time::Duration) -> Result<Self, Self::Error> {
        let secs = value.as_secs();
        let nanos = value.subsec_nanos();
        Timestamp64::new(secs, nanos)
    }
}

/// Represents timestamp 96 extension type.
/// This stores 64bit signed seconds and 32bit nanoseconds
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp96 {
    nanos: u32,
    secs: i64,
}

impl Timestamp96 {
    /// Create a 96‑bit timestamp storing signed seconds and nanoseconds.
    pub fn new(seconds: i64, nanoseconds: u32) -> Self {
        Self {
            nanos: nanoseconds,
            secs: seconds,
        }
    }

    /// Get the nanoseconds component.
    pub fn nanos(&self) -> u32 {
        self.nanos
    }

    /// Get the seconds component.
    pub fn seconds(&self) -> i64 {
        self.secs
    }

    pub(crate) fn to_buf(self) -> [u8; 12] {
        let mut buf = [0u8; 12];
        buf[..4].copy_from_slice(&self.nanos.to_be_bytes());
        buf[4..].copy_from_slice(&self.secs.to_be_bytes());

        buf
    }

    pub(crate) fn from_buf(buf: [u8; 12]) -> Self {
        let mut nano = [0u8; 4];
        nano.copy_from_slice(&buf[..4]);

        let mut second = [0u8; 8];
        second.copy_from_slice(&buf[4..]);

        Self {
            nanos: u32::from_be_bytes(nano),
            secs: i64::from_be_bytes(second),
        }
    }
}

impl TryFrom<ExtensionRef<'_>> for Timestamp96 {
    type Error = TryFromTimeStampError;

    fn try_from(value: ExtensionRef<'_>) -> Result<Self, Self::Error> {
        if value.r#type != TIMESTAMP_EXTENSION_TYPE {
            return Err(TryFromTimeStampError::InvalidType);
        }

        let data = value.data;
        let mut buf = [0u8; 12];
        if data.len() != buf.len() {
            return Err(TryFromTimeStampError::InvalidDataLength);
        }

        buf.copy_from_slice(data);
        Ok(Self::from_buf(buf))
    }
}

impl TryFrom<FixedExtension<12>> for Timestamp96 {
    type Error = TryFromTimeStampError;

    fn try_from(value: FixedExtension<12>) -> Result<Self, Self::Error> {
        value.as_ref().try_into()
    }
}

impl From<Timestamp96> for FixedExtension<12> {
    fn from(value: Timestamp96) -> Self {
        let buf = value.to_buf();
        FixedExtension::new_fixed(TIMESTAMP_EXTENSION_TYPE, buf.len(), buf)
    }
}

impl TryFrom<Timestamp96> for core::time::Duration {
    type Error = core::num::TryFromIntError;

    fn try_from(value: Timestamp96) -> Result<Self, Self::Error> {
        let secs = u64::try_from(value.seconds())?;
        let nanos = value.nanos();

        Ok(core::time::Duration::from_secs(secs) + core::time::Duration::from_nanos(nanos.into()))
    }
}

impl TryFrom<core::time::Duration> for Timestamp96 {
    type Error = core::num::TryFromIntError;

    fn try_from(value: core::time::Duration) -> Result<Self, Self::Error> {
        let secs = i64::try_from(value.as_secs())?;
        let nanos = value.subsec_nanos();
        Ok(Timestamp96::new(secs, nanos))
    }
}

#[cfg(test)]
mod duration_tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn duration_to_timestamp32_roundtrip_within_range() {
        let d = core::time::Duration::from_secs(123);
        let ts32 = Timestamp32::try_from(d).unwrap();
        assert_eq!(ts32.seconds(), 123);
        let back: core::time::Duration = ts32.into();
        assert_eq!(back.as_secs(), 123);
        assert_eq!(back.subsec_nanos(), 0);
    }

    #[rstest]
    fn duration_to_timestamp64_roundtrip() {
        let d = core::time::Duration::from_secs(1_234_567) + core::time::Duration::from_nanos(890);
        let ts64 = Timestamp64::try_from(d).unwrap();
        assert_eq!(ts64.seconds(), 1_234_567);
        assert_eq!(ts64.nanos(), 890);
        let back: core::time::Duration = ts64.into();
        assert_eq!(back, d);
    }

    #[rstest]
    fn timestamp96_to_duration_fails_on_negative() {
        let ts96 = Timestamp96::new(-1, 0);
        let res: Result<core::time::Duration, core::num::TryFromIntError> =
            core::time::Duration::try_from(ts96);
        assert!(res.is_err());
    }

    #[rstest]
    fn duration_to_timestamp96_roundtrip() {
        let d = core::time::Duration::from_secs(12_345) + core::time::Duration::from_nanos(678_901);
        let ts = Timestamp96::try_from(d).unwrap();
        assert_eq!(ts.seconds(), 12_345);
        assert_eq!(ts.nanos(), 678_901);
        let back = core::time::Duration::try_from(ts).unwrap();
        assert_eq!(back, d);
    }
}
