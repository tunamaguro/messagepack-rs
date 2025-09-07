//! Binary (bin8/16/32) decoding helpers.

use super::{Decode, Error, NbyteReader, Result};
use crate::formats::Format;

/// Decode a MessagePack binary blob and return a borrowed byte slice.
pub struct BinDecoder;

impl<'a> Decode<'a> for BinDecoder {
    type Value = &'a [u8];
    fn decode(buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (format, buf) = Format::decode(buf)?;
        match format {
            Format::Bin8 | Format::Bin16 | Format::Bin32 => Self::decode_with_format(format, buf),
            _ => Err(Error::UnexpectedFormat),
        }
    }
    fn decode_with_format(format: Format, buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (len, buf) = match format {
            Format::Bin8 => NbyteReader::<1>::read(buf)?,
            Format::Bin16 => NbyteReader::<2>::read(buf)?,
            Format::Bin32 => NbyteReader::<4>::read(buf)?,
            _ => return Err(Error::UnexpectedFormat),
        };
        let (data, rest) = buf.split_at_checked(len).ok_or(Error::EofData)?;
        Ok((data, rest))
    }
}

impl<'a> Decode<'a> for &'a [u8] {
    type Value = &'a [u8];
    fn decode(buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        BinDecoder::decode(buf)
    }
    fn decode_with_format(format: Format, buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        BinDecoder::decode_with_format(format, buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let (decoded, rest) = BinDecoder::decode(&buf).unwrap();
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);
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

        let (decoded, rest) = BinDecoder::decode(&buf).unwrap();
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);
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

        let (decoded, rest) = BinDecoder::decode(&buf).unwrap();
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);
    }
}
