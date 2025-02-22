use super::{Decode, Error, NbyteReader, Result};
use crate::formats::Format;

impl<'a> Decode<'a> for &'a str {
    type Value = &'a str;

    fn decode(buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (format, buf) = Format::decode(buf)?;
        match format {
            Format::FixStr(_) | Format::Str8 | Format::Str16 | Format::Str32 => {
                Self::decode_with_format(format, buf)
            }
            _ => Err(Error::UnexpectedFormat),
        }
    }

    fn decode_with_format(format: Format, buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (len, buf) = match format {
            Format::FixStr(n) => (n.into(), buf),
            Format::Str8 => NbyteReader::<1>::read(buf)?,
            _ => todo!(),
        };
        let (data, rest) = buf.split_at_checked(len).ok_or(Error::EofData)?;
        let s = core::str::from_utf8(data).map_err(|_| Error::InvalidData)?;
        Ok((s, rest))
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

        let (decoded, rest) = <&str as Decode>::decode(buf).unwrap();
        let expect = "Hello World";
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);
    }

    #[test]
    fn decode_invalid_str() {
        let buf: &[u8] = &[0xa2, 0xc3, 0x28];
        let err = <&str as Decode>::decode(buf).unwrap_err();
        assert_eq!(err, Error::InvalidData);

        let buf: &[u8] = &[0xa1, 0x80];
        let err = <&str as Decode>::decode(buf).unwrap_err();
        assert_eq!(err, Error::InvalidData);
    }
}
