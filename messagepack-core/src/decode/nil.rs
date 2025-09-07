//! Nil and `Option` decoding helpers.

use super::{Decode, Error, Result};
use crate::formats::Format;

/// Decode the MessagePack `nil` value.
pub struct NilDecoder;

impl<'a> Decode<'a> for NilDecoder {
    type Value = ();
    fn decode(buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (format, buf) = Format::decode(buf)?;

        Self::decode_with_format(format, buf)
    }

    fn decode_with_format(format: Format, buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        match format {
            Format::Nil => Ok(((), buf)),
            _ => Err(Error::UnexpectedFormat),
        }
    }
}

impl<'a> Decode<'a> for () {
    type Value = ();
    fn decode(buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        NilDecoder::decode(buf)
    }
    fn decode_with_format(format: Format, buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        NilDecoder::decode_with_format(format, buf)
    }
}

impl<'a, V> Decode<'a> for Option<V>
where
    V: Decode<'a>,
{
    type Value = Option<V::Value>;
    fn decode(buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (format, buf) = Format::decode(buf)?;
        Self::decode_with_format(format, buf)
    }
    fn decode_with_format(format: Format, buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        match format {
            Format::Nil => Ok((None, buf)),
            other => {
                let (val, buf) = V::decode_with_format(other, buf)?;
                Ok((Some(val), buf))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_nil() {
        let buf: &[u8] = &[0xc0];
        let (_, rest) = NilDecoder::decode(buf).unwrap();
        assert_eq!(rest.len(), 0);
    }

    #[test]
    fn decode_option() {
        let buf: &[u8] = &[0xc0];
        let (decoded, rest) = Option::<bool>::decode(buf).unwrap();
        assert_eq!(decoded, None);
        assert_eq!(rest.len(), 0);

        let buf: &[u8] = &[0xc3];
        let (decoded, rest) = Option::<bool>::decode(buf).unwrap();
        assert_eq!(decoded, Some(true));
        assert_eq!(rest.len(), 0);
    }
}
