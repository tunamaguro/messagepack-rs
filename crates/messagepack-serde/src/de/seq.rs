use serde::de;

use super::{Deserializer, Error};

pub struct FixLenAccess<'de, 'a> {
    de: &'a mut Deserializer<'de>,
    left: usize,
}

impl<'de, 'a> FixLenAccess<'de, 'a> {
    pub fn new(de: &'a mut Deserializer<'de>, len: usize) -> Self {
        Self { de, left: len }
    }
}

impl<'de, 'a> de::SeqAccess<'de> for FixLenAccess<'de, 'a>
where
    'de: 'a,
{
    type Error = Error;

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
}

impl<'de, 'a> de::MapAccess<'de> for FixLenAccess<'de, 'a>
where
    'de: 'a,
{
    type Error = Error;

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
}
