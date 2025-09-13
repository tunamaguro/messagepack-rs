use super::{Encode, Result};
use crate::{
    extension::FixedExtension,
    io::IoWrite,
    timestamp::{TIMESTAMP_EXTENSION_TYPE, Timestamp32, Timestamp64, Timestamp96},
};

impl<W: IoWrite> Encode<W> for Timestamp32 {
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
        let buf = self.to_buf();
        FixedExtension::new_fixed(TIMESTAMP_EXTENSION_TYPE, buf.len(), buf).encode(writer)
    }
}

impl<W: IoWrite> Encode<W> for Timestamp64 {
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
        let buf = self.to_buf();
        FixedExtension::new_fixed(TIMESTAMP_EXTENSION_TYPE, buf.len(), buf).encode(writer)
    }
}

impl<W: IoWrite> Encode<W> for Timestamp96 {
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
        let buf = self.to_buf();
        FixedExtension::new_fixed(TIMESTAMP_EXTENSION_TYPE, buf.len(), buf).encode(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TIMESTAMP_EXT_TYPE: u8 = 255; // -1

    #[test]
    fn encode_timestamp32() {
        let ts = Timestamp32::new(123456);
        let mut buf = vec![];

        let n = ts.encode(&mut buf).unwrap();

        let mut expected = vec![0xd6, TIMESTAMP_EXT_TYPE];
        expected.extend_from_slice(&123456_u32.to_be_bytes());

        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }

    #[test]
    fn encode_timestamp64() {
        let ts = Timestamp64::new(123456, 789).unwrap();
        let mut buf = vec![];

        let n = ts.encode(&mut buf).unwrap();

        let mut expected = vec![0xd7, TIMESTAMP_EXT_TYPE];
        let data = (789u64 << 34) | 123456;
        expected.extend_from_slice(&data.to_be_bytes());

        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }

    #[test]
    fn encode_timestamp96() {
        let ts = Timestamp96::new(123456, 789);
        let mut buf = vec![];

        let n = ts.encode(&mut buf).unwrap();

        let mut expected = vec![0xc7, 12, TIMESTAMP_EXT_TYPE];
        expected.extend_from_slice(&789_u32.to_be_bytes());
        expected.extend_from_slice(&123456_u64.to_be_bytes());

        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }
}
