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

    // ---- Non-enum decode coverage (based on de/mod.rs tests) ----
    #[rstest]
    #[case(Value::from(true), true)]
    #[case(Value::from(false), false)]
    fn decode_bool(#[case] v: Value, #[case] expected: bool) {
        let decoded = bool::deserialize(&v).unwrap();
        assert_eq!(decoded, expected);
    }

    #[rstest]
    #[case(Value::from(5u64), 5u8)]
    #[case(Value::from(128u64), 128u8)]
    fn decode_uint8(#[case] v: Value, #[case] expected: u8) {
        let decoded = u8::deserialize(&v).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn decode_float_vec() {
        let v = Value::Array(vec![
            Value::from(1.1f64),
            Value::from(1.2f64),
            Value::from(1.3f64),
            Value::from(1.4f64),
            Value::from(1.5f64),
        ]);
        let decoded = Vec::<f64>::deserialize(&v).unwrap();
        assert_eq!(decoded, vec![1.1, 1.2, 1.3, 1.4, 1.5]);
    }

    #[test]
    fn decode_struct() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct S {
            compact: bool,
            schema: u8,
        }
        let v = Value::Map(vec![
            (Value::from("compact"), Value::from(true)),
            (Value::from("schema"), Value::from(0u64)),
        ]);
        let decoded = S::deserialize(&v).unwrap();
        assert_eq!(decoded, S { compact: true, schema: 0 });
    }

    #[test]
    fn decode_struct_from_array() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct S {
            compact: bool,
            schema: u8,
        }
        let v = Value::Array(vec![Value::from(true), Value::from(0u64)]);
        let decoded = S::deserialize(&v).unwrap();
        assert_eq!(decoded, S { compact: true, schema: 0 });
    }

    #[test]
    fn option_consumes_nil_in_sequence() {
        let v = Value::Array(vec![Value::Nil, Value::from(5u64)]);
        let decoded = <(Option<u8>, u8)>::deserialize(&v).unwrap();
        assert_eq!(decoded, (None, 5));
    }

    #[test]
    fn option_some_simple() {
        let v = Value::from(5u64);
        let decoded = Option::<u8>::deserialize(&v).unwrap();
        assert_eq!(decoded, Some(5));
    }

    #[test]
    fn unit_from_nil() {
        let v = Value::Nil;
        let _: () = <()>::deserialize(&v).unwrap();
    }

    #[test]
    fn unit_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct U;
        let v = Value::Nil;
        let decoded = U::deserialize(&v).unwrap();
        assert_eq!(decoded, U);
    }

    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(untagged)]
    enum Untagged {
        Bool(bool),
        U8(u8),
        Pair(u8, bool),
        Struct { a: bool },
        Nested(E),
    }

    #[rstest]
    #[case(Value::from(true), Untagged::Bool(true))]
    #[case(Value::from(5u64), Untagged::U8(5))]
    #[case(Value::Array(vec![Value::from(2u64), Value::from(true)]), Untagged::Pair(2,true))]
    #[case(Value::Map(vec![(Value::from("a"), Value::from(false))]), Untagged::Struct { a: false })]
    #[case(Value::from("Unit"), Untagged::Nested(E::Unit))]
    fn decode_untagged(#[case] v: Value, #[case] expected: Untagged) {
        let decoded = Untagged::deserialize(&v).unwrap();
        assert_eq!(decoded, expected);
    }

    // -------- Extension tests --------
    use messagepack_core::extension::{ExtensionOwned, ExtensionRef, FixedExtension};

    #[derive(Deserialize, Debug, PartialEq)]
    struct WrapRef<'a>(
        #[serde(with = "crate::extension::ext_ref", borrow)] ExtensionRef<'a>,
    );

    #[test]
    fn decode_extension_ref_from_value() {
        let kind: i8 = 7;
        let data = vec![0x10, 0x20, 0x30];
        let v = Value::Extension(ExtensionOwned { r#type: kind, data: data.clone() });
        let WrapRef(ext) = WrapRef::deserialize(&v).unwrap();
        assert_eq!(ext.r#type, kind);
        assert_eq!(ext.data, &data[..]);
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct WrapOwned(#[serde(with = "crate::extension::ext_owned")] ExtensionOwned);

    #[test]
    fn decode_extension_owned_from_value() {
        let kind: i8 = 10;
        let data = vec![0xAA, 0xBB, 0xCC, 0xDD];
        let v = Value::Extension(ExtensionOwned { r#type: kind, data: data.clone() });
        let WrapOwned(ext) = WrapOwned::deserialize(&v).unwrap();
        assert_eq!(ext.r#type, kind);
        assert_eq!(ext.data, data);
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct WrapFixed<const N: usize>(
        #[serde(with = "crate::extension::ext_fixed")] FixedExtension<N>,
    );

    #[test]
    fn decode_extension_fixed_from_value() {
        let kind: i8 = 12;
        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let v = Value::Extension(ExtensionOwned { r#type: kind, data: data.clone() });
        let WrapFixed::<8>(ext) = WrapFixed::<8>::deserialize(&v).unwrap();
        assert_eq!(ext.r#type, kind);
        assert_eq!(ext.as_slice(), &data[..]);
    }
}
