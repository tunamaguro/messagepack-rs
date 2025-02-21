
use super::{Encode, Error, Result};
use crate::formats::{Format};

pub struct BinaryEncoder<'blob> {
    blob: &'blob [u8],
}

impl<'blob> core::ops::Deref for BinaryEncoder<'blob> {
    type Target = &'blob [u8];
    fn deref(&self) -> &Self::Target {
        &self.blob
    }
}

impl<'blob> BinaryEncoder<'blob> {
    pub fn new(blob: &'blob [u8]) -> Self {
        Self { blob }
    }
}

impl Encode for BinaryEncoder<'_> {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let self_len = self.len();
        let format_len = match self_len {
            0x00..=0xff => {
                let cast = self_len as u8;
                let it = Format::Bin8.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(2)
            }
            0x100..=0xffff => {
                let cast = self_len as u16;
                let it = Format::Bin16.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(3)
            }
            0x10000..=0xffffffff => {
                let cast = self_len as u32;
                let it = Format::Bin32.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(5)
            }
            _ => Err(Error::InvalidType),
        }?;

        buf.extend(self.iter().cloned());
        Ok(format_len + self_len)
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        let self_len = self.len();
        let format_len = match self_len {
            0x00..=0xff => {
                const SIZE: usize = 2;
                let cast = self_len as u8;
                let mut it = Format::Bin8.into_iter().chain(cast.to_be_bytes());
                for (to, byte) in buf.zip(&mut it) {
                    *to = byte
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0x100..=0xffff => {
                const SIZE: usize = 3;
                let cast = self_len as u16;
                let mut it = Format::Bin16.into_iter().chain(cast.to_be_bytes());
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
                let mut it = Format::Bin32.into_iter().chain(cast.to_be_bytes());

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

        let mut it = self.iter();

        for (to, byte) in buf.take(self_len).zip(&mut it) {
            *to = *byte
        }
        if it.next().is_none() {
            Ok(format_len + self_len)
        } else {
            Err(Error::BufferFull)
        }
    }
}
