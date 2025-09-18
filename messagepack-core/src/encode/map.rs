//! Map encoders.

use core::{cell::RefCell, marker::PhantomData, ops::Deref};

use super::{Encode, Error, Result};
use crate::{formats::Format, io::IoWrite};

/// A key-value encoder that writes a single `key, value` pair.
pub trait KVEncode<W>
where
    W: IoWrite,
{
    /// Encode this key‑value pair to the writer and return the number of bytes written.
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error>;
}

impl<W: IoWrite, KV: KVEncode<W>> KVEncode<W> for &KV {
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
        KV::encode(self, writer)
    }
}

impl<W: IoWrite, K: Encode<W>, V: Encode<W>> KVEncode<W> for (K, V) {
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
        let (k, v) = self;
        let k_len = k.encode(writer)?;
        let v_len = v.encode(writer)?;
        Ok(k_len + v_len)
    }
}

/// Encode only the map header for a map of a given length.
pub struct MapFormatEncoder(pub usize);
impl MapFormatEncoder {
    /// Construct from the number of pairs contained in the map.
    pub fn new(size: usize) -> Self {
        Self(size)
    }
}

impl<W: IoWrite> Encode<W> for MapFormatEncoder {
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
        match self.0 {
            0x00..=0xf => {
                let cast = self.0 as u8;
                writer.write(&[Format::FixMap(cast).as_byte()])?;

                Ok(1)
            }
            0x10..=0xffff => {
                let cast = (self.0 as u16).to_be_bytes();
                writer.write(&[Format::Map16.as_byte(), cast[0], cast[1]])?;

                Ok(3)
            }
            0x10000..=0xffffffff => {
                let cast = (self.0 as u32).to_be_bytes();
                writer.write(&[Format::Map32.as_byte(), cast[0], cast[1], cast[2], cast[3]])?;

                Ok(5)
            }
            _ => Err(Error::InvalidFormat),
        }
    }
}

/// Encode a stream of key-value pairs from an iterator.
pub struct MapDataEncoder<I, J, KV> {
    data: RefCell<J>,
    _phantom: PhantomData<(I, J, KV)>,
}

impl<I, KV> MapDataEncoder<I, I::IntoIter, KV>
where
    I: IntoIterator<Item = KV>,
{
    /// Construct from any iterable of key-value pairs.
    pub fn new(data: I) -> Self {
        Self {
            data: RefCell::new(data.into_iter()),
            _phantom: Default::default(),
        }
    }
}

impl<W, I, J, KV> Encode<W> for MapDataEncoder<I, J, KV>
where
    W: IoWrite,
    J: Iterator<Item = KV>,
    KV: KVEncode<W>,
{
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
        let map_len = self
            .data
            .borrow_mut()
            .by_ref()
            .map(|kv| kv.encode(writer))
            .try_fold(0, |acc, v| v.map(|n| acc + n))?;
        Ok(map_len)
    }
}

fn encode_iter<W, I>(writer: &mut W, len: usize, it: I) -> Result<usize, W::Error>
where
    W: IoWrite,
    I: Iterator,
    I::Item: KVEncode<W>,
{
    let format_len = MapFormatEncoder::new(len).encode(writer)?;
    let data_len = it
        .map(|kv| kv.encode(writer))
        .try_fold(0, |acc, v| v.map(|n| acc + n))?;
    Ok(format_len + data_len)
}

/// Encode a slice of key-value pairs.
pub struct MapSliceEncoder<'data, KV> {
    data: &'data [KV],
    _phantom: PhantomData<KV>,
}

impl<'data, KV> MapSliceEncoder<'data, KV> {
    /// Construct from a slice of key-value pairs.
    pub fn new(data: &'data [KV]) -> Self {
        Self {
            data,
            _phantom: Default::default(),
        }
    }
}

impl<'data, KV> Deref for MapSliceEncoder<'data, KV> {
    type Target = &'data [KV];
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<W, KV> Encode<W> for MapSliceEncoder<'_, KV>
where
    W: IoWrite,
    KV: KVEncode<W>,
{
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
        encode_iter(writer, self.data.len(), self.data.iter())
    }
}

#[cfg(feature = "alloc")]
impl<W, K, V> Encode<W> for alloc::collections::BTreeMap<K, V>
where
    W: IoWrite,
    K: Encode<W> + Ord,
    V: Encode<W>,
{
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        encode_iter(writer, self.len(), self.iter())
    }
}

