use messagepack_core::{
    Encode,
    encode::{Error, float::EncodeMinimizeFloat, int::EncodeMinimizeInt},
    io::IoWrite,
};
use num_traits::{ToPrimitive, float::FloatCore};

/// Decide how numeric values are encoded.
pub trait NumEncoder<W: IoWrite> {
    /// decide encode i8
    fn encode_i8(v: i8, writer: &mut W) -> Result<usize, Error<W::Error>>;
    /// decide encode i16
    fn encode_i16(v: i16, writer: &mut W) -> Result<usize, Error<W::Error>>;
    /// decide encode i32
    fn encode_i32(v: i32, writer: &mut W) -> Result<usize, Error<W::Error>>;
    /// decide encode i64
    fn encode_i64(v: i64, writer: &mut W) -> Result<usize, Error<W::Error>>;
    /// decide encode i128
    fn encode_i128(v: i128, writer: &mut W) -> Result<usize, Error<W::Error>> {
        match i64::try_from(v) {
            Ok(i64_int) => Self::encode_i64(i64_int, writer),
            Err(_) => Err(Error::InvalidFormat),
        }
    }
    /// decide encode u8
    fn encode_u8(v: u8, writer: &mut W) -> Result<usize, Error<W::Error>>;
    /// decide encode u16
    fn encode_u16(v: u16, writer: &mut W) -> Result<usize, Error<W::Error>>;
    /// decide encode u32
    fn encode_u32(v: u32, writer: &mut W) -> Result<usize, Error<W::Error>>;
    /// decide encode u64
    fn encode_u64(v: u64, writer: &mut W) -> Result<usize, Error<W::Error>>;
    /// decide encode u128
    fn encode_u128(v: u128, writer: &mut W) -> Result<usize, Error<W::Error>> {
        match u64::try_from(v) {
            Ok(u64_uint) => Self::encode_u64(u64_uint, writer),
            Err(_) => Err(Error::InvalidFormat),
        }
    }
    /// decide encode f32
    fn encode_f32(v: f32, writer: &mut W) -> Result<usize, Error<W::Error>>;
    /// decide encode f64
    fn encode_f64(v: f64, writer: &mut W) -> Result<usize, Error<W::Error>>;
}

/// Encode a given numeric value exactly using its native format.
///
/// This does not minimise or convert, so the value is written as is.
///
/// ## Examples
///
/// `u8` is encoded to `positive fixint` or `uint 8`
///
/// ```rust
/// use serde::Serialize;
/// use messagepack_serde::ser::{to_slice_with_config, Exact};
///
/// let mut buf = [0_u8;1];
/// to_slice_with_config(&1_u8, &mut buf, Exact).unwrap();
///
/// let expected = [1_u8]; // 1 encoded in `positive fixint`
/// assert_eq!(buf,expected);
/// ```
///  
/// `u16` is encoded to `uint 16`
///
/// ```rust
/// use serde::Serialize;
/// use messagepack_serde::ser::{to_slice_with_config, Exact};
///
/// let mut buf = [0_u8;3];
/// to_slice_with_config(&1_u16, &mut buf, Exact).unwrap();
///
/// let expected = [0xcd_u8, 0x00_u8, 1_u8]; // 1 encoded in `uint 16`
/// assert_eq!(buf,expected);
/// ```
pub struct Exact;

impl<W: IoWrite> NumEncoder<W> for Exact {
    fn encode_i8(v: i8, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        v.encode(writer)
    }

    fn encode_i16(v: i16, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        v.encode(writer)
    }

    fn encode_i32(v: i32, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        v.encode(writer)
    }

    fn encode_i64(v: i64, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        v.encode(writer)
    }

    fn encode_i128(v: i128, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        v.encode(writer)
    }

    fn encode_u8(v: u8, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        v.encode(writer)
    }

    fn encode_u16(v: u16, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        v.encode(writer)
    }

    fn encode_u32(v: u32, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        v.encode(writer)
    }

    fn encode_u64(v: u64, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        v.encode(writer)
    }

    fn encode_u128(v: u128, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        v.encode(writer)
    }

    fn encode_f32(v: f32, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        v.encode(writer)
    }

    fn encode_f64(v: f64, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        v.encode(writer)
    }
}

