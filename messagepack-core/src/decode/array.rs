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

        let mut buf_ptr = buf;
        let out = (0..len)
            .map(|_| {
                let (v, next) = V::decode(buf_ptr)?;
                buf_ptr = next;
                Ok::<_, Error>(v)
            })
            .collect::<Result<Array>>()?;
        Ok((out, buf_ptr))
    }
}

impl<'a, const N: usize, V> Decode<'a> for [V; N]
where
    V: Decode<'a>,
{
    type Value = [V::Value; N];
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
        if len != N {
            return Err(Error::InvalidData);
        };

        let mut tmp: [Option<V::Value>; N] = core::array::from_fn(|_| None);
        let mut buf_ptr = buf;
        for i in 0..N {
            let (val, next) = V::decode(buf_ptr)?;
            tmp[i] = Some(val);
            buf_ptr = next
        }
        let out = core::array::from_fn(|i| tmp[i].take().expect("initialized"));
        Ok((out, buf_ptr))
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
    use rstest::rstest;

    #[rstest]
    #[case(&[0x92, 0x01, 0x02, 0x01], vec![1u8, 2], &[0x01])]
    #[case(&[0xdc, 0x00, 0x02, 0x2a, 0x2b], vec![42u8, 43], &[])]
    fn array_decode_success(
        #[case] buf: &[u8],
        #[case] expect: Vec<u8>,
        #[case] rest_expect: &[u8],
    ) {
        let (decoded, rest) = ArrayDecoder::<Vec<u8>, u8>::decode(buf).unwrap();
        assert_eq!(decoded, expect);
        assert_eq!(rest, rest_expect);
    }

    #[rstest]
    fn array_decoder_unexpected_format() {
        let buf = &[0x81, 0x01, 0x02]; // map(1)
        let err = ArrayDecoder::<Vec<u8>, u8>::decode(buf).unwrap_err();
        assert!(matches!(err, Error::UnexpectedFormat));
    }

    #[rstest]
    fn fixed_array_len0_success() {
        let buf = &[0x90]; // array(0)
        let (arr, rest) = <[u8; 0] as Decode>::decode(buf).unwrap();
        assert_eq!(arr, []);
        assert!(rest.is_empty());
    }

    #[rstest]
    fn fixed_array_len3_success() {
        let buf = &[0x93, 0x0a, 0x0b, 0x0c];
        let (arr, rest) = <[u8; 3] as Decode>::decode(buf).unwrap();
        assert_eq!(arr, [10u8, 11, 12]);
        assert!(rest.is_empty());
    }

    #[rstest]
    #[case(&[0x92, 0x01, 0x02])] // len=2
    #[case(&[0x94, 0x01, 0x02, 0x03, 0x04])] // len=4 
    fn fixed_array_len_mismatch(#[case] buf: &[u8]) {
        let err = <[u8; 3] as Decode>::decode(buf).unwrap_err();
        assert!(matches!(err, Error::InvalidData));
    }

    #[rstest]
    fn tuple1_success() {
        let buf = &[0x91, 0x2a]; // [42]
        let ((v0,), rest) = <(u8,) as Decode>::decode(buf).unwrap();
        assert_eq!(v0, 42);
        assert!(rest.is_empty());
    }

    #[rstest]
    #[case(&[0x92, 0x2a, 0x2b])] // fixarray
    #[case(&[0xdc, 0x00, 0x02, 0x2a, 0x2b])] // array16(2)
    fn tuple2_success(#[case] buf: &[u8]) {
        let ((a, b), rest) = <(u8, u8) as Decode>::decode(buf).unwrap();
        assert_eq!((a, b), (42, 43));
        assert!(rest.is_empty());
    }

    #[rstest]
    fn tuple3_success() {
        let buf = &[0x93, 0x01, 0x02, 0x03];
        let ((a, b, c), rest) = <(u8, u8, u8) as Decode>::decode(buf).unwrap();
        assert_eq!((a, b, c), (1, 2, 3));
        assert!(rest.is_empty());
    }

    #[rstest]
    #[case(&[0x92, 0x01, 0x02])] // len 2
    #[case(&[0xdc, 0x00, 0x04, 1, 2, 3, 4])] // len 4
    fn tuple_len_mismatch(#[case] buf: &[u8]) {
        let err = <(u8, u8, u8) as Decode>::decode(buf).unwrap_err();
        assert!(matches!(err, Error::InvalidData));
    }

    #[rstest]
    fn tuple_unexpected_format() {
        let buf = &[0x81, 0x01, 0x02]; // map(1)
        let err = <(u8,) as Decode>::decode(buf).unwrap_err();
        assert!(matches!(err, Error::UnexpectedFormat));
    }
}
