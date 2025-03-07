use super::{Decode, Error, Result};
use crate::formats::Format;

impl<'a> Decode<'a> for bool {
    type Value = Self;
    fn decode(buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (format, buf) = Format::decode(buf)?;
        Self::decode_with_format(format, buf)
    }
    fn decode_with_format(format: Format, buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let val = match format {
            Format::True => Ok(true),
            Format::False => Ok(false),
            _ => Err(Error::UnexpectedFormat),
        }?;

        Ok((val, buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_true() {
        let buf: &[u8] = &[0xc3];
        let (decoded, rest) = bool::decode(buf).unwrap();
        let expect = true;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0)
    }

    #[test]
    fn decode_false() {
        let buf: &[u8] = &[0xc2];
        let (decoded, rest) = bool::decode(buf).unwrap();
        let expect: bool = false;
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0)
    }
}
