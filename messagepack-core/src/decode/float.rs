use super::{Decode, Error, Result};
use crate::formats::Format;

macro_rules! impl_decode_float {
    ($ty:ty,$format:path) => {
        impl<'a> Decode<'a> for $ty {
            type Value = Self;

            fn decode(buf: &[u8]) -> Result<(Self::Value, &[u8])> {
                let (format, buf) = Format::decode(buf)?;
                Self::decode_with_format(format, buf)
            }
            fn decode_with_format(format: Format, buf: &[u8]) -> Result<(Self::Value, &[u8])> {
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

impl_decode_float!(f32, Format::Float32);
impl_decode_float!(f64, Format::Float64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_f32() {
        let buf: &[u8] = &[0xca, 0x42, 0xf6, 0xe9, 0x79];
        let (decoded, rest) = f32::decode(buf).unwrap();
        let expect = 123.456;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);
    }

    #[test]
    fn decode_f64() {
        let buf: &[u8] = &[0xcb, 0x40, 0xfe, 0x24, 0x0c, 0x9f, 0xcb, 0x0c, 0x02];
        let (decoded, rest) = f64::decode(buf).unwrap();
        let expect = 123456.789012;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);
    }
}
