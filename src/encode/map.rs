use core::{borrow::Borrow, marker::PhantomData, ops::Deref};

use super::{Encode, Error, Result};
use crate::formats::Format;

pub struct MapSliceEncoder<'data, B, K, V> {
    data: &'data [B],
    _phantom: PhantomData<(K, V)>,
}

impl<'data, B, K, V> MapSliceEncoder<'data, B, K, V> {
    pub fn new(data: &'data [B]) -> Self {
        Self {
            data,
            _phantom: Default::default(),
        }
    }
}

impl<'data, B, K, V> Deref for MapSliceEncoder<'data, B, K, V> {
    type Target = &'data [B];
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'data, B, K, V> Encode for MapSliceEncoder<'data, B, K, V>
where
    B: Borrow<(K, V)>,
    K: Encode,
    V: Encode,
{
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let self_len = self.data.len();
        let format_len = match self_len {
            0x00..=0xf => {
                let cast = self_len as u8;
                let it = Format::FixMap(cast).into_iter();
                buf.extend(it);
                Ok(1)
            }
            0x10..=0xffff => {
                let cast = self_len as u16;
                let it = Format::Map16.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(3)
            }
            0x10000..=0xffffffff => {
                let cast = self_len as u32;
                let it = Format::Map32.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(5)
            }
            _ => Err(Error::InvalidFormat),
        }?;

        let map_len = self
            .data
            .iter()
            .flat_map(|v| {
                let (k, v) = v.borrow();
                let k_len = k.encode(buf);
                let v_len = v.encode(buf);
                [k_len, v_len]
            })
            .try_fold(0, |acc, v| v.map(|n| acc + n))?;
        Ok(format_len + map_len)
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        let self_len = self.data.len();
        let format_len = match self_len {
            0x00..=0xf => {
                const SIZE: usize = 1;
                let cast = self_len as u8;
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
                let cast = self_len as u16;
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
                let cast = self_len as u32;
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
        }?;

        let map_len = self
            .data
            .iter()
            .flat_map(|v| {
                let (k, v) = v.borrow();
                let k_len = k.encode_to_iter_mut(buf);
                let v_len = v.encode_to_iter_mut(buf);
                [k_len, v_len]
            })
            .try_fold(0, |acc, v| v.map(|n| acc + n))?;
        Ok(format_len + map_len)
    }
}

pub struct MapEncoder<MapLike, B, K, V> {
    map: MapLike,
    _phantom: PhantomData<(B, K, V)>,
}

impl<MapLike, B, K, V> core::ops::Deref for MapEncoder<MapLike, B, K, V> {
    type Target = MapLike;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<MapLike, B, K, V> MapEncoder<MapLike, B, K, V>
where
    MapLike: Iterator<Item = B> + ExactSizeIterator + Clone,
    B: Borrow<(K, V)>,
    K: Encode,
    V: Encode,
{
    pub fn new(map: MapLike) -> Self {
        Self {
            map,
            _phantom: Default::default(),
        }
    }
}

impl<MapLike, B, K, V> Encode for MapEncoder<MapLike, B, K, V>
where
    MapLike: Iterator<Item = B> + ExactSizeIterator + Clone,
    B: Borrow<(K, V)>,
    K: Encode,
    V: Encode,
{
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let clone_map = self.map.clone();
        let self_len = clone_map.len();
        let format_len = match self_len {
            0x00..=0xf => {
                let cast = self_len as u8;
                let it = Format::FixMap(cast).into_iter();
                buf.extend(it);
                Ok(1)
            }
            0x10..=0xffff => {
                let cast = self_len as u16;
                let it = Format::Map16.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(3)
            }
            0x10000..=0xffffffff => {
                let cast = self_len as u32;
                let it = Format::Map32.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(5)
            }
            _ => Err(Error::InvalidFormat),
        }?;

        let map_len = clone_map
            .flat_map(|v| {
                let (k, v) = v.borrow();
                let k_len = k.encode(buf);
                let v_len = v.encode(buf);
                [k_len, v_len]
            })
            .try_fold(0, |acc, v| v.map(|n| acc + n))?;
        Ok(format_len + map_len)
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        let clone_map = self.map.clone();
        let self_len = clone_map.len();
        let format_len = match self_len {
            0x00..=0xf => {
                const SIZE: usize = 1;
                let cast = self_len as u8;
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
                let cast = self_len as u16;
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
                let cast = self_len as u32;
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
        }?;

        let map_len = clone_map
            .flat_map(|v| {
                let (k, v) = v.borrow();
                let k_len = k.encode_to_iter_mut(buf);
                let v_len = v.encode_to_iter_mut(buf);
                [k_len, v_len]
            })
            .try_fold(0, |acc, v| v.map(|n| acc + n))?;
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
    fn encode_iter_fix_array<Map, K, V, B, E>(#[case] value: Map, #[case] expected: E)
    where
        Map: IntoIterator<Item = B>,
        Map::IntoIter: ExactSizeIterator + Clone,
        K: Encode,
        V: Encode,
        B: Borrow<(K, V)>,
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
