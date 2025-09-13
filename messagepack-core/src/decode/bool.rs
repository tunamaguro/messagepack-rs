use super::{Decode, Error};
use crate::{formats::Format, io::IoRead};

impl<'de, 'a> Decode<'de, 'a> for bool {
    type Value = Self;

    fn decode_with_format<R>(
        format: Format,
        _reader: &'a mut R,
    ) -> core::result::Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        let val = match format {
            Format::True => Ok(true),
            Format::False => Ok(false),
            _ => Err(Error::UnexpectedFormat),
        }?;

        Ok(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_true() {
        let buf: &[u8] = &[0xc3];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = bool::decode(&mut r).unwrap();
        let expect = true;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0)
    }

    #[test]
    fn decode_false() {
        let buf: &[u8] = &[0xc2];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = bool::decode(&mut r).unwrap();
        let expect: bool = false;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0)
    }
}
