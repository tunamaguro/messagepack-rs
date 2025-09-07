//! Map decoding helpers.

use core::marker::PhantomData;

use super::{Decode, Error, NbyteReader, Result};
use crate::formats::Format;

/// Decode a MessagePack map of `K -> V` into `Map` collecting iterator.
pub struct MapDecoder<Map, K, V>(PhantomData<(Map, K, V)>);

fn decode_kv<'a, K, V>(buf: &'a [u8]) -> Result<(K::Value, V::Value, &'a [u8])>
where
    K: Decode<'a>,
    V: Decode<'a>,
{
    let (k, buf) = K::decode(buf)?;
    let (v, buf) = V::decode(buf)?;
    Ok((k, v, buf))
}

impl<'a, Map, K, V> Decode<'a> for MapDecoder<Map, K, V>
where
    K: Decode<'a>,
    V: Decode<'a>,
    Map: FromIterator<(K::Value, V::Value)>,
{
    type Value = Map;

    fn decode(buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (format, buf) = Format::decode(buf)?;
        match format {
            Format::FixMap(_) | Format::Map16 | Format::Map32 => {
                Self::decode_with_format(format, buf)
            }
            _ => Err(Error::UnexpectedFormat),
        }
    }

    fn decode_with_format(format: Format, buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (len, buf) = match format {
            Format::FixMap(len) => (len.into(), buf),
            Format::Map16 => NbyteReader::<2>::read(buf)?,
            Format::Map32 => NbyteReader::<4>::read(buf)?,
            _ => return Err(Error::UnexpectedFormat),
        };

        let mut has_err = None;
        let mut buf_ptr = buf;
        let collector =
            core::iter::repeat_n((), len).map_while(|_| match decode_kv::<K, V>(buf_ptr) {
                Ok((k, v, b)) => {
                    buf_ptr = b;
                    Some((k, v))
                }
                Err(e) => {
                    has_err = Some(e);
                    None
                }
            });
        let res = Map::from_iter(collector);

        if let Some(e) = has_err {
            Err(e)
        } else {
            Ok((res, buf_ptr))
        }
    }
}
