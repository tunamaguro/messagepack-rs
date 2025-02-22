use super::{Decode, Error, Result};
use crate::formats::Format;

impl Decode for u8 {
    type Value = Self;
    fn decode<I, B>(buf: &mut I) -> Result<Self::Value>
    where
        I: Iterator<Item = B>,
        B: core::borrow::Borrow<u8>,
    {
        let format = Format::decode(buf)?;
        match format {
            Format::PositiveFixInt(v) => Ok(v),
            Format::Uint8 => {
                let mut bytes = [0_u8; core::mem::size_of::<u8>()];
                let mut bytes_mut = bytes.iter_mut();
                for (to, byte) in bytes_mut.by_ref().zip(buf) {
                    *to = *byte.borrow();
                }
                Ok(u8::from_be_bytes(bytes))
            }
            _ => Err(Error::UnexpectedFormat),
        }
    }
}

macro_rules! impl_decode_unsigned {
    ($ty:ty,$format:path) => {
        impl Decode for $ty {
            type Value = Self;
            fn decode<I, B>(buf: &mut I) -> Result<Self::Value>
            where
                I: Iterator<Item = B>,
                B: core::borrow::Borrow<u8>,
            {
                let format = Format::decode(buf)?;
                match format {
                    $format => {
                        let mut bytes = [0_u8; core::mem::size_of::<$ty>()];
                        let mut bytes_mut = bytes.iter_mut();
                        for (to, byte) in bytes_mut.by_ref().zip(buf) {
                            *to = *byte.borrow();
                        }
                        Ok(<$ty>::from_be_bytes(bytes))
                    }
                    _ => Err(Error::UnexpectedFormat),
                }
            }
        }
    };
}
impl_decode_unsigned!(u16, Format::Uint16);
impl_decode_unsigned!(u32, Format::Uint32);
impl_decode_unsigned!(u64, Format::Uint64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_u8_fix_pos_int() {
        // FixPosInt
        let buf: &[u8] = &[0x00];
        let decoded = u8::decode(&mut buf.iter()).unwrap();
        let expect = u8::MIN;
        assert_eq!(decoded, expect);

        let buf: &[u8] = &[0x7f];
        let decoded = u8::decode(&mut buf.iter()).unwrap();
        let expect = 0x7f;
        assert_eq!(decoded, expect);
    }

    #[test]
    fn decode_uint8() {
        // Uint8
        let buf: &[u8] = &[0xcc, 0x00];
        let decoded = u8::decode(&mut buf.iter()).unwrap();
        let expect = u8::MIN;
        assert_eq!(decoded, expect);

        let buf: &[u8] = &[0xcc, 0xff];
        let decoded = u8::decode(&mut buf.iter()).unwrap();
        let expect = u8::MAX;
        assert_eq!(decoded, expect);
    }

    #[test]
    fn decode_uint16() {
        // Uint16
        let buf: &[u8] = &[0xcd, 0x00, 0x00];
        let decoded = u16::decode(&mut buf.iter()).unwrap();
        let expect = u16::MIN;
        assert_eq!(decoded, expect);

        let buf: &[u8] = &[0xcd, 0x12, 0x34];
        let decoded = u16::decode(&mut buf.iter()).unwrap();
        let expect = 0x1234;
        assert_eq!(decoded, expect);

        let buf: &[u8] = &[0xcd, 0xff, 0xff];
        let decoded = u16::decode(&mut buf.iter()).unwrap();
        let expect = u16::MAX;
        assert_eq!(decoded, expect);

        
    }

    #[test]
    fn decode_uint32() {
        // Uint32
        let buf: &[u8] = &[0xce, 0x00, 0x00, 0x00, 0x00];
        let decoded = u32::decode(&mut buf.iter()).unwrap();
        let expect = u32::MIN;
        assert_eq!(decoded, expect);

        let buf: &[u8] = &[0xce, 0x12, 0x34, 0x56, 0x78];
        let decoded = u32::decode(&mut buf.iter()).unwrap();
        let expect = 0x12345678;
        assert_eq!(decoded, expect);

        let buf: &[u8] = &[0xce, 0xff, 0xff, 0xff, 0xff];
        let decoded = u32::decode(&mut buf.iter()).unwrap();
        let expect = u32::MAX;
        assert_eq!(decoded, expect);
    }

    #[test]
    fn decode_uint64() {
        // Uint64
        let buf: &[u8] = &[0xcf, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let decoded = u64::decode(&mut buf.iter()).unwrap();
        let expect = u64::MIN;
        assert_eq!(decoded, expect);

        let buf: &[u8] = &[0xcf, 0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78];
        let decoded = u64::decode(&mut buf.iter()).unwrap();
        let expect = 0x1234567812345678;
        assert_eq!(decoded, expect);

        let buf: &[u8] = &[0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        let decoded = u64::decode(&mut buf.iter()).unwrap();
        let expect = u64::MAX;
        assert_eq!(decoded, expect);
    }
}
