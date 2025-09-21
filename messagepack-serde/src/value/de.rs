use crate::value::{Number, Value, ValueRef};
use serde::{
    de::{self},
    forward_to_deserialize_any,
};

type Error = crate::de::Error<core::convert::Infallible>;

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

mod value_ref {
    use super::*;
    impl<'de> de::Deserializer<'de> for &'de Value {
        type Error = Error;
        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            match self {
                Value::Nil => visitor.visit_unit(),
                Value::Bool(v) => visitor.visit_bool(*v),
                Value::Bin(items) => visitor.visit_borrowed_bytes(items),
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

    impl<'de> de::IntoDeserializer<'de, Error> for &'de Value {
        type Deserializer = Self;
        fn into_deserializer(self) -> Self::Deserializer {
            self
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
                    Some((content, [])) => {
                        let id = seed.deserialize(&content.0)?;
                        Ok((id, EnumRefVariant::Value(&content.1)))
                    }
                    _ => Err(de::Error::invalid_length(items.len(), &"expect 1 element")),
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
                            let de =
                                serde::de::value::I8Deserializer::<Error>::new(self.ext.r#type);
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

    impl<'de> de::Deserializer<'de> for &ValueRef<'de> {
        type Error = Error;

        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            match self {
                ValueRef::Nil => visitor.visit_unit(),
                ValueRef::Bool(b) => visitor.visit_bool(*b),
                ValueRef::Bin(b) => visitor.visit_borrowed_bytes(b),
                ValueRef::Extension(ext) => de::Deserializer::deserialize_newtype_struct(
                    ExtRefDeserializer { ext: *ext },
                    crate::extension::EXTENSION_STRUCT_NAME,
                    visitor,
                ),
                ValueRef::Number(n) => visit_number(visitor, *n),
                ValueRef::String(s) => visitor.visit_borrowed_str(s),
                ValueRef::Array(items) => {
                    let seq = SeqAccessValueRefBorrowed { it: items.iter() };
                    visitor.visit_seq(seq)
                }
                ValueRef::Map(items) => {
                    let map = MapAccessValueRefBorrowed {
                        it: items.iter(),
                        val: None,
                    };
                    visitor.visit_map(map)
                }
            }
        }

        fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            match self {
                ValueRef::Nil => visitor.visit_none(),
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
            let access = EnumAccessBorrowedValueRef { val: self };
            visitor.visit_enum(access)
        }

        forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf unit unit_struct newtype_struct seq tuple tuple_struct
            map struct identifier ignored_any
        }
    }

    impl<'de> de::IntoDeserializer<'de, Error> for &ValueRef<'de> {
        type Deserializer = Self;
        fn into_deserializer(self) -> Self::Deserializer {
            self
        }
    }

    struct SeqAccessValueRefBorrowed<'a, 'de> {
        it: core::slice::Iter<'a, ValueRef<'de>>,
    }

    impl<'a, 'de> de::SeqAccess<'de> for SeqAccessValueRefBorrowed<'a, 'de> {
        type Error = Error;

        fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where
            T: de::DeserializeSeed<'de>,
        {
            match self.it.next() {
                Some(v) => seed.deserialize(v).map(Some),
                None => Ok(None),
            }
        }

        fn size_hint(&self) -> Option<usize> {
            Some(self.it.len())
        }
    }

    struct MapAccessValueRefBorrowed<'a, 'de> {
        it: core::slice::Iter<'a, (ValueRef<'de>, ValueRef<'de>)>,
        val: Option<&'a ValueRef<'de>>,
    }

    impl<'a, 'de> de::MapAccess<'de> for MapAccessValueRefBorrowed<'a, 'de> {
        type Error = Error;

        fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where
            K: de::DeserializeSeed<'de>,
        {
            match self.it.next() {
                Some((k, v)) => {
                    self.val = Some(v);
                    seed.deserialize(k).map(Some)
                }
                None => Ok(None),
            }
        }

        fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where
            V: de::DeserializeSeed<'de>,
        {
            match self.val.take() {
                Some(v) => seed.deserialize(v),
                None => Err(<Error as de::Error>::custom("value is missing")),
            }
        }

        fn size_hint(&self) -> Option<usize> {
            Some(self.it.len())
        }
    }

    struct EnumAccessBorrowedValueRef<'a, 'de> {
        val: &'a ValueRef<'de>,
    }

    impl<'a, 'de> de::EnumAccess<'de> for EnumAccessBorrowedValueRef<'a, 'de> {
        type Error = Error;
        type Variant = VariantAccessBorrowedValueRef<'a, 'de>;

        fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
        where
            V: de::DeserializeSeed<'de>,
        {
            match self.val {
                ValueRef::String(tag) => {
                    let de = serde::de::value::BorrowedStrDeserializer::<Error>::new(tag);
                    let id = seed.deserialize(de)?;
                    Ok((id, VariantAccessBorrowedValueRef::String))
                }
                ValueRef::Map(items) => match items.as_slice().split_first() {
                    Some((first, [])) => {
                        let id = seed.deserialize(&first.0)?;
                        Ok((id, VariantAccessBorrowedValueRef::Value(&first.1)))
                    }
                    _ => Err(de::Error::invalid_length(items.len(), &"expect 1 element")),
                },
                _ => Err(de::Error::invalid_type(
                    de::Unexpected::Other("non-enum value"),
                    &"string or map for enum",
                )),
            }
        }
    }

    enum VariantAccessBorrowedValueRef<'a, 'de> {
        String,
        Value(&'a ValueRef<'de>),
    }

    impl<'a, 'de> de::VariantAccess<'de> for VariantAccessBorrowedValueRef<'a, 'de> {
        type Error = Error;

        fn unit_variant(self) -> Result<(), Self::Error> {
            match self {
                VariantAccessBorrowedValueRef::String => Ok(()),
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
                VariantAccessBorrowedValueRef::Value(v) => seed.deserialize(v),
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
                VariantAccessBorrowedValueRef::Value(v) => {
                    de::Deserializer::deserialize_seq(v, visitor)
                }
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
                VariantAccessBorrowedValueRef::Value(v) => {
                    de::Deserializer::deserialize_map(v, visitor)
                }
                _ => Err(de::Error::invalid_type(
                    de::Unexpected::Other("non-map variant content"),
                    &"map",
                )),
            }
        }
    }
}

