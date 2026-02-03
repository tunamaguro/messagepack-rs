//! String decoding helpers.

use super::{DecodeBorrowed, Error, NbyteReader};
use crate::{Decode, formats::Format, io::IoRead};

/// Decode a MessagePack string and return a borrowed `&str`.
pub struct StrDecoder;

impl<'de> DecodeBorrowed<'de> for StrDecoder {
    type Value = &'de str;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        let str_ref = ReferenceStrDecoder::decode_with_format(format, reader)?;
        match str_ref {
            ReferenceStr::Borrowed(s) => Ok(s),
            ReferenceStr::Copied(_) => Err(Error::InvalidData),
        }
    }
}

impl<'de> DecodeBorrowed<'de> for &'de str {
    type Value = &'de str;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        StrDecoder::decode_borrowed_with_format(format, reader)
    }
}

/// Borrowed or copied UTF‑8 string reference
pub enum ReferenceStr<'de, 'a> {
    /// Borrowed from the input (`'de`).
    Borrowed(&'de str),
    /// Copied into a transient buffer bound to `'a`.
    Copied(&'a str),
}

/// Decode a MessagePack string and return a [ReferenceStr]
pub struct ReferenceStrDecoder;

impl<'de> Decode<'de> for ReferenceStrDecoder {
    type Value<'a>
        = ReferenceStr<'de, 'a>
    where
        Self: 'a,
        'de: 'a;
    fn decode_with_format<'a, R>(
        format: Format,
        reader: &'a mut R,
    ) -> Result<Self::Value<'a>, Error<R::Error>>
    where
        R: IoRead<'de>,
        'de: 'a,
    {
        let len = match format {
            Format::FixStr(n) => n.into(),
            Format::Str8 => NbyteReader::<1>::read(reader)?,
            Format::Str16 => NbyteReader::<2>::read(reader)?,
            Format::Str32 => NbyteReader::<4>::read(reader)?,
            _ => return Err(Error::UnexpectedFormat),
        };
        let data = reader.read_slice(len).map_err(Error::Io)?;
        match data {
            crate::io::Reference::Borrowed(items) => {
                let s = str::from_utf8(items).map_err(|_| Error::InvalidData)?;
                Ok(ReferenceStr::Borrowed(s))
            }
            crate::io::Reference::Copied(items) => {
                let s = str::from_utf8(items).map_err(|_| Error::InvalidData)?;
                Ok(ReferenceStr::Copied(s))
            }
        }
    }
}

#[cfg(feature = "alloc")]
mod alloc_impl {
    use super::*;
    impl<'de> DecodeBorrowed<'de> for alloc::string::String {
        type Value = alloc::string::String;

        fn decode_borrowed_with_format<R>(
            format: Format,
            reader: &mut R,
        ) -> core::result::Result<Self::Value, Error<R::Error>>
        where
            R: IoRead<'de>,
        {
            let sref = ReferenceStrDecoder::decode_with_format(format, reader)?;
            let owned = match sref {
                ReferenceStr::Borrowed(s) => alloc::string::String::from(s),
                ReferenceStr::Copied(s) => alloc::string::String::from(s),
            };
            Ok(owned)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::Decode;

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

    #[cfg(feature = "alloc")]
    #[test]
    fn decode_string_owned() {
        let buf: &[u8] = &[0xa3, b'f', b'o', b'o'];
        let mut r = crate::io::SliceReader::new(buf);
        let s = <alloc::string::String as Decode>::decode(&mut r).unwrap();
        assert_eq!(s.as_str(), "foo");
        assert!(r.rest().is_empty());
    }
}
