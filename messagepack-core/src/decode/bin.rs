//! Binary (bin8/16/32) decoding helpers.

use super::{Error, NbyteReader};
use crate::{Decode, decode::DecodeBorrowed, formats::Format, io::IoRead};

/// Decode a MessagePack binary blob and return a borrowed byte slice.
pub struct BinDecoder;

impl<'de> DecodeBorrowed<'de> for BinDecoder {
    type Value = &'de [u8];

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        let data = ReferenceDecoder::decode_with_format(format, reader)?;
        match data {
            crate::io::Reference::Borrowed(b) => Ok(b),
            crate::io::Reference::Copied(_) => Err(Error::InvalidData),
        }
    }
}

impl<'de> DecodeBorrowed<'de> for &'de [u8] {
    type Value = &'de [u8];

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        BinDecoder::decode_borrowed_with_format(format, reader)
    }
}

/// Decode a MessagePack binary and return a `Reference` to its bytes
pub struct ReferenceDecoder;

impl<'de> super::Decode<'de> for ReferenceDecoder {
    type Value<'a>
        = crate::io::Reference<'de, 'a>
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
            Format::Bin8 => NbyteReader::<1>::read(reader)?,
            Format::Bin16 => NbyteReader::<2>::read(reader)?,
            Format::Bin32 => NbyteReader::<4>::read(reader)?,
            _ => return Err(Error::UnexpectedFormat),
        };
        let data = reader.read_slice(len).map_err(Error::Io)?;
        Ok(data)
    }
}

/// Owned `Vec<u8>` decoder for MessagePack bin8/16/32.
#[cfg(feature = "alloc")]
pub struct BinOwnedDecoder;

#[cfg(feature = "alloc")]
impl<'de> super::DecodeBorrowed<'de> for BinOwnedDecoder {
    type Value = alloc::vec::Vec<u8>;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> Result<<Self as DecodeBorrowed<'de>>::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        let val = ReferenceDecoder::decode_with_format(format, reader)?;
        Ok(val.as_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::Decode;
    #[test]
    fn decode_bin8() {
        let expect = r#"
MessagePack
"#
        .as_bytes();
        let len = u8::try_from(expect.len()).unwrap();
        let buf = [0xc4_u8]
            .into_iter()
            .chain(len.to_be_bytes())
            .chain(expect.iter().cloned())
            .collect::<Vec<_>>();

        let mut r = crate::io::SliceReader::new(&buf);
        let decoded = BinDecoder::decode(&mut r).unwrap();
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);
    }

    #[test]
    fn decode_bin16() {
        let expect = r#"
MessagePack is an object serialization specification like JSON.

MessagePack has two concepts: type system and formats.

Serialization is conversion from application objects into MessagePack formats via MessagePack type system.

Deserialization is conversion from MessagePack formats into application objects via MessagePack type system.
"#.as_bytes();
        let len = u16::try_from(expect.len()).unwrap();
        let buf = [0xc5_u8]
            .into_iter()
            .chain(len.to_be_bytes())
            .chain(expect.iter().cloned())
            .collect::<Vec<_>>();

        let mut r = crate::io::SliceReader::new(&buf);
        let decoded = BinDecoder::decode(&mut r).unwrap();
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);
    }

    #[test]
    fn decode_bin32() {
        let expect = include_str!("bin.rs").as_bytes();
        let len = u32::try_from(expect.len()).unwrap();
        let buf = [0xc6_u8]
            .into_iter()
            .chain(len.to_be_bytes())
            .chain(expect.iter().cloned())
            .collect::<Vec<_>>();

        let mut r = crate::io::SliceReader::new(&buf);
        let decoded = BinDecoder::decode(&mut r).unwrap();
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn decode_vec_u8_owned() {
        // bin8 with 3 bytes
        let buf = [0xc4, 0x03, 0x01, 0x02, 0x03];
        let mut r = crate::io::SliceReader::new(&buf);
        let v = <BinOwnedDecoder as Decode>::decode(&mut r).unwrap();
        assert_eq!(v, alloc::vec![1u8, 2, 3]);
        assert!(r.rest().is_empty());
    }
}
