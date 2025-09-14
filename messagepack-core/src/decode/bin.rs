//! Binary (bin8/16/32) decoding helpers.

use super::{Error, NbyteReader};
use crate::{decode::DecodeBorrowed, formats::Format, io::IoRead};

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
        let len = match format {
            Format::Bin8 => NbyteReader::<1>::read(reader)?,
            Format::Bin16 => NbyteReader::<2>::read(reader)?,
            Format::Bin32 => NbyteReader::<4>::read(reader)?,
            _ => return Err(Error::UnexpectedFormat),
        };
        let data = reader.read_slice(len).map_err(Error::Io)?;
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
}
