pub(crate) const TIMESTAMP_EXTENSION_TYPE: i8 = -1;

/// Represents timestamp 32 extension type.
/// This stores 32bit unsigned seconds
pub struct Timestamp32 {
    secs: u32,
}

impl Timestamp32 {
    pub fn new(seconds: u32) -> Self {
        Self { secs: seconds }
    }

    pub fn seconds(&self) -> u32 {
        self.secs
    }

    pub(crate) fn to_buf(&self) -> [u8; 4] {
        self.secs.to_be_bytes()
    }

    pub(crate) fn from_buf(buf: [u8; 4]) -> Self {
        Self {
            secs: u32::from_be_bytes(buf),
        }
    }
}

/// Represents timestamp 64 extension type.
/// This stores 34bit unsigned seconds and 30bit nanoseconds
pub struct Timestamp64 {
    data: [u8; 8],
}

impl Timestamp64 {
    pub fn nanos(&self) -> u32 {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&self.data[..4]);
        let nanosec = u32::from_be_bytes(buf);
        nanosec >> 2
    }

    pub fn seconds(&self) -> u64 {
        // 34bit mask
        const MASK: u64 = 0x3FFFF;
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&self.data[..]);
        let seconds = u64::from_be_bytes(buf);

        seconds & MASK
    }

    pub(crate) fn to_buf(&self) -> [u8; 8] {
        self.data
    }

    pub(crate) fn from_buf(buf: [u8; 8]) -> Self {
        Self { data: buf }
    }
}

/// Represents timestamp 96 extension type.
/// This stores 64bit signed seconds and 32bit nanoseconds
pub struct Timestamp96 {
    nanos: u32,
    secs: i64,
}

impl Timestamp96 {
    pub fn new(seconds: i64, nanoseconds: u32) -> Self {
        Self {
            nanos: nanoseconds,
            secs: seconds,
        }
    }

    pub fn nanos(&self) -> u32 {
        self.nanos
    }

    pub fn seconds(&self) -> i64 {
        self.secs
    }

    pub(crate) fn to_buf(&self) -> [u8; 12] {
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
