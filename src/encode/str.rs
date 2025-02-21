use super::{Encode, Error, Result};
use crate::formats::Format;

impl Encode for &str {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let self_len = self.len();
        let format_len = match self_len {
            0x00..=31 => {
                let cast = self_len as u8;
                let it = Format::FixStr(cast).into_iter();
                buf.extend(it);
                Ok(1)
            }
            32..=0xff => {
                let cast = self_len as u8;
                let it = Format::Str8.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(2)
            }
            0x100..=0xffff => {
                let cast = self_len as u16;
                let it = Format::Str16.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(3)
            }
            0x10000..=0xffffffff => {
                let cast = self_len as u32;
                let it = Format::Str32.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(5)
            }
            _ => Err(Error::InvalidType),
        }?;

        buf.extend(self.as_bytes().iter().cloned());
        Ok(format_len + self_len)
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        let self_len = self.len();
        let format_len = match self_len {
            0x00..=31 => {
                const SIZE: usize = 1;
                let cast = self_len as u8;
                let mut it = Format::FixStr(cast).into_iter();
                for (to, byte) in buf.zip(&mut it) {
                    *to = byte
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            32..=0xff => {
                const SIZE: usize = 2;
                let cast = self_len as u8;
                let mut it = Format::Str8.into_iter().chain(cast.to_be_bytes());
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
                let mut it = Format::Str16.into_iter().chain(cast.to_be_bytes());
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
                let mut it = Format::Str32.into_iter().chain(cast.to_be_bytes());
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

        let mut it = self.as_bytes().iter();
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
