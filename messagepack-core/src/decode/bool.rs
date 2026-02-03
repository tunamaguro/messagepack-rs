use super::{DecodeBorrowed, Error};
use crate::{formats::Format, io::IoRead};

impl<'de> DecodeBorrowed<'de> for bool {
    type Value = Self;

    fn decode_borrowed_with_format<R>(
        format: Format,
        _reader: &mut R,
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

impl<'de> DecodeBorrowed<'de> for core::sync::atomic::AtomicBool {
    type Value = Self;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        let val = bool::decode_borrowed_with_format(format, reader)?;
        Ok(Self::new(val))
    }
}

#[cfg(test)]
mod tests {
    use crate::decode::Decode;

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
