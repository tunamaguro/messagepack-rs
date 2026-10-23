//! Integer encoders and size‑minimising helpers.

use super::{Encode, Error, Result};
use crate::{formats::Format, io::IoWrite};

impl Encode for u8 {
    fn encode<W: IoWrite>(&self, writer: &mut W) -> Result<usize, W::Error> {
        match self {
            0x00..=0x7f => {
                writer.write(&Format::PositiveFixInt(*self).as_slice())?;

                Ok(1)
            }
            _ => {
                let mut buf = [0u8; 2];
                let [marker, rest @ ..] = &mut buf;
                *marker = Format::Uint8.as_byte();
                *rest = self.to_be_bytes();
                writer.write(&buf)?;

                Ok(buf.len())
            }
        }
    }
}

impl Encode for u128 {
    fn encode<W: IoWrite>(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        match u64::try_from(*self) {
            Ok(u64_uint) => u64_uint.encode(writer),
            Err(_) => Err(Error::InvalidFormat),
        }
    }
}

impl Encode for usize {
    fn encode<W: IoWrite>(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        match u64::try_from(*self) {
            Ok(u64_uint) => u64_uint.encode(writer),
            Err(_) => Err(Error::InvalidFormat),
        }
    }
}

impl Encode for i8 {
    fn encode<W: IoWrite>(&self, writer: &mut W) -> Result<usize, W::Error> {
        match self {
            -32..=-1 => {
                writer.write(&Format::NegativeFixInt(*self).as_slice())?;
                Ok(1)
            }
            _ => {
                let mut buf = [0u8; 2];
                let [marker, rest @ ..] = &mut buf;
                *marker = Format::Int8.as_byte();
                *rest = self.to_be_bytes();
                writer.write(&buf)?;

                Ok(buf.len())
            }
        }
    }
}

impl Encode for isize {
    fn encode<W: IoWrite>(&self, writer: &mut W) -> Result<usize, W::Error> {
        match i64::try_from(*self) {
            Ok(i64_int) => i64_int.encode(writer),
            Err(_) => Err(Error::InvalidFormat),
        }
    }
}

impl Encode for i128 {
    fn encode<W: IoWrite>(&self, writer: &mut W) -> Result<usize, W::Error> {
        match i64::try_from(*self) {
            Ok(i64_int) => i64_int.encode(writer),
            Err(_) => Err(Error::InvalidFormat),
        }
    }
}

macro_rules! impl_encode_int {
    ($ty:ty,  $format:expr, $size:expr) => {
        impl Encode for $ty {
            fn encode<W: IoWrite>(&self, writer: &mut W) -> Result<usize, W::Error> {
                let mut buf = [0u8; $size];
                let [marker, rest @ ..] = &mut buf;
                *marker = $format.as_byte();
                *rest = self.to_be_bytes();
                writer.write(&buf)?;
                Ok(buf.len())
            }
        }
    };
}
impl_encode_int!(u16, Format::Uint16, 3);
impl_encode_int!(u32, Format::Uint32, 5);
impl_encode_int!(u64, Format::Uint64, 9);
impl_encode_int!(i16, Format::Int16, 3);
impl_encode_int!(i32, Format::Int32, 5);
impl_encode_int!(i64, Format::Int64, 9);

macro_rules! impl_nonzero_int {
    ($ty:ty) => {
        impl Encode for $ty {
            fn encode<W: IoWrite>(&self, writer: &mut W) -> Result<usize, W::Error> {
                self.get().encode(writer)
            }
        }
    };
}
impl_nonzero_int!(core::num::NonZeroU8);
impl_nonzero_int!(core::num::NonZeroU16);
impl_nonzero_int!(core::num::NonZeroU32);
impl_nonzero_int!(core::num::NonZeroU64);
impl_nonzero_int!(core::num::NonZeroUsize);
impl_nonzero_int!(core::num::NonZeroI8);
impl_nonzero_int!(core::num::NonZeroI16);
impl_nonzero_int!(core::num::NonZeroI32);
impl_nonzero_int!(core::num::NonZeroI64);
impl_nonzero_int!(core::num::NonZeroIsize);

