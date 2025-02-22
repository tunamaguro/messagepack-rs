use super::{Decode, Error, Result};
use crate::formats::Format;

macro_rules! impl_decode_float {
    ($ty:ty,$format:path) => {
        impl Decode for $ty {
            type Value = Self;
            fn decode<I, B>(buf: &mut I) -> Result<Self::Value>
            where
                I: Iterator<Item = B>,
                B: core::borrow::Borrow<u8>,
            {
                let format = Format::decode(buf)?;
                Self::decode_with_format(format, buf)
            }
            fn decode_with_format<I, B>(format: Format, buf: &mut I) -> Result<Self::Value>
            where
                I: Iterator<Item = B>,
                B: core::borrow::Borrow<u8>,
            {
                match format {
                    $format => {
                        let mut bytes = [0_u8; core::mem::size_of::<Self>()];
                        let mut bytes_mut = bytes.iter_mut();
                        for (to, byte) in bytes_mut.by_ref().zip(buf) {
                            *to = *byte.borrow();
                        }
                        if bytes_mut.next().is_none() {
                            Ok(<Self>::from_be_bytes(bytes))
                        } else {
                            Err(Error::EofData)
                        }
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
        let decoded = f32::decode(&mut buf.iter()).unwrap();
        let expect = 123.456;
        assert_eq!(decoded, expect);
    }

    #[test]
    fn decode_f64() {
        let buf: &[u8] = &[0xcb, 0x40, 0xfe, 0x24, 0x0c, 0x9f, 0xcb, 0x0c, 0x02];
        let decoded = f64::decode(&mut buf.iter()).unwrap();
        let expect = 123456.789012;
        assert_eq!(decoded, expect);
    }
}
