use messagepack_core::{Decode, Format};
use serde::de;

use super::{Deserializer, Error};

pub struct ArrayDeserializer<'de, 'a> {
    de: &'a mut Deserializer<'de>,
    left: usize,
}

impl<'de, 'a> ArrayDeserializer<'de, 'a> {
    pub fn new(de: &'a mut Deserializer<'de>, len: usize) -> Self {
        Self { de, left: len }
    }
}

impl<'de, 'a> de::SeqAccess<'de> for ArrayDeserializer<'de, 'a>
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

            #[cfg(test)]
            {
                let format = Format::decode(self.de.input)?;
                dbg!(format);
            }

            let value = seed.deserialize(self.de.as_mut());

            value.map(Some)
        } else {
            Ok(None)
        }
    }
}