macro_rules! impl_atomic_int {
    ($ty:ty, $bits:literal) => {
        #[cfg(target_has_atomic = $bits)]
        impl Encode for $ty {
            fn encode<W: IoWrite>(&self, writer: &mut W) -> Result<usize, W::Error> {
                self.load(core::sync::atomic::Ordering::Relaxed)
                    .encode(writer)
            }
        }
    };
}
impl_atomic_int!(core::sync::atomic::AtomicU8, "8");
impl_atomic_int!(core::sync::atomic::AtomicU16, "16");
impl_atomic_int!(core::sync::atomic::AtomicU32, "32");
impl_atomic_int!(core::sync::atomic::AtomicU64, "64");
impl_atomic_int!(core::sync::atomic::AtomicUsize, "ptr");
impl_atomic_int!(core::sync::atomic::AtomicI8, "8");
impl_atomic_int!(core::sync::atomic::AtomicI16, "16");
impl_atomic_int!(core::sync::atomic::AtomicI32, "32");
impl_atomic_int!(core::sync::atomic::AtomicI64, "64");
impl_atomic_int!(core::sync::atomic::AtomicIsize, "ptr");

/// Encode an integer using the smallest MessagePack integer format.
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum EncodeMinimizeInt {
    /// 8-bit unsigned integer.
    U8(u8),
    /// 16-bit unsigned integer.
    U16(u16),
    /// 32-bit unsigned integer.
    U32(u32),
    /// 64-bit unsigned integer.
    U64(u64),
    /// 128-bit unsigned integer.
    U128(u128),
    /// 8-bit signed integer.
    I8(i8),
    /// 16-bit signed integer.
    I16(i16),
    /// 32-bit signed integer.
    I32(i32),
    /// 64-bit signed integer.
    I64(i64),
    /// 128-bit signed integer.
    I128(i128),
}

macro_rules! impl_from_int {
    ($($ty:ty => $variant:ident),* $(,)?) => {
        $(
            impl From<$ty> for EncodeMinimizeInt {
                fn from(value: $ty) -> Self {
                    Self::$variant(value)
                }
            }
        )*
    };
}

impl_from_int! {
    u8 => U8,
    u16 => U16,
    u32 => U32,
    u64 => U64,
    u128 => U128,
    i8 => I8,
    i16 => I16,
    i32 => I32,
    i64 => I64,
    i128 => I128,
}

impl From<usize> for EncodeMinimizeInt {
    fn from(value: usize) -> Self {
        Self::U64(value as u64)
    }
}

impl From<isize> for EncodeMinimizeInt {
    fn from(value: isize) -> Self {
        Self::I64(value as i64)
    }
}

macro_rules! encode_min {
    ($v:expr, $writer:expr, $op:tt, $(($ty:ty, $bound:expr)),* $(,)?) => {
        $(
            if $v $op ($bound as _) {
                return ($v as $ty).encode($writer);
            }
        )*
    };
}

