//! Integer encoders and size‑minimising helpers.

use num_traits::ToPrimitive;

use super::{Encode, Error, Result};
use crate::{formats::Format, io::IoWrite};

impl<W> Encode<W> for u8
where
    W: IoWrite,
{
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
        match self {
            0x00..=0x7f => {
                writer.write(&Format::PositiveFixInt(*self).as_slice())?;

                Ok(1)
            }
            _ => {
                writer.write(&Format::Uint8.as_slice())?;
                writer.write(&self.to_be_bytes())?;
                Ok(2)
            }
        }
    }
}

impl<W> Encode<W> for u128
where
    W: IoWrite,
{
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        match u64::try_from(*self) {
            Ok(u64_uint) => u64_uint.encode(writer),
            Err(_) => Err(Error::InvalidFormat),
        }
    }
}

impl<W> Encode<W> for usize
where
    W: IoWrite,
{
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        match u64::try_from(*self) {
            Ok(u64_uint) => u64_uint.encode(writer),
            Err(_) => Err(Error::InvalidFormat),
        }
    }
}

impl<W> Encode<W> for i8
where
    W: IoWrite,
{
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
        match self {
            -32..=-1 => {
                writer.write(&Format::NegativeFixInt(*self).as_slice())?;
                Ok(1)
            }
            _ => {
                writer.write(&Format::Int8.as_slice())?;
                writer.write(&self.to_be_bytes())?;

                Ok(2)
            }
        }
    }
}

impl<W> Encode<W> for isize
where
    W: IoWrite,
{
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
        match i64::try_from(*self) {
            Ok(i64_int) => i64_int.encode(writer),
            Err(_) => Err(Error::InvalidFormat),
        }
    }
}

impl<W> Encode<W> for i128
where
    W: IoWrite,
{
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
        match i64::try_from(*self) {
            Ok(i64_int) => i64_int.encode(writer),
            Err(_) => Err(Error::InvalidFormat),
        }
    }
}

