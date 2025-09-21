// Deserialization from dynamic Value/ValueRef into arbitrary T.
// All comments must be in English per repository guidelines.

use crate::value::{Number, Value};
use serde::{
    de::{self},
    forward_to_deserialize_any,
};

type Error = crate::de::Error<core::convert::Infallible>;

impl<'de> de::Deserializer<'de> for &'de Value {
    type Error = Error;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Value::Nil => visitor.visit_unit(),
            Value::Bool(v) => visitor.visit_bool(*v),
            Value::Bin(items) => visitor.visit_borrowed_bytes(&items),
            Value::Extension(extension_owned) => {
                // Bridge to the extension helper using a newtype struct
                let ext = extension_owned.as_ref();
                de::Deserializer::deserialize_newtype_struct(
                    ExtRefDeserializer { ext },
                    crate::extension::EXTENSION_STRUCT_NAME,
                    visitor,
                )
            }
            Value::Number(number) => visit_number(visitor, *number),
            Value::String(s) => visitor.visit_borrowed_str(s.as_str()),
            Value::Array(values) => {
                let seq = SeqRefAccess::new(values.iter());
                visitor.visit_seq(seq)
            }
            Value::Map(items) => {
                let map = MapRefAccess::new(items.iter());
                visitor.visit_map(map)
            }
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Value::Nil => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let access = EnumRefAccess::new(self);
        visitor.visit_enum(access)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

fn visit_number<'de, V>(visitor: V, number: Number) -> Result<V::Value, Error>
where
    V: de::Visitor<'de>,
{
    match number {
        Number::PositiveInt(i) => visitor.visit_u64(i),
        Number::NegativeInt(i) => visitor.visit_i64(i),
        Number::Float(f) => visitor.visit_f64(f),
    }
}

struct SeqRefAccess<'de, I>
where
    I: Iterator<Item = &'de Value> + ExactSizeIterator,
{
    iter: I,
}

impl<'de, I> SeqRefAccess<'de, I>
where
    I: Iterator<Item = &'de Value> + ExactSizeIterator,
{
    fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<'de, I> de::SeqAccess<'de> for SeqRefAccess<'de, I>
where
    I: Iterator<Item = &'de Value> + ExactSizeIterator,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(v) => seed.deserialize(v).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

struct MapRefAccess<'de, I>
where
    I: Iterator<Item = &'de (Value, Value)> + ExactSizeIterator,
{
    iter: I,
    val: Option<&'de Value>,
}

impl<'de, I> MapRefAccess<'de, I>
where
    I: Iterator<Item = &'de (Value, Value)> + ExactSizeIterator,
{
    fn new(iter: I) -> Self {
        Self { iter, val: None }
    }
}

impl<'de, I> de::MapAccess<'de> for MapRefAccess<'de, I>
where
    I: Iterator<Item = &'de (Value, Value)> + ExactSizeIterator,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.val = Some(value);
                seed.deserialize(key).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.val.take() {
            Some(val) => seed.deserialize(val),
            None => Err(<Error as de::Error>::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

struct EnumRefAccess<'de> {
    val: &'de Value,
}

impl<'de> EnumRefAccess<'de> {
    fn new(val: &'de Value) -> Self {
        Self { val }
    }
}

impl<'de> de::EnumAccess<'de> for EnumRefAccess<'de> {
    type Error = Error;
    type Variant = EnumRefVariant<'de>;
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.val {
            Value::String(_) => {
                let id = seed.deserialize(self.val)?;
                Ok((id, EnumRefVariant::String))
            }
            // Map-tagged enum: { tag: content }
            Value::Map(items) => match items.split_first() {
                Some((content, rest)) if rest.len() == 0 => {
                    let id = seed.deserialize(&content.0)?;
                    Ok((id, EnumRefVariant::Value(&content.1)))
                }
                _ => return Err(de::Error::invalid_length(items.len(), &"expect 1 element")),
            },
            _ => Err(de::Error::invalid_type(
                de::Unexpected::Other("non-enum value"),
                &"string or map for enum",
            )),
        }
    }
}

