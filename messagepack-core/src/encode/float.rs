//! Floating‑point encoders.

use super::{Encode, Result};
use crate::{formats::Format, io::IoWrite};

impl<W> Encode<W> for f32
where
    W: IoWrite,
{
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        let mut buf = [0; 5];
        buf[0] = Format::Float32.as_byte();
        let _ = &buf[1..].copy_from_slice(&self.to_be_bytes());
        writer.write(&buf)?;
        Ok(buf.len())
    }
}

impl<W> Encode<W> for f64
where
    W: IoWrite,
{
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        let mut buf = [0; 9];
        buf[0] = Format::Float64.as_byte();
        let _ = &buf[1..].copy_from_slice(&self.to_be_bytes());
        writer.write(&buf)?;
        Ok(buf.len())
    }
}

fn is_exactly_representable(x: f64) -> bool {
    x.is_finite() && (x as f32) as f64 == x
}

/// encode minimum byte size
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum EncodeMinimizeFloat {
    /// Encode as `float32` if exact, otherwise upcast to `float64`.
    F32(f32),
    /// Always encode as `float64`.
    F64(f64),
}

impl From<f32> for EncodeMinimizeFloat {
    fn from(value: f32) -> Self {
        EncodeMinimizeFloat::F32(value)
    }
}

impl From<f64> for EncodeMinimizeFloat {
    fn from(value: f64) -> Self {
        EncodeMinimizeFloat::F64(value)
    }
}

impl<W> Encode<W> for EncodeMinimizeFloat
where
    W: IoWrite,
{
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        {
            match self {
                EncodeMinimizeFloat::F32(v) => v.encode(writer),
                EncodeMinimizeFloat::F64(v) => {
                    let v = *v;
                    if is_exactly_representable(v) {
                        (v as f32).encode(writer)
                    } else {
                        v.encode(writer)
                    }
                }
            }
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
    #[case(1.0_f64, [Format::Float32.as_byte(), 0x3f, 0x80, 0x00, 0x00])]
    #[case(1e39_f64, [Format::Float64.as_byte(), 0x48,0x07,0x82,0x87,0xf4,0x9c,0x4a,0x1d])]
    fn encode_float_minimize<V: Into<EncodeMinimizeFloat>, E: AsRef<[u8]> + Sized>(
        #[case] value: V,
        #[case] expected: E,
    ) {
        let expected = expected.as_ref();
        let encoder = value.into();

        let mut buf = vec![];
        let n = encoder.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }
}
