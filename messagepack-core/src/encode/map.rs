use core::{cell::RefCell, marker::PhantomData, ops::Deref};

use super::{Encode, Error, Result};
use crate::{formats::Format, io::IoWrite};

pub trait KVEncode<W>
where
    W: IoWrite,
{
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

pub struct MapFormatEncoder(pub usize);
impl MapFormatEncoder {
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

pub struct MapDataEncoder<I, J, KV> {
    data: RefCell<J>,
    _phantom: PhantomData<(I, J, KV)>,
}

impl<I, KV> MapDataEncoder<I, I::IntoIter, KV>
where
    I: IntoIterator<Item = KV>,
{
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

pub struct MapSliceEncoder<'data, KV> {
    data: &'data [KV],
    _phantom: PhantomData<KV>,
}

impl<'data, KV> MapSliceEncoder<'data, KV> {
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
        let self_len = self.data.len();
        let format_len = MapFormatEncoder::new(self_len).encode(writer)?;
        let map_len = MapDataEncoder::new(self.data.iter()).encode(writer)?;

        Ok(format_len + map_len)
    }
}

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
}
