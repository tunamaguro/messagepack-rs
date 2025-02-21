
use super::{Encode, Error, Result};
use crate::formats::{Format};

impl Encode for f32 {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let it = Format::Float32.into_iter().chain(self.to_be_bytes());
        buf.extend(it);
        Ok(5)
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        const SIZE: usize = 5;
        let mut it = Format::Float32.into_iter().chain(self.to_be_bytes());
        for (to, byte) in buf.zip(&mut it) {
            *to = byte
        }
        if it.next().is_none() {
            Ok(SIZE)
        } else {
            Err(Error::BufferFull)
        }
    }
}

impl Encode for f64 {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let it = Format::Float64.into_iter().chain(self.to_be_bytes());
        buf.extend(it);
        Ok(9)
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        const SIZE: usize = 9;
        let mut it = Format::Float64.into_iter().chain(self.to_be_bytes());
        for (to, byte) in buf.zip(&mut it) {
            *to = byte
        }
        if it.next().is_none() {
            Ok(SIZE)
        } else {
            Err(Error::BufferFull)
        }
    }
}
