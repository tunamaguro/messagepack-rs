use core::{borrow::Borrow, marker::PhantomData};

use super::{Encode, Error, Result};
use crate::formats::Format;

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
            _ => Err(Error::InvalidType),
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
                let mut it = Format::FixMap(cast).into_iter();
                for (to, byte) in buf.zip(&mut it) {
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
                let mut it = Format::Map16.into_iter().chain(cast.to_be_bytes());
                for (to, byte) in buf.zip(&mut it) {
                    *to = byte
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
                let mut it = Format::Map32.into_iter().chain(cast.to_be_bytes());
                for (to, byte) in buf.zip(&mut it) {
                    *to = byte
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            _ => Err(Error::InvalidType),
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

    #[test]
    fn encode_map() {
        let a = [("123", 123), ("456", 456)];
        let encoder = MapEncoder::new(a.iter());

        let expect: &[u8] = &[
            0x82, 0xa3, 0x31, 0x32, 0x33, 0x7b, 0xa3, 0x34, 0x35, 0x36, 0xcd, 0x01, 0xc8,
        ];

        let buf: &mut [u8] = &mut [0x0; 13];
        encoder.encode_to_iter_mut(&mut buf.iter_mut()).unwrap();

        assert_eq!(buf, expect)
    }
}