#[cfg(feature = "std")]
impl<W, K, V, S> Encode<W> for std::collections::HashMap<K, V, S>
where
    W: IoWrite,
    K: Encode<W> + Eq + core::hash::Hash,
    V: Encode<W>,
    S: std::hash::BuildHasher,
{
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        encode_iter(writer, self.len(), self.iter())
    }
}

/// Encode a map from an owned iterator, writing items lazily.
pub struct MapEncoder<W, I, J, KV> {
    map: RefCell<J>,
    _phantom: PhantomData<(W, I, J, KV)>,
}

impl<W, I, KV> MapEncoder<W, I, I::IntoIter, KV>
where
    W: IoWrite,
    I: IntoIterator<Item = KV>,
    KV: KVEncode<W>,
{
    /// Construct from any iterable of key-value pairs.
    pub fn new(map: I) -> Self {
        Self {
            map: RefCell::new(map.into_iter()),
            _phantom: Default::default(),
        }
    }
}

impl<W, I, J, KV> Encode<W> for MapEncoder<W, I, J, KV>
where
    W: IoWrite,
    J: Iterator<Item = KV> + ExactSizeIterator,
    KV: KVEncode<W>,
{
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
        let self_len = self.map.borrow().len();
        let format_len = MapFormatEncoder::new(self_len).encode(writer)?;
        let map_len = MapDataEncoder::new(self.map.borrow_mut().by_ref()).encode(writer)?;

        Ok(format_len + map_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::int::EncodeMinimizeInt;
    use rstest::rstest;

    #[rstest]
    #[case([("123", EncodeMinimizeInt(123)), ("456", EncodeMinimizeInt(456))], [0x82, 0xa3, 0x31, 0x32, 0x33, 0x7b, 0xa3, 0x34, 0x35, 0x36, 0xcd, 0x01, 0xc8])]
    fn encode_slice_fix_array<K, V, Map, E>(#[case] value: Map, #[case] expected: E)
    where
        K: Encode<Vec<u8>>,
        V: Encode<Vec<u8>>,
        Map: AsRef<[(K, V)]>,
        E: AsRef<[u8]> + Sized,
    {
        let expected = expected.as_ref();
        let encoder = MapSliceEncoder::new(value.as_ref());

        let mut buf = vec![];
        let n = encoder.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }

    #[rstest]
    #[case([("123", EncodeMinimizeInt(123)), ("456", EncodeMinimizeInt(456))], [0x82, 0xa3, 0x31, 0x32, 0x33, 0x7b, 0xa3, 0x34, 0x35, 0x36, 0xcd, 0x01, 0xc8])]
    fn encode_iter_fix_array<I, KV, E>(#[case] value: I, #[case] expected: E)
    where
        I: IntoIterator<Item = KV>,
        I::IntoIter: ExactSizeIterator,
        KV: KVEncode<Vec<u8>>,
        E: AsRef<[u8]> + Sized,
    {
        let expected = expected.as_ref();

        let encoder = MapEncoder::new(value.into_iter());
        let mut buf = vec![];
        let n = encoder.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn encode_btreemap_sorted() {
        let mut m = alloc::collections::BTreeMap::new();
        m.insert(2u8, 20u8);
        m.insert(1u8, 10u8);

        let mut buf = alloc::vec::Vec::new();
        let n = m.encode(&mut buf).unwrap();

        // Expect keys encoded in sorted order: 1, 2
        assert_eq!(
            &buf[..n],
            &[0x82, 0x01, 0x0a, 0x02, 0x14] // fixmap(2) {1:10, 2:20}
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn encode_hashmap_roundtrip() {
        use crate::decode::Decode;

        let mut m = std::collections::HashMap::<u8, bool>::new();
        m.insert(1, true);
        m.insert(3, false);

        let mut buf = Vec::new();
        let _ = m.encode(&mut buf).unwrap();

        // Roundtrip decode to HashMap and check contents regardless of order
        let mut r = crate::io::SliceReader::new(&buf);
        let back = <std::collections::HashMap<u8, bool> as Decode>::decode(&mut r).unwrap();
        assert_eq!(back.len(), 2);
        assert_eq!(back.get(&1), Some(&true));
        assert_eq!(back.get(&3), Some(&false));
    }
}
