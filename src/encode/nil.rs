use core::iter;

use super::{Encode, Error, Result};
use crate::formats;

impl Encode for () {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        buf.extend(iter::once(formats::NIL));
        Ok(1)
    }

    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        let mut it = iter::once(formats::NIL);
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
