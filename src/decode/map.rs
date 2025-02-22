use core::marker::PhantomData;

use super::{Decode, Error, NbyteReader, Result};
use crate::formats::Format;

pub struct MapDecoder<Map, K, V>(PhantomData<(Map, K, V)>);

fn decode_kv<K, V, I, B>(buf: &mut I) -> Result<(K::Value, V::Value)>
where
    K: Decode,
    V: Decode,
    I: Iterator<Item = B>,
    B: core::borrow::Borrow<u8>,
{
    let k = K::decode(buf)?;
    let v = V::decode(buf)?;
    Ok((k, v))
}

impl<Map, K, V> Decode for MapDecoder<Map, K, V>
where
    K: Decode,
    V: Decode,
    Map: FromIterator<(K::Value, V::Value)>,
{
    type Value = Map;

    fn decode<I, B>(buf: &mut I) -> Result<Self::Value>
    where
        I: Iterator<Item = B>,
        B: core::borrow::Borrow<u8>,
    {
        let format = Format::decode(buf)?;
        match format {
            Format::FixMap(_) | Format::Map16 | Format::Map32 => {
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
            Format::FixMap(len) => len.into(),
            Format::Map16 => NbyteReader::<2>::read(buf)?,
            Format::Map32 => NbyteReader::<4>::read(buf)?,
            _ => return Err(Error::UnexpectedFormat),
        };

        let mut has_err = None;
        let collector =
            core::iter::repeat_n((), len).map_while(|_| match decode_kv::<K, V, I, B>(buf) {
                Ok(v) => Some(v),
                Err(e) => {
                    has_err = Some(e);
                    None
                }
            });
        let res = Map::from_iter(collector);

        if let Some(e) = has_err {
            Err(e)
        } else {
            Ok(res)
        }
    }
}
