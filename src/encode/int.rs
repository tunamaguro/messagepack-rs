use core::iter;

use super::{Encode, Error, Result};
use crate::formats;

impl Encode for u8 {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        match self {
            0x00..0x7f => {
                buf.extend(iter::once(*self));
                Ok(1)
            }
            _ => {
                buf.extend(iter::once(formats::UINT8).chain(self.to_be_bytes()));
                Ok(2)
            }
        }
    }
    fn encode_to_slice(&self, buf: &mut [u8]) -> Result<usize> {
        match self {
            0x00..0x7f => {
                const SIZE: usize = 1;
                let mut it = iter::once(*self);
                for (to, byte) in buf.iter_mut().take(SIZE).zip(&mut it) {
                    *to = byte
                }
                if it.next().is_some() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            _ => {
                const SIZE: usize = 2;
                let mut it = iter::once(formats::UINT8).chain(self.to_be_bytes());
                for (to, byte) in buf.iter_mut().take(SIZE).zip(&mut it) {
                    *to = byte
                }
                if it.next().is_some() {
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

            fn encode_to_slice(&self, buf: &mut [u8]) -> Result<usize> {
                match <$lower>::try_from(*self) {
                    Ok(lower_val) => lower_val.encode_to_slice(buf),
                    Err(_) => {
                        const SIZE: usize = $size;
                        let mut it = core::iter::once($format).chain(self.to_be_bytes());
                        for (to, byte) in buf.iter_mut().take(SIZE).zip(&mut it) {
                            *to = byte;
                        }
                        if it.next().is_some() {
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
