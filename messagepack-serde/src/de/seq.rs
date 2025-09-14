use core::marker::PhantomData;

use messagepack_core::io::IoRead;
use serde::de;

use crate::de::error::CoreError;

use super::{Deserializer, Error};

pub struct FixLenAccess<'a, R> {
    de: &'a mut Deserializer<R>,
    left: usize,
}

impl<'a, R> FixLenAccess<'a, R> {
    pub fn new(de: &'a mut Deserializer<R>, len: usize) -> Self {
        Self { de, left: len }
    }
}

impl<'de, 'a, R> de::SeqAccess<'de> for FixLenAccess<'a, R>
where
    'de: 'a,
    R: IoRead<'de>,
{
    type Error = Error<R::Error>;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.left > 0 {
            self.left -= 1;

            let value = seed.deserialize(self.de.as_mut());

            value.map(Some)
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.left)
    }
}

impl<'de, 'a, R> de::MapAccess<'de> for FixLenAccess<'a, R>
where
    'de: 'a,
    R: IoRead<'de>,
{
    type Error = Error<R::Error>;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.left > 0 {
            self.left -= 1;

            let value = seed.deserialize(self.de.as_mut());

            value.map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.de.as_mut())
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.left)
    }
}

pub struct ByteAccess<'a, R> {
    buf: &'a [u8],
    marker: PhantomData<fn() -> R>,
}

impl<'a, R> ByteAccess<'a, R> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            buf,
            marker: PhantomData,
        }
    }
}

impl<'de, 'a, R> de::Deserializer<'de> for &mut ByteAccess<'a, R>
where
    R: IoRead<'de>,
    'de: 'a,
{
    type Error = Error<R::Error>;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(de::Error::custom(
            "ByteAccess deserialize only support `u8`",
        ))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (first, rest) = self.buf.split_first().ok_or(CoreError::UnexpectedEof)?;
        self.buf = rest;
        visitor.visit_u8(*first)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, 'a, R> de::SeqAccess<'de> for ByteAccess<'a, R>
where
    R: IoRead<'de>,
    'de: 'a,
{
    type Error = Error<R::Error>;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.buf.is_empty() {
            Ok(None)
        } else {
            let val = seed.deserialize(self);
            val.map(Some)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.buf.len())
    }
}