impl Encode for EncodeMinimizeInt {
    #[inline]
    fn encode<W: IoWrite>(&self, writer: &mut W) -> Result<usize, W::Error> {
        match *self {
            Self::U8(v) => v.encode(writer),
            Self::U16(v) => {
                encode_min!(v, writer, <=, (u8, u8::MAX));
                v.encode(writer)
            }
            Self::U32(v) => {
                encode_min!(v, writer, <=, (u8, u8::MAX), (u16, u16::MAX));
                v.encode(writer)
            }
            Self::U64(v) => {
                encode_min!(
                    v,
                    writer,
                    <=,
                    (u8, u8::MAX),
                    (u16, u16::MAX),
                    (u32, u32::MAX)
                );
                v.encode(writer)
            }
            Self::U128(v) => {
                encode_min!(
                    v,
                    writer,
                    <=,
                    (u8, u8::MAX),
                    (u16, u16::MAX),
                    (u32, u32::MAX),
                    (u64, u64::MAX)
                );
                Err(Error::InvalidFormat)
            }
            Self::I8(v) => {
                if v >= 0 {
                    (v as u8).encode(writer)
                } else {
                    v.encode(writer)
                }
            }
            Self::I16(v) => {
                if v >= 0 {
                    encode_min!(v, writer, <=, (u8, u8::MAX));
                    (v as u16).encode(writer)
                } else {
                    encode_min!(v, writer, >=, (i8, i8::MIN));
                    v.encode(writer)
                }
            }
            Self::I32(v) => {
                if v >= 0 {
                    encode_min!(v, writer, <=, (u8, u8::MAX), (u16, u16::MAX));
                    (v as u32).encode(writer)
                } else {
                    encode_min!(v, writer, >=, (i8, i8::MIN), (i16, i16::MIN));
                    v.encode(writer)
                }
            }
            Self::I64(v) => {
                if v >= 0 {
                    encode_min!(
                        v,
                        writer,
                        <=,
                        (u8, u8::MAX),
                        (u16, u16::MAX),
                        (u32, u32::MAX)
                    );
                    (v as u64).encode(writer)
                } else {
                    encode_min!(
                        v,
                        writer,
                        >=,
                        (i8, i8::MIN),
                        (i16, i16::MIN),
                        (i32, i32::MIN)
                    );
                    v.encode(writer)
                }
            }
            Self::I128(v) => {
                if v >= 0 {
                    encode_min!(
                        v,
                        writer,
                        <=,
                        (u8, u8::MAX),
                        (u16, u16::MAX),
                        (u32, u32::MAX),
                        (u64, u64::MAX)
                    );
                } else {
                    encode_min!(
                        v,
                        writer,
                        >=,
                        (i8, i8::MIN),
                        (i16, i16::MIN),
                        (i32, i32::MIN),
                        (i64, i64::MIN)
                    );
                }
                Err(Error::InvalidFormat)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(u8::MIN,[0x00])]
    #[case(0x7f_u8,[0x7f])]
    #[case(0x80_u8,[Format::Uint8.as_byte(), 0x80])]
    #[case(u8::MAX,[Format::Uint8.as_byte(), 0xff])]
    fn encode_uint8<V: Encode, E: AsRef<[u8]> + Sized>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();

        let mut buf: Vec<u8> = vec![];
        let n = value.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }

    #[rstest]
    #[case(u16::MIN,[Format::Uint16.as_byte(),0x00,0x00])]
    #[case(0x00ff_u16,[Format::Uint16.as_byte(),0x00,0xff])]
    #[case(0x01ff_u16, [Format::Uint16.as_byte(), 0x01, 0xff])]
    #[case(u16::MAX, [Format::Uint16.as_byte(), 0xff, 0xff])]
    fn encode_uint16<V: Encode, E: AsRef<[u8]>>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();

        let mut buf = vec![];
        let n = value.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }

    #[rstest]
    #[case(u32::MIN, [Format::Uint32.as_byte(), 0x00, 0x00,0x00, 0x00])]
    #[case(0x0000ffff_u32, [Format::Uint32.as_byte(), 0x00, 0x00,0xff, 0xff])]
    #[case(0x0001ffff_u32, [Format::Uint32.as_byte(), 0x00, 0x01,0xff, 0xff])]
    #[case(u32::MAX, [Format::Uint32.as_byte(),0xff, 0xff, 0xff,0xff])]
    fn encode_uint32<V: Encode, E: AsRef<[u8]>>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();

