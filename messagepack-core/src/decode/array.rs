//! Array decoding helpers.

use core::marker::PhantomData;

use super::{Decode, Error, NbyteReader, Result};
use crate::formats::Format;

/// Decode a MessagePack array of `V` into `Array` collecting iterator.
pub struct ArrayDecoder<Array, V>(PhantomData<(Array, V)>);

impl<'a, Array, V> Decode<'a> for ArrayDecoder<Array, V>
where
    V: Decode<'a>,
    Array: FromIterator<V::Value>,
{
    type Value = Array;

    fn decode(buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (format, buf) = Format::decode(buf)?;
        match format {
            Format::FixArray(_) | Format::Array16 | Format::Array32 => {
                Self::decode_with_format(format, buf)
            }
            _ => Err(Error::UnexpectedFormat),
        }
    }

    fn decode_with_format(format: Format, buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (len, buf) = match format {
            Format::FixArray(len) => (len.into(), buf),
            Format::Array16 => NbyteReader::<2>::read(buf)?,
            Format::Array32 => NbyteReader::<4>::read(buf)?,
            _ => return Err(Error::UnexpectedFormat),
        };

        let mut has_err = None;
        let mut buf_ptr = buf;
        let collector = core::iter::repeat_n((), len).map_while(|_| match V::decode(buf_ptr) {
            Ok((v, b)) => {
                buf_ptr = b;
                Some(v)
            }
            Err(e) => {
                has_err = Some(e);
                None
            }
        });
        let res = Array::from_iter(collector);

        if let Some(e) = has_err {
            Err(e)
        } else {
            Ok((res, buf_ptr))
        }
    }
}

macro_rules! tuple_decode_impls {
    ($($len:expr => ($($n:tt $name:ident)+))+ $(,)?) => {
        $(
            impl<'a, $($name),+> Decode<'a> for ($($name,)+)
            where
                $($name: Decode<'a>,)+
            {
                type Value = ($(<$name as Decode<'a>>::Value,)+);

                fn decode(buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
                    let (format, buf) = Format::decode(buf)?;
                    match format {
                        Format::FixArray(_) | Format::Array16 | Format::Array32 =>
                            Self::decode_with_format(format, buf),
                        _ => Err(Error::UnexpectedFormat),
                    }
                }

                fn decode_with_format(format: Format, buf: &'a [u8])
                    -> Result<(Self::Value, &'a [u8])>
                {
                    let (len, mut p) = match format {
                        Format::FixArray(len) => (len.into(), buf),
                        Format::Array16 => NbyteReader::<2>::read(buf)?,
                        Format::Array32 => NbyteReader::<4>::read(buf)?,
                        _ => return Err(Error::UnexpectedFormat),
                    };
                    if len != $len {
                        return Err(Error::InvalidData);
                    }

                    let value = (
                        $({
                            let (v, next) = <$name as Decode<'a>>::decode(p)?;
                            p = next;
                            v
                        },)+
                    );
                    Ok((value, p))
                }
            }
        )+
    };
}

tuple_decode_impls! {
    1  => (0 V0)
    2  => (0 V0 1 V1)
    3  => (0 V0 1 V1 2 V2)
    4  => (0 V0 1 V1 2 V2 3 V3)
    5  => (0 V0 1 V1 2 V2 3 V3 4 V4)
    6  => (0 V0 1 V1 2 V2 3 V3 4 V4 5 V5)
    7  => (0 V0 1 V1 2 V2 3 V3 4 V4 5 V5 6 V6)
    8  => (0 V0 1 V1 2 V2 3 V3 4 V4 5 V5 6 V6 7 V7)
    9  => (0 V0 1 V1 2 V2 3 V3 4 V4 5 V5 6 V6 7 V7 8 V8)
    10 => (0 V0 1 V1 2 V2 3 V3 4 V4 5 V5 6 V6 7 V7 8 V8 9 V9)
    11 => (0 V0 1 V1 2 V2 3 V3 4 V4 5 V5 6 V6 7 V7 8 V8 9 V9 10 V10)
    12 => (0 V0 1 V1 2 V2 3 V3 4 V4 5 V5 6 V6 7 V7 8 V8 9 V9 10 V10 11 V11)
    13 => (0 V0 1 V1 2 V2 3 V3 4 V4 5 V5 6 V6 7 V7 8 V8 9 V9 10 V10 11 V11 12 V12)
    14 => (0 V0 1 V1 2 V2 3 V3 4 V4 5 V5 6 V6 7 V7 8 V8 9 V9 10 V10 11 V11 12 V12 13 V13)
    15 => (0 V0 1 V1 2 V2 3 V3 4 V4 5 V5 6 V6 7 V7 8 V8 9 V9 10 V10 11 V11 12 V12 13 V13 14 V14)
    16 => (0 V0 1 V1 2 V2 3 V3 4 V4 5 V5 6 V6 7 V7 8 V8 9 V9 10 V10 11 V11 12 V12 13 V13 14 V14 15 V15)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_int_array() {
        let buf: &[u8] = &[0x95, 0x01, 0x02, 0x03, 0x04, 0x05];
        let (decoded, rest) = ArrayDecoder::<Vec<u8>, u8>::decode(buf).unwrap();

        let expect: &[u8] = &[1, 2, 3, 4, 5];
        assert_eq!(decoded, expect);
        assert_eq!(rest.len(), 0);
    }
}
