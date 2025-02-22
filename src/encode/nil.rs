use super::{Encode, Error, Result};
use crate::formats::Format;

impl Encode for () {
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
