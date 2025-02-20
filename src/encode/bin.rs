use core::iter;

use super::{Encode, Error, Result};
use crate::formats;

impl Encode for [u8] {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let self_len = self.len();
        let format_len = match self_len {
            0x00..0xff => {
                let cast = self_len as u8;
                let it = iter::once(formats::BIN8).chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(2)
            }
            0xff..0xffff => {
                let cast = self_len as u16;
                let it = iter::once(formats::BIN16).chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(3)
            }
            0xffff..=0xffffff => {
                let cast = self_len as u32;
                let it = iter::once(formats::BIN32).chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(5)
            }
            _ => Err(Error::InvalidType),
        }?;

        buf.extend(self.iter().cloned());
        Ok(format_len + self_len)
    }
    fn encode_to_slice(&self, buf: &mut [u8]) -> Result<usize> {
        let self_len = self.len();
        let format_len = match self_len {
            0x00..0xff => {
                const SIZE: usize = 2;
                let cast = self_len as u8;
                let mut it = iter::once(formats::BIN8).chain(cast.to_be_bytes());
                for (to, byte) in buf.iter_mut().take(SIZE).zip(&mut it) {
                    *to = byte
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0xff..0xffff => {
                const SIZE: usize = 3;
                let cast = self_len as u16;
                let mut it = iter::once(formats::BIN16).chain(cast.to_be_bytes());
                for (to, byte) in buf.iter_mut().take(SIZE).zip(&mut it) {
                    *to = byte
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0xffff..=0xffffff => {
                const SIZE: usize = 5;
                let cast = self_len as u32;
                let mut it = iter::once(formats::BIN32).chain(cast.to_be_bytes());

                for (to, byte) in buf.iter_mut().take(SIZE).zip(&mut it) {
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

        for (to, byte) in buf.iter_mut().skip(format_len).take(self_len).zip(&mut it) {
            *to = *byte
        }
        if it.next().is_none() {
            Ok(format_len + self_len)
        } else {
            Err(Error::BufferFull)
        }
    }
}
