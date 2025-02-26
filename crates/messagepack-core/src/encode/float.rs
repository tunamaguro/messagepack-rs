use super::{Encode, Error, Result};
use crate::{formats::Format, io::IoWrite};
use num_traits::ToPrimitive;

impl<W> Encode<W> for f32
where
    W: IoWrite,
{
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        writer.write_bytes(&Format::Float32.as_slice())?;
        writer.write_bytes(&self.to_be_bytes())?;
        Ok(5)
    }
}

impl<W> Encode<W> for f64
where
    W: IoWrite,
{
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        writer.write_bytes(&Format::Float64.as_slice())?;
        writer.write_bytes(&self.to_be_bytes())?;
        Ok(9)
    }
}

/// encode minimum byte size
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct EncodeMinimizeFloat<N>(pub N);

impl<W, N> Encode<W> for EncodeMinimizeFloat<N>
where
    W: IoWrite,
    N: ToPrimitive,
{
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        {
            let n = &self.0;

            if let Some(v) = n.to_f32() {
                if v.is_normal() {
                    return v.encode(writer);
                };
            };
            if let Some(v) = n.to_f64() {
                return v.encode(writer);
            };

            Err(Error::InvalidFormat)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case(123.456_f32,[Format::Float32.as_byte(), 0x42, 0xf6, 0xe9, 0x79])]
    fn encode_float32<V: Encode<Vec<u8>>, E: AsRef<[u8]> + Sized>(
        #[case] value: V,
        #[case] expected: E,
    ) {
        let expected = expected.as_ref();

        let mut buf = vec![];
        let n = value.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }

    #[rstest]
    #[case(123456.789_f64,[Format::Float64.as_byte(), 0x40, 0xfe, 0x24, 0x0c, 0x9f, 0xbe, 0x76, 0xc9])]
    fn encode_float64<V: Encode<Vec<u8>>, E: AsRef<[u8]> + Sized>(
        #[case] value: V,
        #[case] expected: E,
    ) {
        let expected = expected.as_ref();

        let mut buf = vec![];
        let n = value.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }

    #[rstest]
    // #[case(123.456_f64, [Format::Float32.as_byte(), 0x42, 0xf6, 0xe9, 0x79])]
    #[case(1e39_f64, [Format::Float64.as_byte(), 0x48,0x07,0x82,0x87,0xf4,0x9c,0x4a,0x1d])]
    fn encode_float_minimize<V: ToPrimitive, E: AsRef<[u8]> + Sized>(
        #[case] value: V,
        #[case] expected: E,
    ) {
        let expected = expected.as_ref();
        let encoder = EncodeMinimizeFloat(value);

        let mut buf = vec![];
        let n = encoder.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }
}
