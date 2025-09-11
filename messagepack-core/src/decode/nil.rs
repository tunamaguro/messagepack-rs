//! Nil and `Option` decoding helpers.

use super::{Decode, Error};
use crate::{formats::Format, io::IoRead};

/// Decode the MessagePack `nil` value.
pub struct NilDecoder;

impl<'de> Decode<'de> for NilDecoder {
    type Value = ();

    fn decode_with_format<R>(
        format: Format,
        _reader: &mut R,
    ) -> core::result::Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        match format {
            Format::Nil => Ok(()),
            _ => Err(Error::UnexpectedFormat),
        }
    }
}

impl<'de> Decode<'de> for () {
    type Value = ();

    fn decode_with_format<R>(format: Format, reader: &mut R) -> core::result::Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        NilDecoder::decode_with_format(format, reader)
    }
}

impl<'de, V> Decode<'de> for Option<V>
where
    V: Decode<'de>,
{
    type Value = Option<V::Value>;

    fn decode_with_format<R>(format: Format, reader: &mut R) -> core::result::Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        match format {
            Format::Nil => Ok(None),
            other => {
                let val = V::decode_with_format(other, reader)?;
                Ok(Some(val))
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
        let mut r = crate::io::SliceReader::new(buf);
        let _ = NilDecoder::decode(&mut r).unwrap();
        assert_eq!(r.rest().len(), 0);
    }

    #[test]
    fn decode_option() {
        let buf: &[u8] = &[0xc0];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = Option::<bool>::decode(&mut r).unwrap();
        assert_eq!(decoded, None);
        assert_eq!(r.rest().len(), 0);

        let buf: &[u8] = &[0xc3];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = Option::<bool>::decode(&mut r).unwrap();
        assert_eq!(decoded, Some(true));
        assert_eq!(r.rest().len(), 0);
    }
}
