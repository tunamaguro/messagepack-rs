//! Array decoding helpers.

use core::marker::PhantomData;

use super::{Decode, Error, NbyteReader};
use crate::{formats::Format, io::IoRead};

/// Decode a MessagePack array of `V` into `Array` collecting iterator.
pub struct ArrayDecoder<Array, V>(PhantomData<(Array, V)>);

impl<'de, Array, V> Decode<'de> for ArrayDecoder<Array, V>
where
    V: Decode<'de>,
    Array: FromIterator<V::Value>,
{
    type Value = Array;

    fn decode_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        let len = match format {
            Format::FixArray(len) => len.into(),
            Format::Array16 => NbyteReader::<2>::read(reader)?,
            Format::Array32 => NbyteReader::<4>::read(reader)?,
            _ => return Err(Error::UnexpectedFormat),
        };

        let out = (0..len)
            .map(|_| V::decode(reader))
            .collect::<core::result::Result<Array, Error<R::Error>>>()?;
        Ok(out)
    }
}

impl<'de, const N: usize, V> Decode<'de> for [V; N]
where
    V: Decode<'de>,
{
    type Value = [V::Value; N];

    fn decode_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        let len = match format {
            Format::FixArray(len) => len.into(),
            Format::Array16 => NbyteReader::<2>::read(reader)?,
            Format::Array32 => NbyteReader::<4>::read(reader)?,
            _ => return Err(Error::UnexpectedFormat),
        };
        if len != N {
            return Err(Error::InvalidData);
        };

        let mut tmp: [Option<V::Value>; N] = core::array::from_fn(|_| None);
        for item in tmp.iter_mut() {
            *item = Some(V::decode(reader)?);
        }
        let out = core::array::from_fn(|i| tmp[i].take().expect("initialized"));
        Ok(out)
    }
}

