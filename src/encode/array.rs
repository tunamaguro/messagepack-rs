use core::iter;

use super::{Encode, Error, Result};
use crate::formats;

pub struct ArrayEncoder<'array, V> {
    array: &'array [V],
}

impl<'array, V> core::ops::Deref for ArrayEncoder<'array, V> {
    type Target = &'array [V];
    fn deref(&self) -> &Self::Target {
        &self.array
    }
}

impl<'array, V> ArrayEncoder<'array, V> {
    pub fn new(array: &'array [V]) -> Self {
        Self { array }
    }
}

impl<V: Encode> Encode for ArrayEncoder<'_, V>
where
    V: Encode,
{
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let self_len = self.len();
        let format_len = match self_len {
            0x00..=0xf => {
                let cast = self_len as u8;
                let it = iter::once(cast | formats::FIX_ARRAY);
                buf.extend(it);
                Ok(1)
            }
            0xf0..=0xff => {
                let cast = self_len as u16;
                let it = iter::once(formats::ARRAY16).chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(3)
            }
            0xffff..=0xffff => {
                let cast = self_len as u32;
                let it = iter::once(formats::ARRAY32).chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(5)
            }
            _ => Err(Error::InvalidType),
        }?;

        let array_len = self
            .iter()
            .map(|v| v.encode(buf))
            .try_fold(0, |acc, v| v.map(|n| acc + n))?;
        Ok(format_len + array_len)
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        let self_len = self.len();
        let format_len = match self_len {
            0x00..=0xf => {
                const SIZE: usize = 1;
                let cast = self_len as u8;
                let mut it = iter::once(cast | formats::FIX_ARRAY);
                for (to, byte) in buf.take(SIZE).zip(&mut it) {
                    *to = byte
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0xf0..=0xff => {
                const SIZE: usize = 3;
                let cast = self_len as u16;
                let mut it = iter::once(formats::ARRAY16).chain(cast.to_be_bytes());
                for (to, byte) in buf.take(SIZE).zip(&mut it) {
                    *to = byte
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0xffff..=0xffff => {
                const SIZE: usize = 5;
                let cast = self_len as u32;
                let mut it = iter::once(formats::ARRAY32).chain(cast.to_be_bytes());
                for (to, byte) in buf.take(SIZE).zip(&mut it) {
                    *to = byte
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            _ => Err(Error::InvalidType),
        }?;
        let array_len = self
            .iter()
            .map(|v| v.encode_to_iter_mut(buf))
            .try_fold(0, |acc, v| v.map(|n| acc + n))?;
        Ok(format_len + array_len)
    }
}
