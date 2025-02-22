use num_traits::ToPrimitive;

use super::{Encode, Error, Result};
use crate::formats::Format;

impl Encode for u8 {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        match self {
            0x00..=0x7f => {
                buf.extend(Format::PositiveFixInt(*self));
                Ok(1)
            }
            _ => {
                buf.extend(Format::Uint8.into_iter().chain(self.to_be_bytes()));
                Ok(2)
            }
        }
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        match self {
            0x00..=0x7f => {
                const SIZE: usize = 1;
                let it = &mut Format::PositiveFixInt(*self).into_iter();

                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }
                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            _ => {
                const SIZE: usize = 2;
                let it = &mut Format::Uint8.into_iter().chain(self.to_be_bytes());

                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }
                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
        }
    }
}

macro_rules! impl_encode_unsigned {
    ($ty:ty,  $format:expr, $size:expr) => {
        impl Encode for $ty {
            fn encode<T>(&self, buf: &mut T) -> Result<usize>
            where
                T: Extend<u8>,
            {
                buf.extend($format.into_iter().chain(self.to_be_bytes()));
                Ok($size)
            }

            fn encode_to_iter_mut<'a>(
                &self,
                buf: &mut impl Iterator<Item = &'a mut u8>,
            ) -> Result<usize> {
                const SIZE: usize = $size;
                let it = &mut $format.into_iter().chain(self.to_be_bytes());
                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }
                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
        }
    };
}
impl_encode_unsigned!(u16, Format::Uint16, 3);
impl_encode_unsigned!(u32, Format::Uint32, 5);
impl_encode_unsigned!(u64, Format::Uint64, 9);

impl Encode for u128 {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        match u64::try_from(*self) {
            Ok(u64_uint) => u64_uint.encode(buf),
            Err(_) => Err(Error::InvalidFormat),
        }
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        match u64::try_from(*self) {
            Ok(u64_uint) => u64_uint.encode_to_iter_mut(buf),
            Err(_) => Err(Error::InvalidFormat),
        }
    }
}

impl Encode for i8 {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        match self {
            -32..=-1 => {
                let it = Format::NegativeFixInt(*self);
                buf.extend(it);
                Ok(1)
            }
            _ => {
                let it = Format::Int8.into_iter().chain(self.to_be_bytes());
                buf.extend(it);
                Ok(2)
            }
        }
    }

    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        match self {
            -32..=-1 => {
                const SIZE: usize = 1;
                let it = &mut Format::NegativeFixInt(*self).into_iter();
                for (byte, to) in it.zip(buf) {
                    *to = byte
                }
                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            _ => {
                const SIZE: usize = 2;
                let it = &mut Format::Int8.into_iter().chain(self.to_be_bytes());
                for (byte, to) in it.zip(buf) {
                    *to = byte
                }
                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
        }
    }
}

macro_rules! impl_encode_signed {
    ($ty:ty, $format:expr, $size:expr) => {
        impl Encode for $ty {
            fn encode<T>(&self, buf: &mut T) -> Result<usize>
            where
                T: Extend<u8>,
            {
                buf.extend($format.into_iter().chain(self.to_be_bytes()));
                Ok($size)
            }

            fn encode_to_iter_mut<'a>(
                &self,
                buf: &mut impl Iterator<Item = &'a mut u8>,
            ) -> Result<usize> {
                const SIZE: usize = $size;
                let it = &mut $format.into_iter().chain(self.to_be_bytes());
                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }
                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
        }
    };
}
impl_encode_signed!(i16, Format::Int16, 3);
impl_encode_signed!(i32, Format::Int32, 5);
impl_encode_signed!(i64, Format::Int64, 9);

impl Encode for i128 {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        match i64::try_from(*self) {
            Ok(i64_int) => i64_int.encode(buf),
            Err(_) => Err(Error::InvalidFormat),
        }
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        match i64::try_from(*self) {
            Ok(i64_int) => i64_int.encode_to_iter_mut(buf),
            Err(_) => Err(Error::InvalidFormat),
        }
    }
}

/// encode minimum byte size
pub struct EncodeMinimizeInt<N>(pub N);

