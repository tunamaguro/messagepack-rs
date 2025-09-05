pub(crate) const TIMESTAMP_EXTENSION_TYPE: i8 = -1;

/// Represents timestamp 32 extension type.
/// This stores 32bit unsigned seconds
#[derive(Clone, Copy, Debug)]
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

    pub(crate) fn to_buf(self) -> [u8; 4] {
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
#[derive(Clone, Copy, Debug)]
pub struct Timestamp64 {
    data: [u8; 8],
}

/// `seconds` or `nanos` cannot be represented
#[derive(Clone, Debug)]
pub struct Timestamp64Error {
    pub seconds: u64,
    pub nanos: u32,
}

impl Timestamp64 {
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

    pub fn nanos(&self) -> u32 {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&self.data[..4]);
        let nanosec = u32::from_be_bytes(buf);
        nanosec >> 2
    }

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

/// Represents timestamp 96 extension type.
/// This stores 64bit signed seconds and 32bit nanoseconds
#[derive(Clone, Copy, Debug)]
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
