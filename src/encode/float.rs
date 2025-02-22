use super::{Encode, Error, Result};
use crate::formats::Format;

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
        let it = &mut Format::Float32.into_iter().chain(self.to_be_bytes());
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
        let it = &mut Format::Float64.into_iter().chain(self.to_be_bytes());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_float32() {
        let mut buf = vec![];
        (123.456_f32).encode(&mut buf).unwrap();

        let expected: &[u8] = &[0xca, 0x42, 0xf6, 0xe9, 0x79];
        assert_eq!(&buf, expected);
    }

    #[test]
    fn encode_float64() {
        let mut buf = vec![];
        (123456.789_f64).encode(&mut buf).unwrap();

        let expected: &[u8] = &[0xcb, 0x40, 0xfe, 0x24, 0x0c, 0x9f, 0xbe, 0x76, 0xc9];
        assert_eq!(&buf, expected);
    }
}
