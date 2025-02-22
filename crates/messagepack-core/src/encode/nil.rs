use super::{Encode, Error, Result};
use crate::formats::Format;

pub struct NilEncoder;

impl Encode for NilEncoder {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        buf.extend(Format::Nil);
        Ok(1)
    }

    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        let it = &mut Format::Nil.into_iter();
        for (byte, to) in it.zip(buf) {
            *to = byte;
        }
        if it.next().is_none() {
            Ok(1)
        } else {
            Err(Error::BufferFull)
        }
    }
}

impl Encode for () {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        NilEncoder.encode(buf)
    }

    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        NilEncoder.encode_to_iter_mut(buf)
    }
}

impl<V> Encode for Option<V>
where
    V: Encode,
{
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        match self {
            Some(other) => other.encode(buf),
            _ => ().encode(buf),
        }
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        match self {
            Some(other) => other.encode_to_iter_mut(buf),
            _ => ().encode_to_iter_mut(buf),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_nil() {
        let mut buf = vec![];
        NilEncoder.encode(&mut buf).unwrap();

        let expected: &[u8] = &[0xc0];
        assert_eq!(&buf, expected);
    }

    #[test]
    fn encode_none() {
        let mut buf = vec![];
        let option: Option<i32> = None;
        option.encode(&mut buf).unwrap();

        let expected: &[u8] = &[0xc0];
        assert_eq!(&buf, expected);
    }

    #[test]
    fn encode_some() {
        let mut buf = vec![];
        let option: Option<u8> = Some(0x80);
        option.encode(&mut buf).unwrap();

        let expected: &[u8] = &[0xcc, 0x80];
        assert_eq!(&buf, expected);
    }
}
