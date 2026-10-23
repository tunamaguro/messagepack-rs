//! Floating‑point encoders.

use super::{Encode, Result, int::EncodeMinimizeInt};
use crate::{formats::Format, io::IoWrite};

impl Encode for f32 {
    fn encode<W: IoWrite>(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        let mut buf = [0u8; 5];
        let [marker, rest @ ..] = &mut buf;
        *marker = Format::Float32.as_byte();
        *rest = self.to_be_bytes();
        writer.write(&buf)?;
        Ok(buf.len())
    }
}

impl Encode for f64 {
    fn encode<W: IoWrite>(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        let mut buf = [0u8; 9];
        let [marker, rest @ ..] = &mut buf;
        *marker = Format::Float64.as_byte();
        *rest = self.to_be_bytes();
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

impl Encode for EncodeMinimizeFloat {
    fn encode<W: IoWrite>(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
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

/// Encode a floating-point value using the smaller of its integer and float formats.
///
/// Integral finite values are encoded as integers when that representation is no larger
/// than the floating-point representation. All other values are encoded as floats.
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum EncodeAggressiveFloat {
    /// Encode a 32-bit floating-point value.
    F32(f32),
    /// Encode a 64-bit floating-point value.
    F64(f64),
}

impl From<f32> for EncodeAggressiveFloat {
    fn from(value: f32) -> Self {
        EncodeAggressiveFloat::F32(value)
    }
}

impl From<f64> for EncodeAggressiveFloat {
    fn from(value: f64) -> Self {
        EncodeAggressiveFloat::F64(value)
    }
}

#[inline]
fn is_integral_f32(value: f32) -> bool {
    if value < 0.0 {
        (value as i64) as f32 == value
    } else {
        (value as u64) as f32 == value
    }
}

#[inline]
fn is_integral_f64(value: f64) -> bool {
    if value < 0.0 {
        (value as i64) as f64 == value
    } else {
        (value as u64) as f64 == value
    }
}

#[inline]
fn fits_int_f32(value: f32, min: f32, max: f32) -> bool {
    (min..max).contains(&value) && is_integral_f32(value)
}

#[inline]
fn fits_int_f64(value: f64, min: f64, max: f64) -> bool {
    (min..max).contains(&value) && is_integral_f64(value)
}

#[inline]
fn minimized_int_from_f32(value: f32) -> EncodeMinimizeInt {
    if value < 0.0 {
        EncodeMinimizeInt::I64(value as i64)
    } else {
        EncodeMinimizeInt::U64(value as u64)
    }
}

#[inline]
fn minimized_int_from_f64(value: f64) -> EncodeMinimizeInt {
    if value < 0.0 {
        EncodeMinimizeInt::I64(value as i64)
    } else {
        EncodeMinimizeInt::U64(value as u64)
    }
}

impl Encode for EncodeAggressiveFloat {
    #[inline]
    fn encode<W: IoWrite>(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        match self {
            EncodeAggressiveFloat::F32(v) => {
                let v = *v;
                if fits_int_f32(v, i32::MIN as f32, (1_u64 << 32) as f32) {
                    minimized_int_from_f32(v).encode(writer)
                } else {
                    v.encode(writer)
                }
            }
            EncodeAggressiveFloat::F64(v) => {
                let v = *v;
                if is_exactly_representable(v) {
                    if fits_int_f64(v, i32::MIN as f64, (1_u64 << 32) as f64) {
                        return minimized_int_from_f64(v).encode(writer);
                    }
                    return (v as f32).encode(writer);
                }

                if fits_int_f64(v, i64::MIN as f64, (1_u128 << 64) as f64) {
                    minimized_int_from_f64(v).encode(writer)
                } else {
                    v.encode(writer)
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
    fn encode_float32<V: Encode, E: AsRef<[u8]> + Sized>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();

        let mut buf = vec![];
        let n = value.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }

    #[rstest]
    #[case(123456.789_f64,[Format::Float64.as_byte(), 0x40, 0xfe, 0x24, 0x0c, 0x9f, 0xbe, 0x76, 0xc9])]
    fn encode_float64<V: Encode, E: AsRef<[u8]> + Sized>(#[case] value: V, #[case] expected: E) {
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

    fn assert_aggressive<V: Into<EncodeAggressiveFloat>>(value: V, format: Format, len: usize) {
        let mut buf = vec![];
        let n = value.into().encode(&mut buf).unwrap();

        assert_eq!(n, len);
        assert_eq!(buf.len(), len);
        assert_eq!(buf[0], format.as_byte());
    }

    #[test]
    fn encode_aggressive_float_f32() {
        assert_aggressive(1.0_f32, Format::PositiveFixInt(1), 1);
        assert_aggressive(-0.0_f32, Format::PositiveFixInt(0), 1);
        assert_aggressive(1.5_f32, Format::Float32, 5);
        assert_aggressive(1e10_f32, Format::Float32, 5);
        assert_aggressive(-3e9_f32, Format::Float32, 5);
        assert_aggressive(4294967040.0_f32, Format::Uint32, 5);
        assert_aggressive(4294967296.0_f32, Format::Float32, 5);
        assert_aggressive(-2147483648.0_f32, Format::Int32, 5);
        assert_aggressive(f32::NAN, Format::Float32, 5);
        assert_aggressive(f32::INFINITY, Format::Float32, 5);
    }

    #[test]
    fn encode_aggressive_float_f64() {
        assert_aggressive(1.0_f64, Format::PositiveFixInt(1), 1);
        assert_aggressive(-0.0_f64, Format::PositiveFixInt(0), 1);
        assert_aggressive(1.5_f64, Format::Float32, 5);
        assert_aggressive(1e10_f64, Format::Float32, 5);
        assert_aggressive(4294967295.0_f64, Format::Uint32, 5);
        assert_aggressive(4294967296.0_f64, Format::Float32, 5);
        assert_aggressive(-9223372036854775808.0_f64, Format::Float32, 5);
        assert_aggressive(9223372036854775808.0_f64, Format::Float32, 5);
        assert_aggressive(-((1_u64 << 40) as f64 + 1.0), Format::Int64, 9);
        assert_aggressive((1_u64 << 40) as f64 + 1.0, Format::Uint64, 9);
        assert_aggressive(18446744073709549568.0_f64, Format::Uint64, 9);
        assert_aggressive(1e20_f64, Format::Float64, 9);
        assert_aggressive(f64::NAN, Format::Float64, 9);
        assert_aggressive(f64::INFINITY, Format::Float64, 9);
    }
}