enum EnumRefVariant<'de> {
    String,
    Value(&'de Value),
}

impl<'de> de::VariantAccess<'de> for EnumRefVariant<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self {
            EnumRefVariant::String => Ok(()),
            _ => Err(de::Error::invalid_type(
                de::Unexpected::Other("non-unit enum variant"),
                &"unit variant",
            )),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self {
            EnumRefVariant::Value(value) => seed.deserialize(value),
            _ => Err(de::Error::invalid_type(
                de::Unexpected::Other("non-newtype enum variant"),
                &"array or map",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            EnumRefVariant::Value(value) => de::Deserializer::deserialize_seq(value, visitor),
            _ => Err(de::Error::invalid_type(
                de::Unexpected::Other("non-newtype enum variant"),
                &"array or map",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            EnumRefVariant::Value(value) => de::Deserializer::deserialize_map(value, visitor),
            _ => Err(de::Error::invalid_type(
                de::Unexpected::Other("non-map variant content"),
                &"map",
            )),
        }
    }
}

// ------------------------
// Extension bridge for &Value
// ------------------------

struct ExtRefDeserializer<'de> {
    ext: messagepack_core::extension::ExtensionRef<'de>,
}

impl<'de> de::Deserializer<'de> for ExtRefDeserializer<'de> {
    type Error = Error;

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if name == crate::extension::EXTENSION_STRUCT_NAME {
            visitor.visit_newtype_struct(ExtSeqRef { ext: self.ext })
        } else {
            Err(<Error as de::Error>::custom(
                "unexpected newtype name for extension",
            ))
        }
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(<Error as de::Error>::custom("invalid entry point"))
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct seq tuple tuple_struct map struct
        enum identifier ignored_any
    }
}

struct ExtSeqRef<'de> {
    ext: messagepack_core::extension::ExtensionRef<'de>,
}

impl<'de> de::Deserializer<'de> for ExtSeqRef<'de> {
    type Error = Error;

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        struct Access<'de> {
            ext: messagepack_core::extension::ExtensionRef<'de>,
            idx: u8,
        }

        impl<'de> de::SeqAccess<'de> for Access<'de> {
            type Error = Error;
            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
            where
                T: de::DeserializeSeed<'de>,
            {
                match self.idx {
                    0 => {
                        self.idx = 1;
                        let de = serde::de::value::I8Deserializer::<Error>::new(self.ext.r#type);
                        let v = seed.deserialize(de)?;
                        Ok(Some(v))
                    }
                    1 => {
                        self.idx = 2;
                        let de = serde::de::value::BorrowedBytesDeserializer::<Error>::new(
                            self.ext.data,
                        );
                        let v = seed.deserialize(de)?;
                        Ok(Some(v))
                    }
                    _ => Ok(None),
                }
            }
        }

        visitor.visit_seq(Access {
            ext: self.ext,
            idx: 0,
        })
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(<Error as de::Error>::custom("invalid entry point"))
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct tuple tuple_struct
        map struct enum identifier ignored_any
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde::Deserialize;

    #[derive(Deserialize, PartialEq, Debug)]
    enum E {
        Unit,
        Newtype(u8),
        Tuple(u8, bool),
        Struct { a: bool },
    }

    #[rstest]
    #[case(Value::from("Unit"), E::Unit)]
    #[case(
        Value::Map(vec![(Value::from("Newtype"), Value::from(27u64))]),
        E::Newtype(27)
    )]
    #[case(
        Value::Map(vec![(
            Value::from("Tuple"),
            Value::Array(vec![Value::from(3), Value::from(true)])
        )]),
        E::Tuple(3, true)
    )]
    #[case(
        Value::Map(vec![(
            Value::from("Struct"),
            Value::Map(vec![(Value::from("a"), Value::from(false))])
        )]),
        E::Struct { a: false }
    )]
    fn decode_enum(#[case] v: Value, #[case] expected: E) {
        let decoded = E::deserialize(&v).unwrap();
        assert_eq!(decoded, expected);
    }
}
