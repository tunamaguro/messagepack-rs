use messagepack_core::io::IoRead;
use serde::de;

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
