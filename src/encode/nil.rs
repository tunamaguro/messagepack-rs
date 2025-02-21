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
        let mut it = Format::Nil.into_iter();
        for (to, byte) in buf.zip(&mut it) {
            *to = byte;
        }
        if it.next().is_none() {
            Ok(1)
        } else {
            Err(Error::BufferFull)
        }
    }
}
