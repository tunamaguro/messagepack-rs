use messagepack_core::{
    Encode,
    encode::{float::EncodeMinimizeFloat, int::EncodeMinimizeInt},
    io::IoWrite,
};
use num_traits::{Float, ToPrimitive};

use super::error::Error;

/// Decide how numeric values are encoded.
pub trait NumEncoder<W: IoWrite> {
    fn encode_i8(v: i8, writer: &mut W) -> Result<usize, Error<W::Error>>;
    fn encode_i16(v: i16, writer: &mut W) -> Result<usize, Error<W::Error>>;
    fn encode_i32(v: i32, writer: &mut W) -> Result<usize, Error<W::Error>>;
    fn encode_i64(v: i64, writer: &mut W) -> Result<usize, Error<W::Error>>;
    fn encode_i128(v: i128, writer: &mut W) -> Result<usize, Error<W::Error>>;
    fn encode_u8(v: u8, writer: &mut W) -> Result<usize, Error<W::Error>>;
    fn encode_u16(v: u16, writer: &mut W) -> Result<usize, Error<W::Error>>;
    fn encode_u32(v: u32, writer: &mut W) -> Result<usize, Error<W::Error>>;
    fn encode_u64(v: u64, writer: &mut W) -> Result<usize, Error<W::Error>>;
    fn encode_u128(v: u128, writer: &mut W) -> Result<usize, Error<W::Error>>;
    fn encode_f32(v: f32, writer: &mut W) -> Result<usize, Error<W::Error>>;
    fn encode_f64(v: f64, writer: &mut W) -> Result<usize, Error<W::Error>>;
}

/// Encode a given numeric value exactly using its native format.
///
/// This does not minimise or convert, so the value is written as is.
pub struct Exact;

impl<W: IoWrite> NumEncoder<W> for Exact {
    fn encode_i8(v: i8, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Ok(v.encode(writer)?)
    }

    fn encode_i16(v: i16, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Ok(v.encode(writer)?)
    }

    fn encode_i32(v: i32, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Ok(v.encode(writer)?)
    }

    fn encode_i64(v: i64, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Ok(v.encode(writer)?)
    }

    fn encode_i128(v: i128, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Ok(v.encode(writer)?)
    }

    fn encode_u8(v: u8, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Ok(v.encode(writer)?)
    }

    fn encode_u16(v: u16, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Ok(v.encode(writer)?)
    }

    fn encode_u32(v: u32, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Ok(v.encode(writer)?)
    }

    fn encode_u64(v: u64, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Ok(v.encode(writer)?)
    }

    fn encode_u128(v: u128, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Ok(v.encode(writer)?)
    }

    fn encode_f32(v: f32, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Ok(v.encode(writer)?)
    }

    fn encode_f64(v: f64, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Ok(v.encode(writer)?)
    }
}

/// Encode a given numeric value in a lossless minimised format without changing its original format.
///
/// This encoder minimises the encoded size of a numeric value without any loss of information or change in its inherent type.
/// For integer types, it encodes the value using the smallest  integer format that can exactly represent the original value.
/// For floating-point types, it encodes the value using the smallest floating-point format that preserves its precision.
pub struct LosslessMinimize;

impl LosslessMinimize {
    fn encode_int<T: ToPrimitive, W: IoWrite>(
        v: T,
        writer: &mut W,
    ) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Ok(EncodeMinimizeInt(v).encode(writer)?)
    }

    fn encode_float<T: Into<EncodeMinimizeFloat>, W: IoWrite>(
        v: T,
        writer: &mut W,
    ) -> Result<usize, Error<<W as IoWrite>::Error>> {
        let encoder: EncodeMinimizeFloat = v.into();
        let size = encoder.encode(writer)?;
        Ok(size)
    }
}

impl<W: IoWrite> NumEncoder<W> for LosslessMinimize {
    fn encode_i8(v: i8, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_i16(v: i16, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_i32(v: i32, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_i64(v: i64, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_i128(v: i128, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_u8(v: u8, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_u16(v: u16, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_u32(v: u32, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_u64(v: u64, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_u128(v: u128, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_f32(v: f32, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_float(v, writer)
    }

    fn encode_f64(v: f64, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_float(v, writer)
    }
}

/// Encode a given numeric value by aggressively minimising its format.
///
/// If the float is finite and its fractional part is zero, it first tries to encode it as an integer.
/// If this conversion fails, it falls back to encoding the value as a float.
pub struct AggressiveMinimize;

impl AggressiveMinimize {
    fn encode_int<T: ToPrimitive, W: IoWrite>(
        v: T,
        writer: &mut W,
    ) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Ok(EncodeMinimizeInt(v).encode(writer)?)
    }

    fn encode_float<T: Float + Into<EncodeMinimizeFloat>, W: IoWrite>(
        v: T,
        writer: &mut W,
    ) -> Result<usize, Error<<W as IoWrite>::Error>> {
        if v.is_finite() && v.fract().is_zero() {
            let size = Self::encode_int(v, writer).or_else(|_| v.into().encode(writer))?;
            Ok(size)
        } else {
            let size = v.into().encode(writer)?;
            Ok(size)
        }
    }
}

impl<W: IoWrite> NumEncoder<W> for AggressiveMinimize {
    fn encode_i8(v: i8, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_i16(v: i16, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_i32(v: i32, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_i64(v: i64, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_i128(v: i128, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_u8(v: u8, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_u16(v: u16, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_u32(v: u32, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_u64(v: u64, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_u128(v: u128, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_int(v, writer)
    }

    fn encode_f32(v: f32, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_float(v, writer)
    }

    fn encode_f64(v: f64, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_float(v, writer)
    }
}
