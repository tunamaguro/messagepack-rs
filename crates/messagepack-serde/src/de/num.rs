use messagepack_core::{
    Format,
    decode::{Decode, Error},
};
use num_traits::{FromPrimitive, Zero};

/// Decide how to decode numeric values
pub trait NumDecoder<'a> {
    fn decode_u8(buf: &'a [u8]) -> Result<(u8, &'a [u8]), Error>;
    fn decode_u16(buf: &'a [u8]) -> Result<(u16, &'a [u8]), Error>;
    fn decode_u32(buf: &'a [u8]) -> Result<(u32, &'a [u8]), Error>;
    fn decode_u64(buf: &'a [u8]) -> Result<(u64, &'a [u8]), Error>;
    fn decode_u128(buf: &'a [u8]) -> Result<(u128, &'a [u8]), Error>;
    fn decode_i8(buf: &'a [u8]) -> Result<(i8, &'a [u8]), Error>;
    fn decode_i16(buf: &'a [u8]) -> Result<(i16, &'a [u8]), Error>;
    fn decode_i32(buf: &'a [u8]) -> Result<(i32, &'a [u8]), Error>;
    fn decode_i64(buf: &'a [u8]) -> Result<(i64, &'a [u8]), Error>;
    fn decode_i128(buf: &'a [u8]) -> Result<(i128, &'a [u8]), Error>;
    fn decode_f32(buf: &'a [u8]) -> Result<(f32, &'a [u8]), Error>;
    fn decode_f64(buf: &'a [u8]) -> Result<(f64, &'a [u8]), Error>;
}

/// Decode a given numeric value exactly using its native representation.
///
/// This decoder does not perform any conversion or recovery logic.
/// This simply decodes the value as it was originally encoded.
///
/// ## Examples
///
/// ### Succeed
///
/// ```rust
/// use serde::Deserialize;
/// use messagepack_serde::de::{Deserializer, Exact};
///
/// let buf = [1_u8]; // 1 encoded in `positive fixint`
/// let mut de = Deserializer::from_slice(&buf, Exact);
/// let res = u8::deserialize(&mut de).unwrap();
///
/// let expected = 1;
/// assert_eq!(res,expected);
/// ```
///
/// ### Failure
///
/// This decoder does not covert
///
/// ```should_panic
/// use serde::Deserialize;
/// use messagepack_serde::de::{Deserializer, Exact};
///
/// let buf = [1_u8]; // 1 encoded in `positive fixint`
/// let mut de = Deserializer::from_slice(&buf, Exact);
/// let _ = u16::deserialize(&mut de).unwrap();
/// ```
///
/// `u128` always fails because it does not exist in the messagepack format
///
/// ```should_panic
/// use serde::Deserialize;
/// use messagepack_serde::de::{Deserializer, Exact};
///
/// let buf = [1_u8]; // 1 encoded in `positive fixint`
/// let mut de = Deserializer::from_slice(&buf, Exact);
/// let _ = u128::deserialize(&mut de).unwrap();
/// ```
///
pub struct Exact;

impl<'de> NumDecoder<'de> for Exact {
    fn decode_u8(buf: &'de [u8]) -> Result<(u8, &'de [u8]), Error> {
        u8::decode(buf)
    }

    fn decode_u16(buf: &'de [u8]) -> Result<(u16, &'de [u8]), Error> {
        u16::decode(buf)
    }

    fn decode_u32(buf: &'de [u8]) -> Result<(u32, &'de [u8]), Error> {
        u32::decode(buf)
    }

    fn decode_u64(buf: &'de [u8]) -> Result<(u64, &'de [u8]), Error> {
        u64::decode(buf)
    }

    fn decode_u128(_buf: &'de [u8]) -> Result<(u128, &'de [u8]), Error> {
        Err(Error::InvalidData)
    }

    fn decode_i8(buf: &'de [u8]) -> Result<(i8, &'de [u8]), Error> {
        i8::decode(buf)
    }

    fn decode_i16(buf: &'de [u8]) -> Result<(i16, &'de [u8]), Error> {
        i16::decode(buf)
    }

