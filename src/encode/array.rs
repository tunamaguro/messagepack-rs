use super::{Encode, Error, Result};
use crate::formats::Format;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
            0x00..=0b1111 => {
                let cast = self_len as u8;
                let it = Format::FixArray(cast);
                buf.extend(it);
                Ok(1)
            }
            0x10..=0xffff => {
                let cast = self_len as u16;
                let it = Format::Array16.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(3)
            }
            0x10000..=0xffffffff => {
                let cast = self_len as u32;
                let it = Format::Array32.into_iter().chain(cast.to_be_bytes());
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
            0x00..=0b1111 => {
                const SIZE: usize = 1;
                let cast = self_len as u8;
                let mut it = Format::FixArray(cast).into_iter();
                for (to, byte) in buf.zip(&mut it) {
                    *to = byte
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0x10..=0xffff => {
                const SIZE: usize = 3;
                let cast = self_len as u16;
                let mut it = Format::Array16.into_iter().chain(cast.to_be_bytes());
                for (to, byte) in buf.zip(&mut it) {
                    *to = byte
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0x10000..=0xffffffff => {
                const SIZE: usize = 5;
                let cast = self_len as u32;
                let mut it = Format::Array32.into_iter().chain(cast.to_be_bytes());
                for (to, byte) in buf.zip(&mut it) {
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
