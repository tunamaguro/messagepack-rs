use super::{Decode, Error, Result};
use crate::formats::Format;

impl<'a> Decode<'a> for u8 {
    type Value = Self;
    fn decode(buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (format, buf) = Format::decode(buf)?;
        Self::decode_with_format(format, buf)
    }

    fn decode_with_format(format: Format, buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        match format {
            Format::PositiveFixInt(v) => Ok((v, buf)),
            Format::Uint8 => {
                let (first, rest) = buf.split_first().ok_or(Error::EofData)?;
                Ok((*first, rest))
            }
            _ => Err(Error::UnexpectedFormat),
        }
    }
}

impl<'a> Decode<'a> for i8 {
    type Value = Self;
    fn decode(buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (format, buf) = Format::decode(buf)?;
        Self::decode_with_format(format, buf)
    }
    fn decode_with_format(format: Format, buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        match format {
            Format::Int8 => {
                let (first, rest) = buf.split_first().ok_or(Error::EofData)?;
                Ok((*first as i8, rest))
            }
            Format::NegativeFixInt(v) => Ok((v, buf)),
            _ => Err(Error::UnexpectedFormat),
        }
    }
}

macro_rules! impl_decode_int {
    ($ty:ty,$format:path) => {
        impl<'a> Decode<'a> for $ty {
            type Value = Self;

            fn decode(buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
                let (format, buf) = Format::decode(buf)?;
                Self::decode_with_format(format, buf)
            }
            fn decode_with_format(
                format: Format,
                buf: &'a [u8],
            ) -> Result<(Self::Value, &'a [u8])> {
                match format {
                    $format => {
                        const SIZE: usize = core::mem::size_of::<$ty>();

                        let (data, rest) = buf.split_at_checked(SIZE).ok_or(Error::EofData)?;
                        let data: [u8; SIZE] = data.try_into().map_err(|_| Error::EofData)?;
                        let val = <$ty>::from_be_bytes(data);
                        Ok((val, rest))
                    }
                    _ => Err(Error::UnexpectedFormat),
                }
            }
        }
    };
}

impl_decode_int!(u16, Format::Uint16);
impl_decode_int!(u32, Format::Uint32);
impl_decode_int!(u64, Format::Uint64);
impl_decode_int!(i16, Format::Int16);
impl_decode_int!(i32, Format::Int32);
impl_decode_int!(i64, Format::Int64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_u8_fix_pos_int() {
        // FixPosInt
        let buf: &[u8] = &[0x00];
        let (decoded, rest) = u8::decode(buf).unwrap();
        let expect = u8::MIN;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);

        let buf: &[u8] = &[0x7f];
        let (decoded, rest) = u8::decode(buf).unwrap();
        let expect = 0x7f;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);
    }

    #[test]
    fn decode_u8() {
        // Uint8
        let buf: &[u8] = &[0xcc, 0x00];
        let (decoded, rest) = u8::decode(buf).unwrap();
        let expect = u8::MIN;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);

        let buf: &[u8] = &[0xcc, 0xff];
        let (decoded, rest) = u8::decode(buf).unwrap();
        let expect = u8::MAX;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);
    }

    #[test]
    fn decode_u16() {
        // Uint16
        let buf: &[u8] = &[0xcd, 0x00, 0x00];
        let (decoded, rest) = u16::decode(buf).unwrap();
        let expect = u16::MIN;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);

        let buf: &[u8] = &[0xcd, 0x12, 0x34];
        let (decoded, rest) = u16::decode(buf).unwrap();
        let expect = 0x1234;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);

        let buf: &[u8] = &[0xcd, 0xff, 0xff];
        let (decoded, rest) = u16::decode(buf).unwrap();
        let expect = u16::MAX;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);
    }

    #[test]
    fn decode_u32() {
        // Uint32
        let buf: &[u8] = &[0xce, 0x00, 0x00, 0x00, 0x00];
        let (decoded, rest) = u32::decode(buf).unwrap();
        let expect = u32::MIN;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);

        let buf: &[u8] = &[0xce, 0x12, 0x34, 0x56, 0x78];
        let (decoded, rest) = u32::decode(buf).unwrap();
        let expect = 0x12345678;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);

        let buf: &[u8] = &[0xce, 0xff, 0xff, 0xff, 0xff];
        let (decoded, rest) = u32::decode(buf).unwrap();
        let expect = u32::MAX;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);
    }

    #[test]
    fn decode_u64() {
        // Uint64
        let buf: &[u8] = &[0xcf, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let (decoded, rest) = u64::decode(buf).unwrap();
        let expect = u64::MIN;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);

        let buf: &[u8] = &[0xcf, 0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78];
        let (decoded, rest) = u64::decode(buf).unwrap();
        let expect = 0x1234567812345678;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);

        let buf: &[u8] = &[0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        let (decoded, rest) = u64::decode(buf).unwrap();
        let expect = u64::MAX;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);
    }

    #[test]
    fn decode_i8_fix_neg_int() {
        // FixNegInt
        let buf: &[u8] = &[0xff];
        let (decoded, rest) = i8::decode(buf).unwrap();
        let expect = -1;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);

        let buf: &[u8] = &[0xe0];
        let (decoded, rest) = i8::decode(buf).unwrap();
        let expect = -32;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);
    }

    #[test]
    fn decode_i8() {
        // Int8
        let buf: &[u8] = &[0xd0, 0x80];
        let (decoded, rest) = i8::decode(buf).unwrap();
        let expect = i8::MIN;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);

        let buf: &[u8] = &[0xd0, 0x7f];
        let (decoded, rest) = i8::decode(buf).unwrap();
        let expect = i8::MAX;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);
    }

    #[test]
    fn decode_i16() {
        // Int16
        let buf: &[u8] = &[0xd1, 0x80, 0x00];
        let (decoded, rest) = i16::decode(buf).unwrap();
        let expect = i16::MIN;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);

        let buf: &[u8] = &[0xd1, 0x7f, 0xff];
        let (decoded, rest) = i16::decode(buf).unwrap();
        let expect = i16::MAX;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);
    }

    #[test]
    fn decode_i32() {
        // Int16
        let buf: &[u8] = &[0xd2, 0x80, 0x00, 0x00, 0x00];
        let (decoded, rest) = i32::decode(buf).unwrap();
        let expect = i32::MIN;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);

        let buf: &[u8] = &[0xd2, 0x7f, 0xff, 0xff, 0xff];
        let (decoded, rest) = i32::decode(buf).unwrap();
        let expect = i32::MAX;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);
    }

    #[test]
    fn decode_i64() {
        // Int16
        let buf: &[u8] = &[0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let (decoded, rest) = i64::decode(buf).unwrap();
        let expect = i64::MIN;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);

        let buf: &[u8] = &[0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        let (decoded, rest) = i64::decode(buf).unwrap();
        let expect = i64::MAX;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);
    }
}