impl<N> Encode for EncodeMinimizeInt<N>
where
    N: ToPrimitive,
{
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let n = &self.0;
        if let Some(v) = n.to_u8() {
            v.encode(buf)
        } else if let Some(v) = n.to_i8() {
            v.encode(buf)
        } else if let Some(v) = n.to_u16() {
            v.encode(buf)
        } else if let Some(v) = n.to_i16() {
            v.encode(buf)
        } else if let Some(v) = n.to_u32() {
            v.encode(buf)
        } else if let Some(v) = n.to_i32() {
            v.encode(buf)
        } else if let Some(v) = n.to_u64() {
            v.encode(buf)
        } else if let Some(v) = n.to_i64() {
            v.encode(buf)
        } else {
            Err(Error::InvalidFormat)
        }
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        let n = &self.0;
        if let Some(v) = n.to_u8() {
            v.encode_to_iter_mut(buf)
        } else if let Some(v) = n.to_i8() {
            v.encode_to_iter_mut(buf)
        } else if let Some(v) = n.to_u16() {
            v.encode_to_iter_mut(buf)
        } else if let Some(v) = n.to_i16() {
            v.encode_to_iter_mut(buf)
        } else if let Some(v) = n.to_u32() {
            v.encode_to_iter_mut(buf)
        } else if let Some(v) = n.to_i32() {
            v.encode_to_iter_mut(buf)
        } else if let Some(v) = n.to_u64() {
            v.encode_to_iter_mut(buf)
        } else if let Some(v) = n.to_i64() {
            v.encode_to_iter_mut(buf)
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
    fn encode_uint8<V: Encode, E: AsRef<[u8]> + Sized>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();
        {
            let mut buf = vec![];
            let n = value.encode(&mut buf).unwrap();
            assert_eq!(buf, expected);
            assert_eq!(n, expected.len());
        }

        {
            let mut buf = vec![0xff; core::mem::size_of::<E>()];
            let n = value.encode_to_slice(buf.as_mut_slice()).unwrap();
            assert_eq!(&buf, expected);
            assert_eq!(n, expected.len());
        }
    }

    #[rstest]
    #[case(u16::MIN,[Format::Uint16.as_byte(),0x00,0x00])]
    #[case(0x00ff_u16,[Format::Uint16.as_byte(),0x00,0xff])]
    #[case(0x01ff_u16, [Format::Uint16.as_byte(), 0x01, 0xff])]
    #[case(u16::MAX, [Format::Uint16.as_byte(), 0xff, 0xff])]
    fn encode_uint16<V: Encode, E: AsRef<[u8]>>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();
        {
            let mut buf = vec![];
            let n = value.encode(&mut buf).unwrap();
            assert_eq!(buf, expected);
            assert_eq!(n, expected.len());
        }

        {
            let mut buf = vec![0xff; core::mem::size_of::<E>()];
            let n = value.encode_to_slice(buf.as_mut_slice()).unwrap();
            assert_eq!(&buf, expected);
            assert_eq!(n, expected.len());
        }
    }

    #[rstest]
    #[case(u32::MIN, [Format::Uint32.as_byte(), 0x00, 0x00,0x00, 0x00])]
    #[case(0x0000ffff_u32, [Format::Uint32.as_byte(), 0x00, 0x00,0xff, 0xff])]
    #[case(0x0001ffff_u32, [Format::Uint32.as_byte(), 0x00, 0x01,0xff, 0xff])]
    #[case(u32::MAX, [Format::Uint32.as_byte(),0xff, 0xff, 0xff,0xff])]
    fn encode_uint32<V: Encode, E: AsRef<[u8]>>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();
        {
            let mut buf = vec![];
            let n = value.encode(&mut buf).unwrap();
            assert_eq!(buf, expected);
            assert_eq!(n, expected.len());
        }

        {
            let mut buf = vec![0xff; core::mem::size_of::<E>()];
            let n = value.encode_to_slice(buf.as_mut_slice()).unwrap();
            assert_eq!(&buf, expected);
            assert_eq!(n, expected.len());
        }
    }

    #[rstest]
    #[case(u64::MIN, [Format::Uint64.as_byte(), 0x00, 0x00,0x00, 0x00,0x00, 0x00,0x00, 0x00])]
    #[case(u64::MAX, [Format::Uint64.as_byte(), 0xff, 0xff, 0xff,0xff,0xff, 0xff, 0xff,0xff])]
    fn encode_uint64<V: Encode, E: AsRef<[u8]>>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();
        {
            let mut buf = vec![];
            let n = value.encode(&mut buf).unwrap();
            assert_eq!(buf, expected);
            assert_eq!(n, expected.len());
        }

        {
            let mut buf = vec![0xff; core::mem::size_of::<E>()];
            let n = value.encode_to_slice(buf.as_mut_slice()).unwrap();
            assert_eq!(&buf, expected);
            assert_eq!(n, expected.len());
        }
    }

    #[rstest]
    #[case(i8::MIN,[Format::Int8.as_byte(),0x80])]
    #[case(-32_i8,[0xe0])]
    #[case(-1_i8,[0xff])]
    #[case(0_i8,[Format::Int8.as_byte(),0x00])]
    #[case(i8::MAX,[Format::Int8.as_byte(),0x7f])]
    fn encode_int8<V: Encode, E: AsRef<[u8]> + Sized>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();
        {
            let mut buf = vec![];
            let n = value.encode(&mut buf).unwrap();
            assert_eq!(buf, expected);
            assert_eq!(n, expected.len());
        }

        {
            let mut buf = vec![0xff; core::mem::size_of::<E>()];
            let n = value.encode_to_slice(buf.as_mut_slice()).unwrap();
            assert_eq!(&buf, expected);
            assert_eq!(n, expected.len());
        }
    }

    #[rstest]
    #[case(i16::MIN,[Format::Int16.as_byte(),0x80,0x00])]
    #[case(-1_i16,[Format::Int16.as_byte(),0xff,0xff])]
    #[case(0_i16,[Format::Int16.as_byte(),0x00,0x00])]
    #[case(i16::MAX,[Format::Int16.as_byte(),0x7f,0xff])]
    fn encode_int16<V: Encode, E: AsRef<[u8]> + Sized>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();
        {
            let mut buf = vec![];
            let n = value.encode(&mut buf).unwrap();
            assert_eq!(buf, expected);
            assert_eq!(n, expected.len());
        }

        {
            let mut buf = vec![0xff; core::mem::size_of::<E>()];
            let n = value.encode_to_slice(buf.as_mut_slice()).unwrap();
            assert_eq!(&buf, expected);
            assert_eq!(n, expected.len());
        }
    }

    #[rstest]
    #[case(i32::MIN,[Format::Int32.as_byte(),0x80,0x00,0x00,0x00])]
    #[case(-1_i32,[Format::Int32.as_byte(),0xff,0xff,0xff,0xff])]
    #[case(0_i32,[Format::Int32.as_byte(),0x00,0x00,0x00,0x00])]
    #[case(i32::MAX,[Format::Int32.as_byte(),0x7f,0xff,0xff,0xff])]
    fn encode_int32<V: Encode, E: AsRef<[u8]> + Sized>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();
        {
            let mut buf = vec![];
            let n = value.encode(&mut buf).unwrap();
            assert_eq!(buf, expected);
            assert_eq!(n, expected.len());
        }

        {
            let mut buf = vec![0xff; core::mem::size_of::<E>()];
            let n = value.encode_to_slice(buf.as_mut_slice()).unwrap();
            assert_eq!(&buf, expected);
            assert_eq!(n, expected.len());
        }
    }

    #[rstest]
    #[case(i64::MIN,[Format::Int64.as_byte(),0x80,0x00,0x00,0x00,0x00,0x00,0x00,0x00])]
    #[case(-1_i64,[Format::Int64.as_byte(),0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff])]
    #[case(0_i64,[Format::Int64.as_byte(),0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00])]
    #[case(i64::MAX,[Format::Int64.as_byte(),0x7f,0xff,0xff,0xff,0xff,0xff,0xff,0xff])]
    fn encode_int64<V: Encode, E: AsRef<[u8]> + Sized>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();
        {
            let mut buf = vec![];
            let n = value.encode(&mut buf).unwrap();
            assert_eq!(buf, expected);
            assert_eq!(n, expected.len());
        }

        {
            let mut buf = vec![0xff; core::mem::size_of::<E>()];
            let n = value.encode_to_slice(buf.as_mut_slice()).unwrap();
            assert_eq!(&buf, expected);
            assert_eq!(n, expected.len());
        }
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
    fn encode_int_minimize<V: ToPrimitive, E: AsRef<[u8]> + Sized>(
        #[case] value: V,
        #[case] expected: E,
    ) {
        let expected = expected.as_ref();
        let encoder = EncodeMinimizeInt(value);
        {
            let mut buf = vec![];
            let n = encoder.encode(&mut buf).unwrap();
            assert_eq!(buf, expected);
            assert_eq!(n, expected.len());
        }

        {
            let mut buf = vec![0xff; core::mem::size_of::<E>()];
            let n = encoder.encode_to_slice(buf.as_mut_slice()).unwrap();
            assert_eq!(&buf, expected);
            assert_eq!(n, expected.len());
        }
    }
}
