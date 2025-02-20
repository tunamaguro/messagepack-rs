use core::iter;

use super::{Encode, Error, Result};
use crate::formats;

impl Encode for str {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let self_len = self.len();
        let format_len = match self_len {
            0x00..=0x1f => {
                let cast = self_len as u8;
                let it = iter::once(cast | formats::FIX_STR);
                buf.extend(it);
                Ok(1)
            }
            0x20..=0xff => {
                let cast = self_len as u8;
                let it = iter::once(formats::STR8).chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(2)
            }
            0x100..=0xffff => {
                let cast = self_len as u16;
                let it = iter::once(formats::STR16).chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(3)
            }
            0x10000..=0xffffffff => {
                let cast = self_len as u32;
                let it = iter::once(formats::STR32).chain(cast.to_be_bytes());
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
            0x00..=0x1f => {
                const SIZE: usize = 1;
                let cast = self_len as u8;
                let mut it = iter::once(cast | formats::FIX_STR);
                for (to, byte) in buf.take(SIZE).zip(&mut it) {
                    *to = byte
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0x20..=0xff => {
                const SIZE: usize = 2;
                let cast = self_len as u8;
                let mut it = iter::once(formats::STR8).chain(cast.to_be_bytes());
                for (to, byte) in buf.take(SIZE).zip(&mut it) {
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
                let mut it = iter::once(formats::STR16).chain(cast.to_be_bytes());
                for (to, byte) in buf.take(SIZE).zip(&mut it) {
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
                let mut it = iter::once(formats::STR32).chain(cast.to_be_bytes());
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