mod value_owned {
    use super::*;

    impl<'de> de::Deserializer<'de> for Value {
        type Error = Error;

        fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            match self {
                Value::Nil => visitor.visit_unit(),
                Value::Bool(b) => visitor.visit_bool(b),
                Value::Bin(b) => visitor.visit_byte_buf(b),
                Value::Extension(ext) => de::Deserializer::deserialize_newtype_struct(
                    ExtDeserializerOwned {
                        kind: ext.r#type,
                        data: ext.data,
                    },
                    crate::extension::EXTENSION_STRUCT_NAME,
                    visitor,
                ),
                Value::Number(n) => visit_number(visitor, n),
                Value::String(s) => visitor.visit_string(s),
                Value::Array(items) => visitor.visit_seq(OwnedSeqAccess {
                    it: items.into_iter(),
                }),
                Value::Map(items) => visitor.visit_map(OwnedMapAccess {
                    it: items.into_iter(),
                    pending_value: None,
                }),
            }
        }

        fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            match self {
                Value::Nil => visitor.visit_none(),
                other => visitor.visit_some(other),
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
            let access = EnumAccessOwned { val: self };
            visitor.visit_enum(access)
        }

        forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf unit unit_struct newtype_struct seq tuple tuple_struct
            map struct identifier ignored_any
        }
    }

    impl<'de> de::IntoDeserializer<'de, Error> for Value {
        type Deserializer = Self;
        fn into_deserializer(self) -> Self::Deserializer {
            self
        }
    }

    struct OwnedSeqAccess {
        it: alloc::vec::IntoIter<Value>,
    }

    impl<'de> de::SeqAccess<'de> for OwnedSeqAccess {
        type Error = Error;
        fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where
            T: de::DeserializeSeed<'de>,
        {
            match self.it.next() {
                Some(v) => seed.deserialize(v).map(Some),
                None => Ok(None),
            }
        }

        fn size_hint(&self) -> Option<usize> {
            Some(self.it.as_slice().len())
        }
    }

    struct OwnedMapAccess {
        it: alloc::vec::IntoIter<(Value, Value)>,
        pending_value: Option<Value>,
    }

    impl<'de> de::MapAccess<'de> for OwnedMapAccess {
        type Error = Error;

        fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
        where
            K: de::DeserializeSeed<'de>,
        {
            match self.it.next() {
                Some((k, v)) => {
                    self.pending_value = Some(v);
                    seed.deserialize(k).map(Some)
                }
                None => Ok(None),
            }
        }

        fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
        where
            V: de::DeserializeSeed<'de>,
        {
            let v = self
                .pending_value
                .take()
                .ok_or_else(|| <Error as de::Error>::custom("value is missing for map key"))?;
            seed.deserialize(v)
        }

        fn size_hint(&self) -> Option<usize> {
            let plus_one = if self.pending_value.is_some() { 1 } else { 0 };
            Some(self.it.as_slice().len() + plus_one)
        }
    }

    struct EnumAccessOwned {
        val: Value,
    }

    enum VariantAccessOwned {
        String,
        Value(Value),
    }

    impl<'de> de::EnumAccess<'de> for EnumAccessOwned {
        type Error = Error;
        type Variant = VariantAccessOwned;

        fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
        where
            V: de::DeserializeSeed<'de>,
        {
            match self.val {
                Value::String(tag) => {
                    let de = serde::de::value::StrDeserializer::<Error>::new(&tag);
                    let id = seed.deserialize(de)?;
                    Ok((id, VariantAccessOwned::String))
                }
                Value::Map(mut items) => {
                    if items.len() != 1 {
                        return Err(de::Error::invalid_length(
                            items.len(),
                            &"{variant: content}",
                        ));
                    }
                    let (k, v) = items.remove(0);
                    let id = seed.deserialize(k)?;
                    Ok((id, VariantAccessOwned::Value(v)))
                }
                _other => Err(de::Error::invalid_type(
                    de::Unexpected::Other("non-enum value"),
                    &"string, array, or map for enum",
                )),
            }
        }
    }

    impl<'de> de::VariantAccess<'de> for VariantAccessOwned {
        type Error = Error;

        fn unit_variant(self) -> Result<(), Self::Error> {
            match self {
                VariantAccessOwned::String => Ok(()),
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
                Self::Value(v) => seed.deserialize(v),
                _ => Err(de::Error::invalid_type(
                    de::Unexpected::Other("non-newtype enum variant"),
                    &"expect map",
                )),
            }
        }

        fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            match self {
                Self::Value(v) => de::Deserializer::deserialize_seq(v, visitor),
                _ => Err(de::Error::invalid_type(
                    de::Unexpected::Other("non-seq variant content"),
                    &"array",
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
                Self::Value(v) => de::Deserializer::deserialize_map(v, visitor),
                _ => Err(de::Error::invalid_type(
                    de::Unexpected::Other("non-map variant content"),
                    &"map",
                )),
            }
        }
    }

    // Bridge for owned extension
    struct ExtDeserializerOwned {
        kind: i8,
        data: alloc::vec::Vec<u8>,
    }

    impl<'de> de::Deserializer<'de> for ExtDeserializerOwned {
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
                visitor.visit_newtype_struct(ExtSeqOwned {
                    kind: self.kind,
                    data: self.data,
                })
            } else {
                Err(<Error as de::Error>::custom("invalid entry point"))
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

    struct ExtSeqOwned {
        kind: i8,
        data: alloc::vec::Vec<u8>,
    }

    impl<'de> de::Deserializer<'de> for ExtSeqOwned {
        type Error = Error;

        fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            struct Access {
                kind: Option<i8>,
                data: Option<alloc::vec::Vec<u8>>,
            }
            impl<'de> de::SeqAccess<'de> for Access {
                type Error = Error;
                fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
                where
                    T: de::DeserializeSeed<'de>,
                {
                    if let Some(k) = self.kind.take() {
                        let de = serde::de::value::I8Deserializer::<Error>::new(k);
                        let v = seed.deserialize(de)?;
                        Ok(Some(v))
                    } else if let Some(bytes) = self.data.take() {
                        // Yield bytes as borrowed if possible; for simplicity, pass by bytes/byte_buf
                        let v = seed.deserialize(BytesElemOwned(bytes))?;
                        Ok(Some(v))
                    } else {
                        Ok(None)
                    }
                }
            }
            visitor.visit_seq(Access {
                kind: Some(self.kind),
                data: Some(self.data),
            })
        }

        fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            Err(<Error as de::Error>::custom("invalid entry point"))
        }

        fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            self.deserialize_seq(visitor)
        }

        fn deserialize_tuple_struct<V>(
            self,
            _name: &'static str,
            _len: usize,
            visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            self.deserialize_seq(visitor)
        }

        forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct
            map struct enum identifier ignored_any
        }
    }

    // Minimal wrapper to feed byte content for owned extension
    struct BytesElemOwned(alloc::vec::Vec<u8>);

    impl<'de> de::Deserializer<'de> for BytesElemOwned {
        type Error = Error;
        fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            Err(<Error as de::Error>::custom("invalid entry point"))
        }
        fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            visitor.visit_bytes(&self.0)
        }
        fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            visitor.visit_byte_buf(self.0)
        }
        forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            option unit unit_struct newtype_struct seq tuple tuple_struct map struct enum identifier ignored_any
        }
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

    #[rstest]
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

    #[rstest]
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
        assert_eq!(
            decoded,
            S {
                compact: true,
                schema: 0
            }
        );
    }

    #[rstest]
    fn decode_struct_from_array() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct S {
            compact: bool,
            schema: u8,
        }
        let v = Value::Array(vec![Value::from(true), Value::from(0u64)]);
        let decoded = S::deserialize(&v).unwrap();
        assert_eq!(
            decoded,
            S {
                compact: true,
                schema: 0
            }
        );
    }

    #[rstest]
    fn option_consumes_nil_in_sequence() {
        let v = Value::Array(vec![Value::Nil, Value::from(5u64)]);
        let decoded = <(Option<u8>, u8)>::deserialize(&v).unwrap();
        assert_eq!(decoded, (None, 5));
    }

    #[rstest]
    fn option_some_simple() {
        let v = Value::from(5u64);
        let decoded = Option::<u8>::deserialize(&v).unwrap();
        assert_eq!(decoded, Some(5));
    }

    #[rstest]
    fn unit_from_nil() {
        let v = Value::Nil;
        let _: () = <()>::deserialize(&v).unwrap();
    }

    #[rstest]
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
    struct WrapRef<'a>(#[serde(with = "crate::extension::ext_ref", borrow)] ExtensionRef<'a>);

    #[rstest]
    fn decode_extension_ref_from_value() {
        let kind: i8 = 7;
        let data = vec![0x10, 0x20, 0x30];
        let v = Value::Extension(ExtensionOwned {
            r#type: kind,
            data: data.clone(),
        });
        let WrapRef(ext) = WrapRef::deserialize(&v).unwrap();
        assert_eq!(ext.r#type, kind);
        assert_eq!(ext.data, &data[..]);
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct WrapOwned(#[serde(with = "crate::extension::ext_owned")] ExtensionOwned);

    #[rstest]
    fn decode_extension_owned_from_value() {
        let kind: i8 = 10;
        let data = vec![0xAA, 0xBB, 0xCC, 0xDD];
        let v = Value::Extension(ExtensionOwned {
            r#type: kind,
            data: data.clone(),
        });
        let WrapOwned(ext) = WrapOwned::deserialize(&v).unwrap();
        assert_eq!(ext.r#type, kind);
        assert_eq!(ext.data, data);
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct WrapFixed<const N: usize>(
        #[serde(with = "crate::extension::ext_fixed")] FixedExtension<N>,
    );

    #[rstest]
    fn decode_extension_fixed_from_value() {
        let kind: i8 = 12;
        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let v = Value::Extension(ExtensionOwned {
            r#type: kind,
            data: data.clone(),
        });
        let WrapFixed::<8>(ext) = WrapFixed::<8>::deserialize(&v).unwrap();
        assert_eq!(ext.r#type, kind);
        assert_eq!(ext.as_slice(), &data[..]);
    }

    // ---- Zero-copy efficiency tests for owned Value ----
    #[rstest]
    fn bin_zero_copy_into_bytebuf() {
        let mut buf = vec![0u8; 256];
        for (i, b) in buf.iter_mut().enumerate() {
            *b = (i as u8) ^ 0x5A;
        }
        let ptr = buf.as_ptr() as usize;
        let cap = buf.capacity();
        let v = Value::Bin(buf);

        let bb = serde_bytes::ByteBuf::deserialize(v).unwrap();
        let moved = bb.into_vec();
        assert_eq!(moved.len(), 256);
        assert_eq!(moved.capacity(), cap);
        assert_eq!(moved.as_ptr() as usize, ptr);
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename = "$__MSGPACK_EXTENSION_STRUCT")]
    struct ExtBuf((i8, serde_bytes::ByteBuf));

    #[rstest]
    fn extension_zero_copy_into_bytebuf_newtype() {
        let kind: i8 = 42;
        let mut data = vec![0u8; 128];
        for (i, b) in data.iter_mut().enumerate() {
            *b = (i as u8) ^ 0xA5;
        }
        let ptr = data.as_ptr() as usize;
        let cap = data.capacity();

        // Owned Value with Extension
        let v = Value::Extension(ExtensionOwned { r#type: kind, data });

        // Decode into a custom newtype that accepts (i8, ByteBuf) inside the extension newtype
        let ExtBuf((ty, bb)) = ExtBuf::deserialize(v).unwrap();
        assert_eq!(ty, kind);
        let moved = bb.into_vec();
        assert_eq!(moved.len(), 128);
        assert_eq!(moved.capacity(), cap);
        assert_eq!(moved.as_ptr() as usize, ptr);
    }

    #[rstest]
    #[case(true)]
    #[case(false)]
    fn vref_decode_bool(#[case] expected: bool) {
        let v = ValueRef::Bool(expected);
        let b = bool::deserialize(&v).unwrap();
        assert_eq!(b, expected);
    }

    #[rstest]
    #[case(5u64, 5u8)]
    #[case(128u64, 128u8)]
    fn vref_decode_u8(#[case] input: u64, #[case] expected: u8) {
        let v = ValueRef::from(input);
        let n = u8::deserialize(&v).unwrap();
        assert_eq!(n, expected);
    }

    #[rstest]
    #[case(1.5f64)]
    #[case(0.0f64)]
    fn vref_decode_f64(#[case] input: f64) {
        let v = ValueRef::from(input);
        let f = f64::deserialize(&v).unwrap();
        assert_eq!(f, input);
    }

    #[rstest]
    #[case("hello")]
    #[case("")]
    fn vref_decode_borrowed_str(#[case] s: &'static str) {
        let v = ValueRef::String(s);
        let out = <&str>::deserialize(&v).unwrap();
        assert_eq!(out, s);
        assert_eq!(out.as_ptr(), s.as_ptr());
    }

    #[rstest]
    #[case(b"world".as_slice())]
    #[case(b"".as_slice())]
    fn vref_decode_borrowed_bytes(#[case] bytes: &'static [u8]) {
        let v = ValueRef::Bin(bytes);
        let out = <&[u8]>::deserialize(&v).unwrap();
        assert_eq!(out, bytes);
        assert_eq!(out.as_ptr(), bytes.as_ptr());
    }

    #[rstest]
    fn vref_decode_vec_and_struct() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct S {
            compact: bool,
            schema: u8,
        }

        let v = ValueRef::Array(vec![ValueRef::from(true), ValueRef::from(0u64)]);
        let s = S::deserialize(&v).unwrap();
        assert_eq!(
            s,
            S {
                compact: true,
                schema: 0,
            }
        );

        let v = ValueRef::Array(vec![
            ValueRef::from(1.1f64),
            ValueRef::from(1.2f64),
            ValueRef::from(1.3f64),
        ]);
        let out = Vec::<f64>::deserialize(&v).unwrap();
        assert_eq!(out, vec![1.1, 1.2, 1.3]);
    }

    #[rstest]
    #[case(ValueRef::from("Unit"), E::Unit)]
    #[case(ValueRef::Map(vec![(ValueRef::from("Newtype"), ValueRef::from(27u64))]), E::Newtype(27))]
    #[case(
        ValueRef::Map(vec![(
            ValueRef::from("Tuple"),
            ValueRef::Array(vec![ValueRef::from(3u64), ValueRef::from(true)]),
        )]),
        E::Tuple(3, true)
    )]
    #[case(
        ValueRef::Map(vec![(
            ValueRef::from("Struct"),
            ValueRef::Map(vec![(ValueRef::from("a"), ValueRef::from(false))]),
        )]),
        E::Struct { a: false }
    )]
    fn vref_decode_enum(#[case] v: ValueRef<'_>, #[case] expected: E) {
        let decoded = E::deserialize(&v).unwrap();
        assert_eq!(decoded, expected);
    }

    #[rstest]
    #[case(5u64, Some(5u8))]
    #[case(255u64, Some(255u8))]
    fn vref_decode_option_some(#[case] input: u64, #[case] expected: Option<u8>) {
        let v = ValueRef::from(input);
        let o = Option::<u8>::deserialize(&v).unwrap();
        assert_eq!(o, expected);
    }

    #[test]
    fn vref_option_consumes_nil_in_sequence() {
        let v = ValueRef::Array(vec![ValueRef::Nil, ValueRef::from(5u64)]);
        let out = <(Option<u8>, u8)>::deserialize(&v).unwrap();
        assert_eq!(out, (None, 5));
    }

    #[test]
    fn vref_decode_extension_ref() {
        use messagepack_core::extension::ExtensionRef;

        #[derive(Deserialize, Debug, PartialEq)]
        struct WrapRef<'a>(#[serde(with = "crate::extension::ext_ref", borrow)] ExtensionRef<'a>);

        let kind: i8 = 7;
        let data: &'static [u8] = &[0x10, 0x20, 0x30];
        let v = ValueRef::Extension(ExtensionRef::new(kind, data));
        let WrapRef(ext) = WrapRef::deserialize(&v).unwrap();
        assert_eq!(ext.r#type, kind);
        assert_eq!(ext.data, data);
    }
}
