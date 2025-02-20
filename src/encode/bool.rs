use core::iter;

use super::{Encode, Error, Result};
use crate::formats;

impl Encode for bool {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        match self {
            true => {
                buf.extend(iter::once(formats::TRUE));
                Ok(1)
            }
            false => {
                buf.extend(iter::once(formats::FALSE));
                Ok(1)
            }
        }
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        if let Some(v) = buf.next() {
            match self {
                true => *v = formats::TRUE,
                false => *v = formats::FALSE,
            };
            Ok(1)
        } else {
            Err(Error::BufferFull)
        }
    }
}