    fn decode_i32(buf: &'de [u8]) -> Result<(i32, &'de [u8]), Error> {
        i32::decode(buf)
    }

    fn decode_i64(buf: &'de [u8]) -> Result<(i64, &'de [u8]), Error> {
        i64::decode(buf)
    }

    fn decode_i128(_buf: &'de [u8]) -> Result<(i128, &'de [u8]), Error> {
        Err(Error::InvalidData)
    }

    fn decode_f32(buf: &'de [u8]) -> Result<(f32, &'de [u8]), Error> {
        f32::decode(buf)
    }

    fn decode_f64(buf: &'de [u8]) -> Result<(f64, &'de [u8]), Error> {
        f64::decode(buf)
    }
}

/// Decode a given numeric value in lenient manner without altering its inherent type.
///
/// This supports both integer and floating-point encodings and converts them into the target type without changing the underlying format.
///
/// ## Note
///
/// ## Example
///
/// ### Succeed
///
/// `u8` -> `u16`
///
/// ```rust
/// use serde::Deserialize;
/// use messagepack_serde::de::{Deserializer, Lenient};
///
/// let buf = [1_u8]; // 1 encoded in `positive fixint`
/// let mut de = Deserializer::from_slice(&buf, Lenient);
/// let res = u16::deserialize(&mut de).unwrap();
///
/// let expected = 1;
/// assert_eq!(res, expected);
/// ```
///
/// `u8` -> `u128`
///
/// ```rust
/// use serde::Deserialize;
/// use messagepack_serde::de::{Deserializer, Lenient};
///
/// let buf = [1_u8]; // 1 encoded in `positive fixint`
/// let mut de = Deserializer::from_slice(&buf, Lenient);
/// let res = u128::deserialize(&mut de).unwrap();
///
/// let expected = 1;
/// assert_eq!(res, expected);
/// ```
///
/// `f32` -> `f64`
///
/// ```rust
/// use serde::Deserialize;
/// use messagepack_serde::de::{Deserializer, Lenient};
///
/// let buf = [0xca,0x3f,0x80,0x00,0x00]; // 1.0 encoded in `float 32`
/// let mut de = Deserializer::from_slice(&buf, Lenient);
/// let res = f64::deserialize(&mut de).unwrap();
///
/// let expected = 1.0;
/// assert_eq!(res, expected);
/// ```
///
/// `f32` -> `f64` with loss
///
/// ```rust
/// use serde::Deserialize;
/// use messagepack_serde::de::{Deserializer, Lenient};
///
/// let buf = [0xca,0x3d,0xcc,0xcc,0xcd]; // 0.1 encoded in `float 32`
/// let mut de = Deserializer::from_slice(&buf, Lenient);
/// let res = f64::deserialize(&mut de).unwrap();
///
/// let expected = 0.1;
/// assert!((res - expected).abs() < 1e-8);
/// ```
///
/// ### Failure
///
/// `f32` -> `u8`
///
/// ```should_panic
/// use serde::Deserialize;
/// use messagepack_serde::de::{Deserializer, Lenient};
///
/// let buf = [0xca,0x3f,0x80,0x00,0x00]; // 1.0 encoded in `float 32`
/// let mut de = Deserializer::from_slice(&buf, Lenient);
/// let _ = u8::deserialize(&mut de).unwrap();
/// ```
///
/// `u8` -> `f32`
///
/// ```should_panic
/// use serde::Deserialize;
/// use messagepack_serde::de::{Deserializer, Lenient};
///
/// let buf = [1_u8]; // 1 encoded in `positive fixint`
/// let mut de = Deserializer::from_slice(&buf, Lenient);
/// let _ = f32::deserialize(&mut de).unwrap();
/// ```
///
pub struct Lenient;

