use super::{DecodeBorrowed, Error};
use crate::{formats::Format, io::IoRead};

macro_rules! impl_decode_float {
    ($ty:ty,$format:path) => {
        impl<'de> DecodeBorrowed<'de> for $ty {
            type Value = Self;

            fn decode_borrowed_with_format<R>(
                format: Format,
                reader: &mut R,
            ) -> core::result::Result<Self::Value, Error<R::Error>>
            where
                R: IoRead<'de>,
            {
                match format {
                    $format => {
                        const SIZE: usize = core::mem::size_of::<$ty>();
                        let bytes = reader.read_slice(SIZE).map_err(Error::Io)?;
                        let slice = bytes.as_bytes();
                        let data: [u8; SIZE] =
                            slice.try_into().map_err(|_| Error::UnexpectedEof)?;
                        let val = <$ty>::from_be_bytes(data);
                        Ok(val)
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
    use crate::decode::Decode;

    #[test]
    fn decode_f32() {
        let buf: &[u8] = &[0xca, 0x42, 0xf6, 0xe9, 0x79];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = f32::decode(&mut r).unwrap();
        let expect = 123.456;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);
    }

    #[test]
    fn decode_f64() {
        let buf: &[u8] = &[0xcb, 0x40, 0xfe, 0x24, 0x0c, 0x9f, 0xcb, 0x0c, 0x02];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = f64::decode(&mut r).unwrap();
        let expect = 123456.789012;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);
    }
}