macro_rules! tuple_decode_impls {
    ($($len:expr => ($($name:ident)+))+ $(,)?) => {
        $(
            impl<'de, $($name),+> Decode<'de> for ($($name,)+)
            where
                $($name: Decode<'de>,)+
            {
                type Value = ($(<$name as Decode<'de>>::Value,)+);

                fn decode_with_format<R>(format: Format, reader: &mut R) -> core::result::Result<Self::Value, Error<R::Error>>
                where
                    R: IoRead<'de>,
                {
                    let len = match format {
                        Format::FixArray(len) => len.into(),
                        Format::Array16 => NbyteReader::<2>::read(reader)?,
                        Format::Array32 => NbyteReader::<4>::read(reader)?,
                        _ => return Err(Error::UnexpectedFormat),
                    };
                    if len != $len {
                        return Err(Error::InvalidData);
                    }

                    let value = (
                        $({
                            let v = <$name as Decode<'de>>::decode(reader)?;
                            v
                        },)+
                    );
                    Ok(value)
                }
            }
        )+
    };
}

tuple_decode_impls! {
    1  => (V0)
    2  => (V0 V1)
    3  => (V0 V1 V2)
    4  => (V0 V1 V2 V3)
    5  => (V0 V1 V2 V3 V4)
    6  => (V0 V1 V2 V3 V4 V5)
    7  => (V0 V1 V2 V3 V4 V5 V6)
    8  => (V0 V1 V2 V3 V4 V5 V6 V7)
    9  => (V0 V1 V2 V3 V4 V5 V6 V7 V8)
    10 => (V0 V1 V2 V3 V4 V5 V6 V7 V8 V9)
    11 => (V0 V1 V2 V3 V4 V5 V6 V7 V8 V9 V10)
    12 => (V0 V1 V2 V3 V4 V5 V6 V7 V8 V9 V10 V11)
    13 => (V0 V1 V2 V3 V4 V5 V6 V7 V8 V9 V10 V11 V12)
    14 => (V0 V1 V2 V3 V4 V5 V6 V7 V8 V9 V10 V11 V12 V13)
    15 => (V0 V1 V2 V3 V4 V5 V6 V7 V8 V9 V10 V11 V12 V13 V14)
    16 => (V0 V1 V2 V3 V4 V5 V6 V7 V8 V9 V10 V11 V12 V13 V14 V15)
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
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = ArrayDecoder::<Vec<u8>, u8>::decode(&mut r).unwrap();
        assert_eq!(decoded, expect);
        assert_eq!(r.rest(), rest_expect);
    }

    #[rstest]
    fn array_decoder_unexpected_format() {
        let buf = &[0x81, 0x01, 0x02]; // map(1)
        let mut r = crate::io::SliceReader::new(buf);
        let err = ArrayDecoder::<Vec<u8>, u8>::decode(&mut r).unwrap_err();
        assert!(matches!(err, Error::UnexpectedFormat));
    }

    #[rstest]
    fn fixed_array_len0_success() {
        let buf = &[0x90]; // array(0)
        let mut r = crate::io::SliceReader::new(buf);
        let arr = <[u8; 0] as Decode>::decode(&mut r).unwrap();
        assert_eq!(arr, []);
        assert!(r.rest().is_empty());
    }

    #[rstest]
    fn fixed_array_len3_success() {
        let buf = &[0x93, 0x0a, 0x0b, 0x0c];
        let mut r = crate::io::SliceReader::new(buf);
        let arr = <[u8; 3] as Decode>::decode(&mut r).unwrap();
        assert_eq!(arr, [10u8, 11, 12]);
        assert!(r.rest().is_empty());
    }

    #[rstest]
    #[case(&[0x92, 0x01, 0x02])] // len=2
    #[case(&[0x94, 0x01, 0x02, 0x03, 0x04])] // len=4 
    fn fixed_array_len_mismatch(#[case] buf: &[u8]) {
        let mut r = crate::io::SliceReader::new(buf);
        let err = <[u8; 3] as Decode>::decode(&mut r).unwrap_err();
        assert!(matches!(err, Error::InvalidData));
    }

    #[rstest]
    fn tuple1_success() {
        let buf = &[0x91, 0x2a]; // [42]
        let mut r = crate::io::SliceReader::new(buf);
        let (v0,) = <(u8,) as Decode>::decode(&mut r).unwrap();
        assert_eq!(v0, 42);
        assert!(r.rest().is_empty());
    }

    #[rstest]
    #[case(&[0x92, 0x2a, 0x2b])] // fixarray
    #[case(&[0xdc, 0x00, 0x02, 0x2a, 0x2b])] // array16(2)
    fn tuple2_success(#[case] buf: &[u8]) {
        let mut r = crate::io::SliceReader::new(buf);
        let (a, b) = <(u8, u8) as Decode>::decode(&mut r).unwrap();
        assert_eq!((a, b), (42, 43));
        assert!(r.rest().is_empty());
    }

    #[rstest]
    fn tuple3_success() {
        let buf = &[0x93, 0x01, 0x02, 0x03];
        let mut r = crate::io::SliceReader::new(buf);
        let (a, b, c) = <(u8, u8, u8) as Decode>::decode(&mut r).unwrap();
        assert_eq!((a, b, c), (1, 2, 3));
        assert!(r.rest().is_empty());
    }

    #[rstest]
    #[case(&[0x92, 0x01, 0x02])] // len 2
    #[case(&[0xdc, 0x00, 0x04, 1, 2, 3, 4])] // len 4
    fn tuple_len_mismatch(#[case] buf: &[u8]) {
        let mut r = crate::io::SliceReader::new(buf);
        let err = <(u8, u8, u8) as Decode>::decode(&mut r).unwrap_err();
        assert!(matches!(err, Error::InvalidData));
    }

    #[rstest]
    fn tuple_unexpected_format() {
        let buf = &[0x81, 0x01, 0x02]; // map(1)
        let mut r = crate::io::SliceReader::new(buf);
        let err = <(u8,) as Decode>::decode(&mut r).unwrap_err();
        assert!(matches!(err, Error::UnexpectedFormat));
    }
}
