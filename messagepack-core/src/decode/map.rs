//! Map decoding helpers.

use core::marker::PhantomData;

use super::{DecodeBorrowed, Error, NbyteReader};
use crate::{formats::Format, io::IoRead};

/// Decode a MessagePack map of `K -> V` into `Map` collecting iterator.
pub struct MapDecoder<Map, K, V>(PhantomData<(Map, K, V)>);

#[allow(clippy::type_complexity)]
fn decode_kv<'de, R, K, V>(reader: &mut R) -> Result<(K::Value, V::Value), Error<R::Error>>
where
    R: IoRead<'de>,
    K: DecodeBorrowed<'de>,
    V: DecodeBorrowed<'de>,
{
    let k = K::decode_borrowed(reader)?;
    let v = V::decode_borrowed(reader)?;
    Ok((k, v))
}

impl<'de, Map, K, V> DecodeBorrowed<'de> for MapDecoder<Map, K, V>
where
    K: DecodeBorrowed<'de>,
    V: DecodeBorrowed<'de>,
    Map: FromIterator<(K::Value, V::Value)>,
{
    type Value = Map;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> Result<Self::Value, Error<R::Error>>
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

#[cfg(feature = "alloc")]
impl<'de, K, V> DecodeBorrowed<'de> for alloc::collections::BTreeMap<K, V>
where
    K: DecodeBorrowed<'de>,
    V: DecodeBorrowed<'de>,
    K::Value: Ord,
{
    type Value = alloc::collections::BTreeMap<K::Value, V::Value>;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        MapDecoder::<Self::Value, K, V>::decode_borrowed_with_format(format, reader)
    }
}

#[cfg(feature = "std")]
impl<'de, K, V> DecodeBorrowed<'de> for std::collections::HashMap<K, V>
where
    K: DecodeBorrowed<'de>,
    V: DecodeBorrowed<'de>,
    K::Value: Eq + core::hash::Hash,
{
    type Value = std::collections::HashMap<K::Value, V::Value>;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        MapDecoder::<Self::Value, K, V>::decode_borrowed_with_format(format, reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::Decode;
    use rstest::rstest;

    #[rstest]
    #[case(&[0x82, 0x01, 0x0a, 0x02, 0x14], vec![(1u8, 10u8), (2, 20)], &[])]
    #[case(&[0xde, 0x00, 0x02, 0x01, 0x0a, 0x02, 0x14], vec![(1u8, 10u8), (2, 20)], &[])]
    fn map_decode_success(
        #[case] buf: &[u8],
        #[case] expect: Vec<(u8, u8)>,
        #[case] rest_expect: &[u8],
    ) {
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = MapDecoder::<Vec<(u8, u8)>, u8, u8>::decode(&mut r).unwrap();
        assert_eq!(decoded, expect);
        assert_eq!(r.rest(), rest_expect);
    }

    #[test]
    fn map_decoder_unexpected_format() {
        // array(1) where a map is expected
        let buf = &[0x91, 0x00];
        let mut r = crate::io::SliceReader::new(buf);
        let err = MapDecoder::<Vec<(u8, u8)>, u8, u8>::decode(&mut r).unwrap_err();
        assert!(matches!(err, Error::UnexpectedFormat));
    }

    #[test]
    fn map_decode_eof_on_key() {
        // map(1) but missing key/value bytes
        let buf = &[0x81];
        let mut r = crate::io::SliceReader::new(buf);
        let err = MapDecoder::<Vec<(u8, u8)>, u8, u8>::decode(&mut r).unwrap_err();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn map_decode_key_unexpected_format() {
        // map(1) with a string key (invalid for u8 key)
        let buf = &[0x81, 0xa1, b'a', 0x02];
        let mut r = crate::io::SliceReader::new(buf);
        let err = MapDecoder::<Vec<(u8, u8)>, u8, u8>::decode(&mut r).unwrap_err();
        assert!(matches!(err, Error::UnexpectedFormat));
    }

    #[test]
    fn map_decode_value_error_after_first_pair() {
        // map(2): first pair ok (1->1), second pair truncated (key present, value missing)
        let buf = &[0x82, 0x01, 0x01, 0x02];
        let mut r = crate::io::SliceReader::new(buf);
        let err = MapDecoder::<Vec<(u8, u8)>, u8, u8>::decode(&mut r).unwrap_err();
        // read_slice should fail while decoding second value
        assert!(matches!(err, Error::Io(_)));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn btreemap_decode_success() {
        // {1:10, 2:20}
        let buf = &[0x82, 0x01, 0x0a, 0x02, 0x14];
        let mut r = crate::io::SliceReader::new(buf);
        let m = <alloc::collections::BTreeMap<u8, u8> as Decode>::decode(&mut r).unwrap();
        assert_eq!(m.len(), 2);
        assert_eq!(m.get(&1), Some(&10));
        assert_eq!(m.get(&2), Some(&20));
        assert!(r.rest().is_empty());
    }

    #[cfg(feature = "std")]
    #[test]
    fn hashmap_decode_success() {
        // {1: true, 3: false}
        let buf = &[0x82, 0x01, 0xc3, 0x03, 0xc2];
        let mut r = crate::io::SliceReader::new(buf);
        let m = <std::collections::HashMap<u8, bool> as Decode>::decode(&mut r).unwrap();
        assert_eq!(m.len(), 2);
        assert_eq!(m.get(&1), Some(&true));
        assert_eq!(m.get(&3), Some(&false));
        assert!(r.rest().is_empty());
    }
}