impl Lenient {
    fn decode_int_inner<T: FromPrimitive>(buf: &[u8], format: Format) -> Result<(T, &[u8]), Error> {
        #[cfg(test)]
        {
            dbg!(&format);
        }
        let (n, rest) = match format {
            Format::PositiveFixInt(v) => (T::from_u8(v), buf),
            Format::Uint8 => {
                let (v, rest) = u8::decode_with_format(format, buf)?;
                (T::from_u8(v), rest)
            }
            Format::Uint16 => {
                let (v, rest) = u16::decode_with_format(format, buf)?;
                (T::from_u16(v), rest)
            }
            Format::Uint32 => {
                let (v, rest) = u32::decode_with_format(format, buf)?;
                (T::from_u32(v), rest)
            }
            Format::Uint64 => {
                let (v, rest) = u64::decode_with_format(format, buf)?;
                (T::from_u64(v), rest)
            }
            Format::NegativeFixInt(v) => (T::from_i8(v), buf),
            Format::Int8 => {
                let (v, rest) = i8::decode_with_format(format, buf)?;
                (T::from_i8(v), rest)
            }
            Format::Int16 => {
                let (v, rest) = i16::decode_with_format(format, buf)?;
                (T::from_i16(v), rest)
            }
            Format::Int32 => {
                let (v, rest) = i32::decode_with_format(format, buf)?;
                (T::from_i32(v), rest)
            }
            Format::Int64 => {
                let (v, rest) = i64::decode_with_format(format, buf)?;
                (T::from_i64(v), rest)
            }
            _ => return Err(Error::UnexpectedFormat),
        };

        let n = n.ok_or(Error::InvalidData)?;
        Ok((n, rest))
    }

    fn decode_int<T: FromPrimitive>(buf: &[u8]) -> Result<(T, &[u8]), Error> {
        let (format, rest) = Format::decode(buf)?;

        Self::decode_int_inner(rest, format)
    }

    fn decode_float<T: FromPrimitive>(buf: &[u8]) -> Result<(T, &[u8]), Error> {
        let (format, rest) = Format::decode(buf)?;
        let (n, rest) = match format {
            Format::Float32 => {
                let (v, rest) = f32::decode_with_format(format, rest)?;
                (T::from_f32(v), rest)
            }
            Format::Float64 => {
                let (v, rest) = f64::decode_with_format(format, rest)?;
                (T::from_f64(v), rest)
            }
            _ => return Err(Error::UnexpectedFormat),
        };

        let n = n.ok_or(Error::InvalidData)?;
        Ok((n, rest))
    }
}

