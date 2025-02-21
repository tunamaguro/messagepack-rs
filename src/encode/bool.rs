use super::{Encode, Error, Result};
use crate::formats::Format;

impl Encode for bool {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        match self {
            true => {
                buf.extend(Format::True);
                Ok(1)
            }
            false => {
                buf.extend(Format::False);
                Ok(1)
            }
        }
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        if let Some(v) = buf.next() {
            match self {
                true => *v = Format::True.as_byte(),
                false => *v = Format::False.as_byte(),
            };
            Ok(1)
        } else {
            Err(Error::BufferFull)
        }
    }
}
