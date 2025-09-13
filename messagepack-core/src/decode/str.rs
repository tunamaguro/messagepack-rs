//! String decoding helpers.

use super::{Decode, Error, NbyteReader};
use crate::{formats::Format, io::IoRead};

/// Decode a MessagePack string and return a borrowed `&str`.
pub struct StrDecoder;

impl<'de> Decode<'de> for StrDecoder {
    type Value<'a> = &'de str;

    fn decode_with_format<'a, R>(
        format: Format,
        reader: &'a mut R,
    ) -> core::result::Result<Self::Value<'a>, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        let len = match format {
            Format::FixStr(n) => n.into(),
            Format::Str8 => NbyteReader::<1>::read(reader)?,
            Format::Str16 => NbyteReader::<2>::read(reader)?,
            Format::Str32 => NbyteReader::<4>::read(reader)?,
            _ => return Err(Error::UnexpectedFormat),
        };
        let data = reader.read_slice(len).map_err(Error::Io)?;
        // Lifetime-sensitive: return only if Borrowed
        let bytes = match data {
            crate::io::Reference::Borrowed(b) => b,
            crate::io::Reference::Copied(_) => return Err(Error::InvalidData),
        };
        let s = core::str::from_utf8(bytes).map_err(|_| Error::InvalidData)?;
        Ok(s)
    }
}

impl<'de> Decode<'de> for &'de str {
    type Value<'a>
        = &'de str
    where
        'de: 'a;

    fn decode_with_format<'a, R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value<'de>, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        StrDecoder::decode_with_format(format, reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_str() {
        let buf: &[u8] = &[
            0xab, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64,
        ];

        let mut r = crate::io::SliceReader::new(buf);
        let decoded = StrDecoder::decode(&mut r).unwrap();
        let expect = "Hello World";
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);
    }

    #[test]
    fn decode_invalid_str() {
        let buf: &[u8] = &[0xa2, 0xc3, 0x28];
        let mut r = crate::io::SliceReader::new(buf);
        let err = StrDecoder::decode(&mut r).unwrap_err();
        assert_eq!(err, Error::InvalidData);

        let buf: &[u8] = &[0xa1, 0x80];
        let mut r = crate::io::SliceReader::new(buf);
        let err = StrDecoder::decode(&mut r).unwrap_err();
        assert_eq!(err, Error::InvalidData);
    }
}