impl<'de> NumDecoder<'de> for Lenient {
    fn decode_u8(buf: &'de [u8]) -> Result<(u8, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_u16(buf: &'de [u8]) -> Result<(u16, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_u32(buf: &'de [u8]) -> Result<(u32, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_u64(buf: &'de [u8]) -> Result<(u64, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_u128(buf: &'de [u8]) -> Result<(u128, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_i8(buf: &'de [u8]) -> Result<(i8, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_i16(buf: &'de [u8]) -> Result<(i16, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_i32(buf: &'de [u8]) -> Result<(i32, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_i64(buf: &'de [u8]) -> Result<(i64, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_i128(buf: &'de [u8]) -> Result<(i128, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_f32(buf: &'de [u8]) -> Result<(f32, &'de [u8]), Error> {
        Self::decode_float(buf)
    }

    fn decode_f64(buf: &'de [u8]) -> Result<(f64, &'de [u8]), Error> {
        Self::decode_float(buf)
    }
}

/// Aggressively decode a given numeric value by flexibly interpreting its encoded format.
///
/// This decoder is "aggressive" in that if a numeric value is encoded as a floating-point value with no fractional part, it will be interpreted as an integer.
/// This allows for a more compact representation to be recovered even if the original encoding was done in a float format.
/// For values that cannot be interpreted as integers, the decoder falls back to standard float decoding.
///
/// ## Example
///
/// ### Succeed
///
/// `f32` without fractional part -> `u8`
///
/// ```rust
/// use serde::Deserialize;
/// use messagepack_serde::de::{Deserializer, AggressiveLenient};
///
/// let buf = [0xca,0x3f,0x80,0x00,0x00]; // 1.0 encoded in `float 32`
/// let mut de = Deserializer::from_slice(&buf, AggressiveLenient);
/// let res = u8::deserialize(&mut de).unwrap();
///
/// let expected = 1;
/// assert_eq!(res, expected);
/// ```
///
/// `u8` -> `f32`
///
/// ```rust
/// use serde::Deserialize;
/// use messagepack_serde::de::{Deserializer, AggressiveLenient};
///
/// let buf = [1_u8]; // 1 encoded in `positive fixint`
/// let mut de = Deserializer::from_slice(&buf, AggressiveLenient);
/// let res = f32::deserialize(&mut de).unwrap();
///
/// let expected = 1.0;
/// assert_eq!(res, expected);
/// ```
///
/// ## Failure
///
/// `f32` with fractional part -> `u8`
///
/// ```should_panic
/// use serde::Deserialize;
/// use messagepack_serde::de::{Deserializer, AggressiveLenient};
///
/// let buf = [0xca,0x3f,0x00,0x00,0x00]; // 0.5 encoded in `float 32`
/// let mut de = Deserializer::from_slice(&buf, AggressiveLenient);
/// let _ = u8::deserialize(&mut de).unwrap();
/// ```
///
pub struct AggressiveLenient;

impl AggressiveLenient {
    fn decode_int<T: FromPrimitive>(buf: &[u8]) -> Result<(T, &[u8]), Error> {
        let (format, rest) = Format::decode(buf)?;
        let (n, rest) = match format {
            Format::Float32 => {
                let (v, rest) = f32::decode_with_format(format, rest)?;
                if !v.is_finite() || !v.fract().is_zero() {
                    return Err(Error::InvalidData);
                }
                (T::from_f32(v), rest)
            }
            Format::Float64 => {
                let (v, rest) = f64::decode_with_format(format, rest)?;
                if !v.is_finite() || !v.fract().is_zero() {
                    return Err(Error::InvalidData);
                }
                (T::from_f64(v), rest)
            }
            _ => {
                let (v, rest) = Lenient::decode_int_inner::<T>(buf, format)?;
                (Some(v), rest)
            }
        };

        let n = n.ok_or(Error::InvalidData)?;
        Ok((n, rest))
    }
    fn decode_float<T: FromPrimitive>(buf: &[u8]) -> Result<(T, &[u8]), Error> {
        let (format, rest) = Format::decode(buf)?;
        let (n, rest) = match format {
            Format::Float32 => {
                let (v, rest) = f32::decode_with_format(format, rest)?;
                (T::from_f32(v), rest)
            }
            Format::Float64 => {
                let (v, rest) = f64::decode_with_format(format, rest)?;
                (T::from_f64(v), rest)
            }
            _ => {
                let (v, rest) = Lenient::decode_int_inner::<T>(buf, format)?;
                (Some(v), rest)
            }
        };

        let n = n.ok_or(Error::InvalidData)?;
        Ok((n, rest))
    }
}

impl<'de> NumDecoder<'de> for AggressiveLenient {
    fn decode_u8(buf: &'de [u8]) -> Result<(u8, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_u16(buf: &'de [u8]) -> Result<(u16, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_u32(buf: &'de [u8]) -> Result<(u32, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_u64(buf: &'de [u8]) -> Result<(u64, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_u128(buf: &'de [u8]) -> Result<(u128, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_i8(buf: &'de [u8]) -> Result<(i8, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_i16(buf: &'de [u8]) -> Result<(i16, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_i32(buf: &'de [u8]) -> Result<(i32, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_i64(buf: &'de [u8]) -> Result<(i64, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_i128(buf: &'de [u8]) -> Result<(i128, &'de [u8]), Error> {
        Self::decode_int(buf)
    }

    fn decode_f32(buf: &'de [u8]) -> Result<(f32, &'de [u8]), Error> {
        Self::decode_float(buf)
    }

    fn decode_f64(buf: &'de [u8]) -> Result<(f64, &'de [u8]), Error> {
        Self::decode_float(buf)
    }
}
