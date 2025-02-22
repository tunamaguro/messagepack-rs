use core::marker::PhantomData;

use super::{Decode, Error, NbyteReader, Result};
use crate::formats::Format;

pub struct ArrayDecoder<Array, V>(PhantomData<(Array, V)>);

impl<Array, V> Decode for ArrayDecoder<Array, V>
where
    Array: FromIterator<V::Value>,
    V: Decode,
{
    type Value = Array;

    fn decode<I, B>(buf: &mut I) -> Result<Self::Value>
    where
        I: Iterator<Item = B>,
        B: core::borrow::Borrow<u8>,
    {
        let format = Format::decode(buf)?;
        match format {
            Format::FixArray(_) | Format::Array16 | Format::Array32 => {
                Self::decode_with_format(format, buf)
            }
            _ => Err(Error::UnexpectedFormat),
        }
    }

    fn decode_with_format<I, B>(format: Format, buf: &mut I) -> Result<Self::Value>
    where
        I: Iterator<Item = B>,
        B: core::borrow::Borrow<u8>,
    {
        let len = match format {
            Format::FixArray(len) => len.into(),
            Format::Array16 => NbyteReader::<2>::read(buf)?,
            Format::Array32 => NbyteReader::<4>::read(buf)?,
            _ => return Err(Error::UnexpectedFormat),
        };

        let mut has_err = None;
        let collector = core::iter::repeat_n((), len).map_while(|_| match V::decode(buf) {
            Ok(v) => Some(v),
            Err(e) => {
                has_err = Some(e);
                None
            }
        });
        let res = Array::from_iter(collector);

        if let Some(e) = has_err {
            Err(e)
        } else {
            Ok(res)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_int_array() {
        let buf: &[u8] = &[0x95, 0x01, 0x02, 0x03, 0x04, 0x05];
        let decoded: Vec<u8> = ArrayDecoder::<Vec<u8>, u8>::decode(&mut buf.iter()).unwrap();

        let expect: &[u8] = &[1, 2, 3, 4, 5];
        assert_eq!(decoded, expect)
    }
}
