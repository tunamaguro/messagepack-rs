use super::{Decode, Error, Result};
use crate::formats::Format;

pub struct NilDecoder;

impl Decode for NilDecoder {
    type Value = ();
    fn decode<I, B>(buf: &mut I) -> Result<Self::Value>
    where
        I: Iterator<Item = B>,
        B: core::borrow::Borrow<u8>,
    {
        match Format::decode(buf)? {
            Format::Nil => Ok(()),
            _ => Err(Error::UnexpectedFormat),
        }
    }

    fn decode_with_format<I, B>(format: Format, buf: &mut I) -> Result<Self::Value>
    where
        I: Iterator<Item = B>,
        B: core::borrow::Borrow<u8>,
    {
        let _ = (format, buf);
        Ok(())
    }
}

impl Decode for () {
    type Value = ();
    fn decode<I, B>(buf: &mut I) -> Result<Self::Value>
    where
        I: Iterator<Item = B>,
        B: core::borrow::Borrow<u8>,
    {
        NilDecoder::decode(buf)
    }
    fn decode_with_format<I, B>(format: Format, buf: &mut I) -> Result<Self::Value>
    where
        I: Iterator<Item = B>,
        B: core::borrow::Borrow<u8>,
    {
        NilDecoder::decode_with_format(format, buf)
    }
}

impl<V> Decode for Option<V>
where
    V: Decode,
{
    type Value = Option<V::Value>;
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
        let op = match format {
            Format::Nil => None,
            other => Some(V::decode_with_format(other, buf)?),
        };

        Ok(op)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_nil() {
        let buf: &[u8] = &[0xc0];
        NilDecoder::decode(&mut buf.iter()).unwrap();
    }

    #[test]
    fn decode_option() {
        let buf: &[u8] = &[0xc0];
        let decoded = Option::<bool>::decode(&mut buf.iter()).unwrap();
        assert_eq!(decoded, None);

        let buf: &[u8] = &[0xc3];
        let decoded = Option::<bool>::decode(&mut buf.iter()).unwrap();
        assert_eq!(decoded, Some(true));
    }
}
