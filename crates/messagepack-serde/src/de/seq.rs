use serde::de;

use super::{Deserializer, Error, num::NumDecoder};

pub struct FixLenAccess<'de, 'a, Num> {
    de: &'a mut Deserializer<'de, Num>,
    left: usize,
}

impl<'de, 'a, Num> FixLenAccess<'de, 'a, Num> {
    pub fn new(de: &'a mut Deserializer<'de, Num>, len: usize) -> Self {
        Self { de, left: len }
    }
}

impl<'de, 'a, Num> de::SeqAccess<'de> for FixLenAccess<'de, 'a, Num>
where
    'de: 'a,
    Num: NumDecoder<'de>,
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

impl<'de, 'a, Num> de::MapAccess<'de> for FixLenAccess<'de, 'a, Num>
where
    'de: 'a,
    Num: NumDecoder<'de>,
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