macro_rules! impl_encode_int {
    ($ty:ty,  $format:expr, $size:expr) => {
        impl<W> Encode<W> for $ty
        where
            W: IoWrite,
        {
            fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
                writer.write(&$format.as_slice())?;
                writer.write(&self.to_be_bytes())?;
                Ok($size)
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
        impl<W> Encode<W> for $ty
        where
            W: IoWrite,
        {
            fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
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
    ($ty:ty) => {
        impl<W> Encode<W> for $ty
        where
            W: IoWrite,
        {
            fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
                self.load(core::sync::atomic::Ordering::Relaxed)
                    .encode(writer)
            }
        }
    };
}
impl_atomic_int!(core::sync::atomic::AtomicU8);
impl_atomic_int!(core::sync::atomic::AtomicU16);
impl_atomic_int!(core::sync::atomic::AtomicU32);
#[cfg(target_has_atomic = "64")]
impl_atomic_int!(core::sync::atomic::AtomicU64);
impl_atomic_int!(core::sync::atomic::AtomicUsize);
impl_atomic_int!(core::sync::atomic::AtomicI8);
impl_atomic_int!(core::sync::atomic::AtomicI16);
impl_atomic_int!(core::sync::atomic::AtomicI32);
#[cfg(target_has_atomic = "64")]
impl_atomic_int!(core::sync::atomic::AtomicI64);
impl_atomic_int!(core::sync::atomic::AtomicIsize);

/// encode minimum byte size
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct EncodeMinimizeInt<N>(pub N);

impl<W, N> Encode<W> for EncodeMinimizeInt<N>
where
    W: IoWrite,
    N: ToPrimitive,
{
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
        let n = &self.0;
        if let Some(v) = n.to_u8() {
            v.encode(writer)
        } else if let Some(v) = n.to_i8() {
            v.encode(writer)
        } else if let Some(v) = n.to_u16() {
            v.encode(writer)
        } else if let Some(v) = n.to_i16() {
            v.encode(writer)
        } else if let Some(v) = n.to_u32() {
            v.encode(writer)
        } else if let Some(v) = n.to_i32() {
            v.encode(writer)
        } else if let Some(v) = n.to_u64() {
            v.encode(writer)
        } else if let Some(v) = n.to_i64() {
            v.encode(writer)
        } else {
            Err(Error::InvalidFormat)
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
    fn encode_uint8<V: Encode<Vec<u8>>, E: AsRef<[u8]> + Sized>(
        #[case] value: V,
        #[case] expected: E,
    ) {
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
    fn encode_uint16<V: Encode<Vec<u8>>, E: AsRef<[u8]>>(#[case] value: V, #[case] expected: E) {
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
    fn encode_uint32<V: Encode<Vec<u8>>, E: AsRef<[u8]>>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();

        let mut buf = vec![];
        let n = value.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }

    #[rstest]
    #[case(u64::MIN, [Format::Uint64.as_byte(), 0x00, 0x00,0x00, 0x00,0x00, 0x00,0x00, 0x00])]
    #[case(u64::MAX, [Format::Uint64.as_byte(), 0xff, 0xff, 0xff,0xff,0xff, 0xff, 0xff,0xff])]
    fn encode_uint64<V: Encode<Vec<u8>>, E: AsRef<[u8]>>(#[case] value: V, #[case] expected: E) {
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
    fn encode_int8<V: Encode<Vec<u8>>, E: AsRef<[u8]> + Sized>(
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
    #[case(i16::MIN,[Format::Int16.as_byte(),0x80,0x00])]
    #[case(-1_i16,[Format::Int16.as_byte(),0xff,0xff])]
    #[case(0_i16,[Format::Int16.as_byte(),0x00,0x00])]
    #[case(i16::MAX,[Format::Int16.as_byte(),0x7f,0xff])]
    fn encode_int16<V: Encode<Vec<u8>>, E: AsRef<[u8]> + Sized>(
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
    #[case(i32::MIN,[Format::Int32.as_byte(),0x80,0x00,0x00,0x00])]
    #[case(-1_i32,[Format::Int32.as_byte(),0xff,0xff,0xff,0xff])]
    #[case(0_i32,[Format::Int32.as_byte(),0x00,0x00,0x00,0x00])]
    #[case(i32::MAX,[Format::Int32.as_byte(),0x7f,0xff,0xff,0xff])]
    fn encode_int32<V: Encode<Vec<u8>>, E: AsRef<[u8]> + Sized>(
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
    #[case(i64::MIN,[Format::Int64.as_byte(),0x80,0x00,0x00,0x00,0x00,0x00,0x00,0x00])]
    #[case(-1_i64,[Format::Int64.as_byte(),0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff])]
    #[case(0_i64,[Format::Int64.as_byte(),0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00])]
    #[case(i64::MAX,[Format::Int64.as_byte(),0x7f,0xff,0xff,0xff,0xff,0xff,0xff,0xff])]
    fn encode_int64<V: Encode<Vec<u8>>, E: AsRef<[u8]> + Sized>(
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
    #[case(0_i8,[0x00])]
    #[case(0x7f_i8,[0x7f])]
    #[case(0_u16,[0x00])]
    #[case(0x7f_u16,[0x7f])]
    #[case(0x80_u16,[Format::Uint8.as_byte(),0x80])]
    #[case(0_i16,[0x00])]
    #[case(0x7f_i16,[0x7f])]
    #[case(0_u32,[0x00])]
    #[case(0_u64,[0x00])]
    #[case(0_u128,[0x00])]
    #[case(0_i32,[0x00])]
    #[case(0_i64,[0x00])]
    #[case(0_i128,[0x00])]
    #[case(3.0_f32,[0x03])]
    fn encode_int_minimize<V: ToPrimitive, E: AsRef<[u8]> + Sized>(
        #[case] value: V,
        #[case] expected: E,
    ) {
        let expected = expected.as_ref();
        let encoder = EncodeMinimizeInt(value);

        let mut buf = vec![];
        let n = encoder.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }
}
