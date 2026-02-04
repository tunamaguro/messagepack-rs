use messagepack_core::io::IoRead;
use serde::de;

use super::{Deserializer, Error, error::CoreError};

pub struct Enum<'a, R> {
    de: &'a mut Deserializer<R>,
}

impl<'a, R> Enum<'a, R> {
    pub fn new(de: &'a mut Deserializer<R>) -> Self {
        Enum { de }
    }
}

impl<'de, 'a, R> de::EnumAccess<'de> for Enum<'a, R>
where
    'de: 'a,
    R: IoRead<'de>,
{
    type Error = Error<R::Error>;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let val = seed.deserialize(self.de.as_mut())?;

        Ok((val, self))
    }
}

impl<'de, 'a, R> de::VariantAccess<'de> for Enum<'a, R>
where
    'de: 'a,
    R: IoRead<'de>,
{
    type Error = Error<R::Error>;

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
