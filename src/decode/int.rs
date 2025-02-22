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

impl Decode for i8 {
    type Value = Self;
    fn decode<I, B>(buf: &mut I) -> Result<Self::Value>
    where
        I: Iterator<Item = B>,
        B: core::borrow::Borrow<u8>,
    {
        match Format::decode(buf)? {
            Format::Int8 => {
                let mut bytes = [0_u8; core::mem::size_of::<u8>()];
                let mut bytes_mut = bytes.iter_mut();
                for (to, byte) in bytes_mut.by_ref().zip(buf) {
                    *to = *byte.borrow();
                }
                Ok(i8::from_be_bytes(bytes))
            }
            Format::NegativeFixInt(v) => Ok(v),
            _ => Err(Error::UnexpectedFormat),
        }
    }
}

macro_rules! impl_decode_signed {
    ($ty:ty,$format:path) => {
        impl Decode for $ty {
            type Value = Self;
            fn decode<I, B>(buf: &mut I) -> Result<Self::Value>
            where
                I: Iterator<Item = B>,
                B: core::borrow::Borrow<u8>,
            {
                match Format::decode(buf)? {
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
impl_decode_signed!(i16, Format::Int16);
impl_decode_signed!(i32, Format::Int32);
impl_decode_signed!(i64, Format::Int64);

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
    fn decode_u8() {
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
    fn decode_u16() {
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
    fn decode_u32() {
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
    fn decode_u64() {
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

    #[test]
    fn decode_i8_fix_neg_int() {
        // FixNegInt
        let buf: &[u8] = &[0xff];
        let decoded = i8::decode(&mut buf.iter()).unwrap();
        let expect = -1;
        assert_eq!(decoded, expect);

        let buf: &[u8] = &[0xe0];
        let decoded = i8::decode(&mut buf.iter()).unwrap();
        let expect = -32;
        assert_eq!(decoded, expect);
    }

    #[test]
    fn decode_i8() {
        // Int8
        let buf: &[u8] = &[0xd0, 0x80];
        let decoded = i8::decode(&mut buf.iter()).unwrap();
        let expect = i8::MIN;
        assert_eq!(decoded, expect);

        let buf: &[u8] = &[0xd0, 0x7f];
        let decoded = i8::decode(&mut buf.iter()).unwrap();
        let expect = i8::MAX;
        assert_eq!(decoded, expect);
    }

    #[test]
    fn decode_i16() {
        // Int16
        let buf: &[u8] = &[0xd1, 0x80, 0x00];
        let decoded = i16::decode(&mut buf.iter()).unwrap();
        let expect = i16::MIN;
        assert_eq!(decoded, expect);

        let buf: &[u8] = &[0xd1, 0x7f, 0xff];
        let decoded = i16::decode(&mut buf.iter()).unwrap();
        let expect = i16::MAX;
        assert_eq!(decoded, expect);
    }

    #[test]
    fn decode_i32() {
        // Int16
        let buf: &[u8] = &[0xd2, 0x80, 0x00, 0x00, 0x00];
        let decoded = i32::decode(&mut buf.iter()).unwrap();
        let expect = i32::MIN;
        assert_eq!(decoded, expect);

        let buf: &[u8] = &[0xd2, 0x7f, 0xff, 0xff, 0xff];
        let decoded = i32::decode(&mut buf.iter()).unwrap();
        let expect = i32::MAX;
        assert_eq!(decoded, expect);
    }

    #[test]
    fn decode_i64() {
        // Int16
        let buf: &[u8] = &[0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let decoded = i64::decode(&mut buf.iter()).unwrap();
        let expect = i64::MIN;
        assert_eq!(decoded, expect);

        let buf: &[u8] = &[0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        let decoded = i64::decode(&mut buf.iter()).unwrap();
        let expect = i64::MAX;
        assert_eq!(decoded, expect);
    }
}
