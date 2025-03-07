use serde::de;

use super::{Deserializer, Error, error::CoreError, num::NumDecoder};

pub struct Enum<'de, 'a, Num>
where
    'de: 'a,
{
    de: &'a mut Deserializer<'de, Num>,
}

impl<'de, 'a, Num: NumDecoder<'de>> Enum<'de, 'a, Num> {
    pub fn new(de: &'a mut Deserializer<'de, Num>) -> Self {
        Enum { de }
    }
}

impl<'de, Num: NumDecoder<'de>> de::EnumAccess<'de> for Enum<'de, '_, Num> {
    type Error = Error;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let val = seed.deserialize(self.de.as_mut())?;

        Ok((val, self))
    }
}

impl<'de, Num: NumDecoder<'de>> de::VariantAccess<'de> for Enum<'de, '_, Num> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        // Unit variant should handle before
        Err(CoreError::UnexpectedFormat.into())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.de.as_mut())
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}
