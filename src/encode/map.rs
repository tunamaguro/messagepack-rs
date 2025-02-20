use core::{borrow::Borrow, iter, marker::PhantomData};

use super::{Encode, Error, Result};
use crate::formats;

pub struct MapEncoder<'b, MapLike, B, K, V> {
    map: MapLike,
    _phantom: PhantomData<(B, K, V, &'b ())>,
}

impl<MapLike, B, K, V> core::ops::Deref for MapEncoder<'_, MapLike, B, K, V> {
    type Target = MapLike;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<'b, MapLike, B, K, V> MapEncoder<'b, MapLike, B, K, V>
where
    MapLike: Iterator<Item = &'b B> + ExactSizeIterator + Clone,
    B: Borrow<(K, V)> + 'b,
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

impl<MapLike, B, K, V> Encode for MapEncoder<'_, MapLike, B, K, V>
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
                let it = iter::once(cast | formats::FIX_MAP);
                buf.extend(it);
                Ok(1)
            }
            0xf0..=0xff => {
                let cast = self_len as u16;
                let it = iter::once(formats::MAP16).chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(3)
            }
            0xffff..=0xffff => {
                let cast = self_len as u32;
                let it = iter::once(formats::MAP32).chain(cast.to_be_bytes());
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
                let mut it = iter::once(cast | formats::FIX_MAP);
                for (to, byte) in buf.take(SIZE).zip(&mut it) {
                    *to = byte
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0xf0..=0xff => {
                const SIZE: usize = 3;
                let cast = self_len as u16;
                let mut it = iter::once(formats::MAP16).chain(cast.to_be_bytes());
                for (to, byte) in buf.take(SIZE).zip(&mut it) {
                    *to = byte
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0xffff..=0xffff => {
                const SIZE: usize = 5;
                let cast = self_len as u32;
                let mut it = iter::once(formats::MAP32).chain(cast.to_be_bytes());
                for (to, byte) in buf.take(SIZE).zip(&mut it) {
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
