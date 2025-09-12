//! Map decoding helpers.

use core::marker::PhantomData;

use super::{Decode, Error, NbyteReader};
use crate::{formats::Format, io::IoRead};

/// Decode a MessagePack map of `K -> V` into `Map` collecting iterator.
pub struct MapDecoder<Map, K, V>(PhantomData<(Map, K, V)>);

#[allow(clippy::type_complexity)]
fn decode_kv<'de, R, K, V>(reader: &mut R) -> Result<(K::Value, V::Value), Error<R::Error>>
where
    R: IoRead<'de>,
    K: Decode<'de>,
    V: Decode<'de>,
{
    let k = K::decode(reader)?;
    let v = V::decode(reader)?;
    Ok((k, v))
}

impl<'de, Map, K, V> Decode<'de> for MapDecoder<Map, K, V>
where
    K: Decode<'de>,
    V: Decode<'de>,
    Map: FromIterator<(K::Value, V::Value)>,
{
    type Value = Map;

    fn decode_with_format<R>(format: Format, reader: &mut R) -> Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        let len = match format {
            Format::FixMap(len) => len.into(),
            Format::Map16 => NbyteReader::<2>::read(reader)?,
            Format::Map32 => NbyteReader::<4>::read(reader)?,
            _ => return Err(Error::UnexpectedFormat),
        };

        let mut err: Option<Error<R::Error>> = None;
        let iter = (0..len).map_while(|_| match decode_kv::<R, K, V>(reader) {
            Ok((k, v)) => Some((k, v)),
            Err(e) => {
                err = Some(e);
                None
            }
        });
        let res = Map::from_iter(iter);
        match err {
            Some(e) => Err(e),
            None => Ok(res),
        }
    }
}
