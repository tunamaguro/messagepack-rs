use core::iter;

use super::{Encode, Error, Result};
use crate::formats;

impl Encode for u8 {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        match self {
            0x00..=0x7f => {
                buf.extend(iter::once(*self));
                Ok(1)
            }
            _ => {
                buf.extend(iter::once(formats::UINT8).chain(self.to_be_bytes()));
                Ok(2)
            }
        }
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        match self {
            0x00..=0x7f => {
                const SIZE: usize = 1;
                let mut it = iter::once(*self);
                for (to, byte) in buf.take(SIZE).zip(&mut it) {
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
                let mut it = iter::once(formats::UINT8).chain(self.to_be_bytes());
                for (to, byte) in buf.take(SIZE).zip(&mut it) {
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

macro_rules! impl_encode_unsigned {
    ($ty:ty, $lower:ty, $format:expr, $size:expr) => {
        impl Encode for $ty {
            fn encode<T>(&self, buf: &mut T) -> Result<usize>
            where
                T: Extend<u8>,
            {
                match <$lower>::try_from(*self) {
                    Ok(lower_val) => lower_val.encode(buf),
                    Err(_) => {
                        buf.extend(core::iter::once($format).chain(self.to_be_bytes()));
                        Ok($size)
                    }
                }
            }

            fn encode_to_iter_mut<'a>(
                &self,
                buf: &mut impl Iterator<Item = &'a mut u8>,
            ) -> Result<usize> {
                match <$lower>::try_from(*self) {
                    Ok(lower_val) => lower_val.encode_to_iter_mut(buf),
                    Err(_) => {
                        const SIZE: usize = $size;
                        let mut it = core::iter::once($format).chain(self.to_be_bytes());
                        for (to, byte) in buf.take(SIZE).zip(&mut it) {
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
    };
}
impl_encode_unsigned!(u16, u8, formats::UINT16, 3);
impl_encode_unsigned!(u32, u16, formats::UINT32, 5);
impl_encode_unsigned!(u64, u32, formats::UINT64, 9);

impl Encode for u128 {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        match u64::try_from(*self) {
            Ok(u64_uint) => u64_uint.encode(buf),
            Err(_) => Err(Error::InvalidType),
        }
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        match u64::try_from(*self) {
            Ok(u64_uint) => u64_uint.encode_to_iter_mut(buf),
            Err(_) => Err(Error::InvalidType),
        }
    }
}

impl Encode for i8 {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        match u8::try_from(*self) {
            Ok(u8_int) => u8_int.encode(buf),
            Err(_) => match self {
                -0b11111..=0b00000 => {
                    let cast = *self as u8;
                    let it = iter::once(formats::NEGATIVE_FIXINT | cast);
                    buf.extend(it);
                    Ok(1)
                }
                _ => {
                    let it = iter::once(formats::INT8).chain(self.to_be_bytes());
                    buf.extend(it);
                    Ok(2)
                }
            },
        }
    }

    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        match u8::try_from(*self) {
            Ok(u8_int) => u8_int.encode_to_iter_mut(buf),
            Err(_) => match self {
                -0b11111..=0b00000 => {
                    const SIZE: usize = 1;
                    let cast = *self as u8;
                    let mut it = iter::once(formats::NEGATIVE_FIXINT | cast);
                    for (to, byte) in buf.take(SIZE).zip(&mut it) {
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
                    let mut it = iter::once(formats::INT8).chain(self.to_be_bytes());
                    for (to, byte) in buf.take(SIZE).zip(&mut it) {
                        *to = byte
                    }
                    if it.next().is_none() {
                        Ok(SIZE)
                    } else {
                        Err(Error::BufferFull)
                    }
                }
            },
        }
    }
}

macro_rules! impl_encode_signed {
    ($ty:ty,$lower_unsign:ty, $lower_sign:ty, $format:expr, $size:expr) => {
        impl Encode for $ty {
            fn encode<T>(&self, buf: &mut T) -> Result<usize>
            where
                T: Extend<u8>,
            {
                if let Ok(lower_val) = <$lower_unsign>::try_from(*self) {
                    lower_val.encode(buf)
                } else if let Ok(lower_val) = <$lower_sign>::try_from(*self) {
                    lower_val.encode(buf)
                } else {
                    buf.extend(core::iter::once($format).chain(self.to_be_bytes()));
                    Ok($size)
                }
            }

            fn encode_to_iter_mut<'a>(
                &self,
                buf: &mut impl Iterator<Item = &'a mut u8>,
            ) -> Result<usize> {
                if let Ok(lower_val) = <$lower_unsign>::try_from(*self) {
                    lower_val.encode_to_iter_mut(buf)
                } else if let Ok(lower_val) = <$lower_sign>::try_from(*self) {
                    lower_val.encode_to_iter_mut(buf)
                } else {
                    const SIZE: usize = $size;
                    let mut it = core::iter::once($format).chain(self.to_be_bytes());
                    for (slot, byte) in buf.take(SIZE).zip(&mut it) {
                        *slot = byte;
                    }
                    if it.next().is_none() {
                        Ok(SIZE)
                    } else {
                        Err(Error::BufferFull)
                    }
                }
            }
        }
    };
}
impl_encode_signed!(i16, u8, i8, formats::INT16, 3);
impl_encode_signed!(i32, u16, i16, formats::INT32, 5);
impl_encode_signed!(i64, u32, i32, formats::INT64, 9);

impl Encode for i128 {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        match i64::try_from(*self) {
            Ok(i64_int) => i64_int.encode(buf),
            Err(_) => Err(Error::InvalidType),
        }
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        match i64::try_from(*self) {
            Ok(i64_int) => i64_int.encode_to_iter_mut(buf),
            Err(_) => Err(Error::InvalidType),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_pos_7bit() {
        let expect: &[u8] = &[0x00];
        let buf = &mut [0xff];
        0x00_u8.encode_to_slice(buf).unwrap();
        assert_eq!(buf, expect);

        let expect: &[u8] = &[0x7f];
        let buf = &mut [0xff];
        0x7f_u8.encode_to_slice(buf).unwrap();
        assert_eq!(buf, expect)
    }

    #[test]
    fn encode_uint_8bit() {
        let expect: &[u8] = &[formats::UINT8, 0x80];
        let buf = &mut [0xff; 2];
        128_u8.encode_to_slice(buf).unwrap();
        assert_eq!(buf, expect);

        let expect: &[u8] = &[formats::UINT8, 0xff];
        let buf = &mut [0xff; 2];
        255_u8.encode_to_slice(buf).unwrap();
        assert_eq!(buf, expect);

        let expect: &[u8] = &[formats::UINT8, 0xff];
        let buf = &mut [0xff; 2];
        255_i16.encode_to_slice(buf).unwrap();
        assert_eq!(buf, expect);
    }
}