/// Encode a given numeric value in a lossless minimised format without changing its original format.
///
/// This encoder minimises the encoded size of a numeric value without any loss of information or change in its inherent type.
/// For integer types, it encodes the value using the smallest integer format that can exactly represent the original value.
/// For floating-point types, it encodes the value using the smallest floating-point format that preserves its precision.
///
/// ## Examples
///
/// If there is no loss in a smaller format, it is encoded with that value
///
/// ```rust
/// use serde::Serialize;
/// use messagepack_serde::ser::{to_slice_with_config, LosslessMinimize};
///
/// let mut buf = [0_u8;1];
/// to_slice_with_config(&1_u16, &mut buf, LosslessMinimize).unwrap();
/// let expected = [1_u8]; // 1 encoded in `positive fixint`
/// assert_eq!(buf,expected);
/// ```
///
/// Floating point is encoded as floating point type
///
/// ```rust
/// use serde::Serialize;
/// use messagepack_serde::ser::{to_slice_with_config, LosslessMinimize};
///
/// let mut buf = [0_u8;5];
/// to_slice_with_config(&1.0_f32, &mut buf, LosslessMinimize).unwrap();
///
/// let expected = [0xca,0x3f,0x80,0x00,0x00]; // 1.0 encoded in `float 32`
/// assert_eq!(buf,expected);
/// ```
///
/// If floating point can be encoded without loss, it will be encoded in a smaller format
///
/// ```rust
/// use serde::Serialize;
/// use messagepack_serde::ser::{to_slice_with_config, LosslessMinimize};
///
/// let mut buf = [0_u8;5];
/// to_slice_with_config(&1.0_f64, &mut buf, LosslessMinimize).unwrap();
///
/// let expected = [0xca,0x3f,0x80,0x00,0x00]; // 1.0 encoded in `float 32`
/// assert_eq!(buf,expected);
/// ```
///
/// `0.1` is encoded as `float 64` due to loss when converted to `f32
///
/// ```rust
/// use serde::Serialize;
/// use messagepack_serde::ser::{to_slice_with_config, LosslessMinimize};
///
/// let mut buf = [0_u8;9];
/// to_slice_with_config(&0.1_f64, &mut buf, LosslessMinimize).unwrap();
///
/// let expected = [0xcb,0x3f,0xb9,0x99,0x99,0x99,0x99,0x99,0x9a]; // 0.1 encoded in `float 64`
/// assert_eq!(buf,expected);
/// ```
pub struct LosslessMinimize;

impl LosslessMinimize {
    fn encode_int<T: ToPrimitive, W: IoWrite>(
        v: T,
        writer: &mut W,
    ) -> Result<usize, Error<<W as IoWrite>::Error>> {
        EncodeMinimizeInt(v).encode(writer)
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
///
/// ## Examples
///
/// If there is no loss in a smaller format, it is encoded with that value
///
/// ```rust
/// use serde::Serialize;
/// use messagepack_serde::ser::{to_slice_with_config, AggressiveMinimize};
///
/// let mut buf = [0_u8;1];
/// to_slice_with_config(&1_u16, &mut buf, AggressiveMinimize).unwrap();
///
/// let expected = [1_u8]; // 1 encoded in `positive fixint`
/// assert_eq!(buf,expected);
/// ```
///
/// Floating point without fractional part is encoded as `int`
///
/// ```rust
/// use serde::Serialize;
/// use messagepack_serde::ser::{to_slice_with_config, AggressiveMinimize};
///
/// let mut buf = [0_u8;1];
/// to_slice_with_config(&1.0_f32, &mut buf, AggressiveMinimize).unwrap();
///
/// let expected = [1_u8]; // 1 encoded in `positive fixint`
/// assert_eq!(buf,expected);
/// ```
///
/// `f64` is encoded in the same way as `f32`.
///
/// ```rust
/// use serde::Serialize;
/// use messagepack_serde::ser::{to_slice_with_config, AggressiveMinimize};
///
/// let mut buf = [0_u8;1];
/// to_slice_with_config(&1.0_f64, &mut buf, AggressiveMinimize).unwrap();
///
/// let expected = [1_u8]; // 1 encoded in `positive fixint`
/// assert_eq!(buf,expected);
/// ```
pub struct AggressiveMinimize;

impl AggressiveMinimize {
    fn encode_float<T: FloatCore + Into<EncodeMinimizeFloat>, W: IoWrite>(
        v: T,
        writer: &mut W,
    ) -> Result<usize, Error<<W as IoWrite>::Error>> {
        if v.is_finite() && v.fract().is_zero() {
            let size =
                LosslessMinimize::encode_int(v, writer).or_else(|_| v.into().encode(writer))?;
            Ok(size)
        } else {
            let size = v.into().encode(writer)?;
            Ok(size)
        }
    }
}

impl<W: IoWrite> NumEncoder<W> for AggressiveMinimize {
    fn encode_i8(v: i8, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        LosslessMinimize::encode_int(v, writer)
    }

    fn encode_i16(v: i16, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        LosslessMinimize::encode_int(v, writer)
    }

    fn encode_i32(v: i32, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        LosslessMinimize::encode_int(v, writer)
    }

    fn encode_i64(v: i64, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        LosslessMinimize::encode_int(v, writer)
    }

    fn encode_i128(v: i128, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        LosslessMinimize::encode_int(v, writer)
    }

    fn encode_u8(v: u8, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        LosslessMinimize::encode_int(v, writer)
    }

    fn encode_u16(v: u16, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        LosslessMinimize::encode_int(v, writer)
    }

    fn encode_u32(v: u32, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        LosslessMinimize::encode_int(v, writer)
    }

    fn encode_u64(v: u64, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        LosslessMinimize::encode_int(v, writer)
    }

    fn encode_u128(v: u128, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        LosslessMinimize::encode_int(v, writer)
    }

    fn encode_f32(v: f32, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_float(v, writer)
    }

    fn encode_f64(v: f64, writer: &mut W) -> Result<usize, Error<<W as IoWrite>::Error>> {
        Self::encode_float(v, writer)
    }
}
