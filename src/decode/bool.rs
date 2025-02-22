use super::{Decode, Error, Result};
use crate::formats::Format;

impl Decode for bool {
    type Value = Self;
    fn decode<I, B>(buf: &mut I) -> Result<Self::Value>
    where
        I: Iterator<Item = B>,
        B: core::borrow::Borrow<u8>,
    {
        let format = Format::decode(buf)?;
        Self::decode_with_format(format, buf)
    }
    fn decode_with_format<I, B>(format: Format, buf: &mut I) -> Result<Self::Value>
    where
        I: Iterator<Item = B>,
        B: core::borrow::Borrow<u8>,
    {
        let _ = buf;
        match format {
            Format::True => Ok(true),
            Format::False => Ok(false),
            _ => Err(Error::UnexpectedFormat),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_true() {
        let buf: &[u8] = &[0xc3];
        let decoded = bool::decode(&mut buf.iter()).unwrap();
        let expect = true;
        assert_eq!(decoded, expect);
    }

    #[test]
    fn decode_false() {
        let buf: &[u8] = &[0xc2];
        let decoded = bool::decode(&mut buf.iter()).unwrap();
        let expect = false;
        assert_eq!(decoded, expect);
    }
}

