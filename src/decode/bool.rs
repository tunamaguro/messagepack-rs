use super::{Decode, Error, Result};
use crate::formats::Format;

impl Decode for bool {
    fn decode<I, B>(buf: &mut I) -> Result<Self>
    where
        I: Iterator<Item = B>,
        B: core::borrow::Borrow<u8>,
    {
        let format = Format::decode(buf)?;
        match format {
            Format::True => Ok(true),
            Format::False => Ok(false),
            _ => Err(Error::UnexpectedFormat),
        }
    }
}