        let mut buf = vec![];
        let n = value.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }

    #[rstest]
    #[case(u64::MIN, [Format::Uint64.as_byte(), 0x00, 0x00,0x00, 0x00,0x00, 0x00,0x00, 0x00])]
    #[case(u64::MAX, [Format::Uint64.as_byte(), 0xff, 0xff, 0xff,0xff,0xff, 0xff, 0xff,0xff])]
    fn encode_uint64<V: Encode, E: AsRef<[u8]>>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();

        let mut buf = vec![];
        let n = value.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }

    #[rstest]
    #[case(i8::MIN,[Format::Int8.as_byte(),0x80])]
    #[case(-32_i8,[0xe0])]
    #[case(-1_i8,[0xff])]
    #[case(0_i8,[Format::Int8.as_byte(),0x00])]
    #[case(i8::MAX,[Format::Int8.as_byte(),0x7f])]
    fn encode_int8<V: Encode, E: AsRef<[u8]> + Sized>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();

        let mut buf = vec![];
        let n = value.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }

    #[rstest]
    #[case(i16::MIN,[Format::Int16.as_byte(),0x80,0x00])]
    #[case(-1_i16,[Format::Int16.as_byte(),0xff,0xff])]
    #[case(0_i16,[Format::Int16.as_byte(),0x00,0x00])]
    #[case(i16::MAX,[Format::Int16.as_byte(),0x7f,0xff])]
    fn encode_int16<V: Encode, E: AsRef<[u8]> + Sized>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();

        let mut buf = vec![];
        let n = value.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }

    #[rstest]
    #[case(i32::MIN,[Format::Int32.as_byte(),0x80,0x00,0x00,0x00])]
    #[case(-1_i32,[Format::Int32.as_byte(),0xff,0xff,0xff,0xff])]
    #[case(0_i32,[Format::Int32.as_byte(),0x00,0x00,0x00,0x00])]
    #[case(i32::MAX,[Format::Int32.as_byte(),0x7f,0xff,0xff,0xff])]
    fn encode_int32<V: Encode, E: AsRef<[u8]> + Sized>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();

        let mut buf = vec![];
        let n = value.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }

    #[rstest]
    #[case(i64::MIN,[Format::Int64.as_byte(),0x80,0x00,0x00,0x00,0x00,0x00,0x00,0x00])]
    #[case(-1_i64,[Format::Int64.as_byte(),0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff])]
    #[case(0_i64,[Format::Int64.as_byte(),0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00])]
    #[case(i64::MAX,[Format::Int64.as_byte(),0x7f,0xff,0xff,0xff,0xff,0xff,0xff,0xff])]
    fn encode_int64<V: Encode, E: AsRef<[u8]> + Sized>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();

        let mut buf = vec![];
        let n = value.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }

    #[rstest]
    #[case(0_u8,[0x00])]
    #[case(0x7f_u8,[0x7f])]
    #[case(0x80_u8,[Format::Uint8.as_byte(),0x80])]
    #[case(u8::MAX,[Format::Uint8.as_byte(),0xff])]
    #[case(0_u16,[0x00])]
    #[case(0x7f_u16,[0x7f])]
    #[case(0x80_u16,[Format::Uint8.as_byte(),0x80])]
    #[case(0xff_u16,[Format::Uint8.as_byte(),0xff])]
    #[case(0x100_u16,[Format::Uint16.as_byte(),0x01,0x00])]
    #[case(u16::MAX,[Format::Uint16.as_byte(),0xff,0xff])]
    #[case(0_u32,[0x00])]
    #[case(0xffff_u32,[Format::Uint16.as_byte(),0xff,0xff])]
    #[case(0x10000_u32,[Format::Uint32.as_byte(),0x00,0x01,0x00,0x00])]
    #[case(u32::MAX,[Format::Uint32.as_byte(),0xff,0xff,0xff,0xff])]
    #[case(0_u64,[0x00])]
    #[case(u32::MAX as u64,[Format::Uint32.as_byte(),0xff,0xff,0xff,0xff])]
    #[case((u32::MAX as u64) + 1,[Format::Uint64.as_byte(),0x00,0x00,0x00,0x01,0x00,0x00,0x00,0x00])]
    #[case(u64::MAX,[Format::Uint64.as_byte(),0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff])]
    #[case(0_u128,[0x00])]
    #[case(u64::MAX as u128,[Format::Uint64.as_byte(),0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff])]
    #[case(i8::MIN,[Format::Int8.as_byte(),0x80])]
    #[case(-33_i8,[Format::Int8.as_byte(),0xdf])]
    #[case(-32_i8,[0xe0])]
    #[case(-1_i8,[0xff])]
    #[case(0_i8,[0x00])]
    #[case(i8::MAX,[0x7f])]
    #[case(i16::MIN,[Format::Int16.as_byte(),0x80,0x00])]
    #[case(-129_i16,[Format::Int16.as_byte(),0xff,0x7f])]
    #[case(-128_i16,[Format::Int8.as_byte(),0x80])]
    #[case(-33_i16,[Format::Int8.as_byte(),0xdf])]
    #[case(-32_i16,[0xe0])]
    #[case(-1_i16,[0xff])]
    #[case(0_i16,[0x00])]
    #[case(0x7f_i16,[0x7f])]
    #[case(0x80_i16,[Format::Uint8.as_byte(),0x80])]
    #[case(0xff_i16,[Format::Uint8.as_byte(),0xff])]
    #[case(0x100_i16,[Format::Uint16.as_byte(),0x01,0x00])]
    #[case(i16::MAX,[Format::Uint16.as_byte(),0x7f,0xff])]
    #[case(i32::MIN,[Format::Int32.as_byte(),0x80,0x00,0x00,0x00])]
    #[case(-32769_i32,[Format::Int32.as_byte(),0xff,0xff,0x7f,0xff])]
    #[case(-32768_i32,[Format::Int16.as_byte(),0x80,0x00])]
    #[case(-129_i32,[Format::Int16.as_byte(),0xff,0x7f])]
    #[case(-128_i32,[Format::Int8.as_byte(),0x80])]
    #[case(-1_i32,[0xff])]
    #[case(0_i32,[0x00])]
    #[case(0x7f_i32,[0x7f])]
    #[case(0x80_i32,[Format::Uint8.as_byte(),0x80])]
    #[case(0xffff_i32,[Format::Uint16.as_byte(),0xff,0xff])]
    #[case(0x10000_i32,[Format::Uint32.as_byte(),0x00,0x01,0x00,0x00])]
    #[case(i32::MAX,[Format::Uint32.as_byte(),0x7f,0xff,0xff,0xff])]
    #[case(i64::MIN,[Format::Int64.as_byte(),0x80,0x00,0x00,0x00,0x00,0x00,0x00,0x00])]
    #[case(-2147483649_i64,[Format::Int64.as_byte(),0xff,0xff,0xff,0xff,0x7f,0xff,0xff,0xff])]
    #[case(-2147483648_i64,[Format::Int32.as_byte(),0x80,0x00,0x00,0x00])]
    #[case(-1_i64,[0xff])]
    #[case(0_i64,[0x00])]
    #[case(0xffff_ffff_i64,[Format::Uint32.as_byte(),0xff,0xff,0xff,0xff])]
    #[case(0x1_0000_0000_i64,[Format::Uint64.as_byte(),0x00,0x00,0x00,0x01,0x00,0x00,0x00,0x00])]
    #[case(i64::MAX,[Format::Uint64.as_byte(),0x7f,0xff,0xff,0xff,0xff,0xff,0xff,0xff])]
    #[case(0_i128,[0x00])]
    #[case(-1_i128,[0xff])]
    #[case(u64::MAX as i128,[Format::Uint64.as_byte(),0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff])]
    #[case(i64::MIN as i128,[Format::Int64.as_byte(),0x80,0x00,0x00,0x00,0x00,0x00,0x00,0x00])]
    fn encode_int_minimize<V: Into<EncodeMinimizeInt>, E: AsRef<[u8]> + Sized>(
        #[case] value: V,
        #[case] expected: E,
    ) {
        let expected = expected.as_ref();
        let encoder: EncodeMinimizeInt = value.into();

        let mut buf = vec![];
        let n = encoder.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }

    #[rstest]
    #[case(u128::MAX)]
    #[case((u64::MAX as u128) + 1)]
    fn encode_int_minimize_rejects_large_u128(#[case] value: u128) {
        let mut buf = vec![];
        let result = EncodeMinimizeInt::from(value).encode(&mut buf);
        assert!(matches!(result, Err(Error::InvalidFormat)));
    }

    #[rstest]
    #[case(i128::MIN)]
    #[case((i64::MIN as i128) - 1)]
    #[case(i128::MAX)]
    fn encode_int_minimize_rejects_large_i128(#[case] value: i128) {
        let mut buf = vec![];
        let result = EncodeMinimizeInt::from(value).encode(&mut buf);
        assert!(matches!(result, Err(Error::InvalidFormat)));
    }

    #[rstest]
    #[case(0_usize, [0x00])]
    #[case(0_isize, [0x00])]
    #[case(-1_isize, [0xff])]
    fn encode_int_minimize_platform_ints<V: Into<EncodeMinimizeInt>, E: AsRef<[u8]> + Sized>(
        #[case] value: V,
        #[case] expected: E,
    ) {
        let expected = expected.as_ref();
        let encoder: EncodeMinimizeInt = value.into();

        let mut buf = vec![];
        let n = encoder.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }
}
