use core::{cell::RefCell, marker::PhantomData, ops::Deref};

use super::{Encode, Error, Result};
use crate::formats::Format;

pub trait KVEncode {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>;
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize>;
}

impl<KV: KVEncode> KVEncode for &KV {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        KV::encode(self, buf)
    }

    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        KV::encode_to_iter_mut(self, buf)
    }
}

impl<K: Encode, V: Encode> KVEncode for (K, V) {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let (k, v) = self;
        let k_len = k.encode(buf)?;
        let v_len = v.encode(buf)?;
        Ok(k_len + v_len)
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        let (k, v) = self;
        let k_len = k.encode_to_iter_mut(buf)?;
        let v_len = v.encode_to_iter_mut(buf)?;
        Ok(k_len + v_len)
    }
}

pub struct MapFormatEncoder(pub usize);
impl MapFormatEncoder {
    pub fn new(size: usize) -> Self {
        Self(size)
    }
}

impl Encode for MapFormatEncoder {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        match self.0 {
            0x00..=0xf => {
                let cast = self.0 as u8;
                let it = Format::FixMap(cast).into_iter();
                buf.extend(it);
                Ok(1)
            }
            0x10..=0xffff => {
                let cast = self.0 as u16;
                let it = Format::Map16.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(3)
            }
            0x10000..=0xffffffff => {
                let cast = self.0 as u32;
                let it = Format::Map32.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(5)
            }
            _ => Err(Error::InvalidFormat),
        }
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        match self.0 {
            0x00..=0xf => {
                const SIZE: usize = 1;
                let cast = self.0 as u8;
                let it = &mut Format::FixMap(cast).into_iter();

                for (byte, to) in it.zip(buf.by_ref()) {
                    *to = byte
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0x10..=0xffff => {
                const SIZE: usize = 3;
                let cast = self.0 as u16;
                let it = &mut Format::Map16.into_iter().chain(cast.to_be_bytes());

                for (byte, to) in it.zip(buf.by_ref()) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0x10000..=0xffffffff => {
                const SIZE: usize = 5;
                let cast = self.0 as u32;
                let it = &mut Format::Map32.into_iter().chain(cast.to_be_bytes());
                for (byte, to) in it.zip(buf.by_ref()) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
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

impl<I, J, KV> Encode for MapDataEncoder<I, J, KV>
where
    J: Iterator<Item = KV>,
    KV: KVEncode,
{
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let map_len = self
            .data
            .borrow_mut()
            .by_ref()
            .map(|kv| kv.encode(buf))
            .try_fold(0, |acc, v| v.map(|n| acc + n))?;
        Ok(map_len)
    }

    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        let map_len = self
            .data
            .borrow_mut()
            .by_ref()
            .map(|kv| kv.encode_to_iter_mut(buf))
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

impl<KV> Encode for MapSliceEncoder<'_, KV>
where
    KV: KVEncode,
{
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let self_len = self.data.len();
        let format_len = MapFormatEncoder::new(self_len).encode(buf)?;
        let map_len = MapDataEncoder::new(self.data.iter()).encode(buf)?;

        Ok(format_len + map_len)
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        let self_len = self.data.len();
        let format_len = MapFormatEncoder::new(self_len).encode_to_iter_mut(buf)?;
        let map_len = MapDataEncoder::new(self.data.iter()).encode_to_iter_mut(buf)?;

        Ok(format_len + map_len)
    }
}

pub struct MapEncoder<I, J, KV> {
    map: RefCell<J>,
    _phantom: PhantomData<(I, J, KV)>,
}

impl<I, KV> MapEncoder<I, I::IntoIter, KV>
where
    I: IntoIterator<Item = KV>,
    KV: KVEncode,
{
    pub fn new(map: I) -> Self {
        Self {
            map: RefCell::new(map.into_iter()),
            _phantom: Default::default(),
        }
    }
}

impl<I, J, KV> Encode for MapEncoder<I, J, KV>
where
    J: Iterator<Item = KV> + ExactSizeIterator,
    KV: KVEncode,
{
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let self_len = self.map.borrow().len();
        let format_len = MapFormatEncoder::new(self_len).encode(buf)?;
        let map_len = MapDataEncoder::new(self.map.borrow_mut().by_ref()).encode(buf)?;

        Ok(format_len + map_len)
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        let self_len = self.map.borrow().len();
        let format_len = MapFormatEncoder::new(self_len).encode_to_iter_mut(buf)?;
        let map_len =
            MapDataEncoder::new(self.map.borrow_mut().by_ref()).encode_to_iter_mut(buf)?;

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
        K: Encode,
        V: Encode,
        Map: AsRef<[(K, V)]>,
        E: AsRef<[u8]> + Sized,
    {
        let expected = expected.as_ref();
        let encoder = MapSliceEncoder::new(value.as_ref());
        {
            let mut buf = vec![];
            let n = encoder.encode(&mut buf).unwrap();
            assert_eq!(buf, expected);
            assert_eq!(n, expected.len());
        }

        {
            let mut buf = vec![0xff; core::mem::size_of::<E>()];
            let n = encoder.encode_to_slice(buf.as_mut_slice()).unwrap();
            assert_eq!(&buf, expected);
            assert_eq!(n, expected.len());
        }
    }

    #[rstest]
    #[case([("123", EncodeMinimizeInt(123)), ("456", EncodeMinimizeInt(456))], [0x82, 0xa3, 0x31, 0x32, 0x33, 0x7b, 0xa3, 0x34, 0x35, 0x36, 0xcd, 0x01, 0xc8])]
    fn encode_iter_fix_array<I, KV, E>(#[case] value: I, #[case] expected: E)
    where
        I: IntoIterator<Item = KV>,
        I::IntoIter: ExactSizeIterator,
        KV: KVEncode,
        E: AsRef<[u8]> + Sized,
    {
        let expected = expected.as_ref();
        {
            let encoder = MapEncoder::new(value.into_iter());
            let mut buf = vec![];
            let n = encoder.encode(&mut buf).unwrap();
            assert_eq!(buf, expected);
            assert_eq!(n, expected.len());
        }
    }
}
