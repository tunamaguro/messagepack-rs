use messagepack_core::{
    Format,
    decode::{Decode, Error},
};
use num_traits::{FromPrimitive, Zero};

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

struct LosslessLenient;

impl LosslessLenient {
    fn decode_int<T: FromPrimitive>(buf: &[u8]) -> Result<(T, &[u8]), Error> {
        let (format, rest) = Format::decode(buf)?;

        let (n, rest) = match format {
            Format::PositiveFixInt(v) => (T::from_u8(v), rest),
            Format::Uint8 => {
                let (v, rest) = u8::decode_with_format(format, rest)?;
                (T::from_u8(v), rest)
            }
            Format::Uint16 => {
                let (v, rest) = u16::decode_with_format(format, rest)?;
                (T::from_u16(v), rest)
            }
            Format::Uint32 => {
                let (v, rest) = u32::decode_with_format(format, rest)?;
                (T::from_u32(v), rest)
            }
            Format::Uint64 => {
                let (v, rest) = u64::decode_with_format(format, rest)?;
                (T::from_u64(v), rest)
            }
            Format::NegativeFixInt(v) => (T::from_i8(v), rest),
            Format::Int8 => {
                let (v, rest) = i8::decode_with_format(format, rest)?;
                (T::from_i8(v), rest)
            }
            Format::Int16 => {
                let (v, rest) = i16::decode_with_format(format, rest)?;
                (T::from_i16(v), rest)
            }
            Format::Int32 => {
                let (v, rest) = i32::decode_with_format(format, rest)?;
                (T::from_i32(v), rest)
            }
            Format::Int64 => {
                let (v, rest) = i64::decode_with_format(format, rest)?;
                (T::from_i64(v), rest)
            }
            _ => return Err(Error::UnexpectedFormat),
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
            _ => return Err(Error::UnexpectedFormat),
        };

        let n = n.ok_or(Error::InvalidData)?;
        Ok((n, rest))
    }
}

impl<'de> NumDecoder<'de> for LosslessLenient {
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
                let (v, rest) = LosslessLenient::decode_int::<T>(buf)?;
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
                let (v, rest) = LosslessLenient::decode_int::<T>(buf)?;
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
