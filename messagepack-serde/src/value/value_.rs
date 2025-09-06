use super::{extension::ExtensionRef, number::Number};
use alloc::vec::Vec;
use serde::{Deserialize, de::Visitor, ser::SerializeMap};

/// Represents any messagepack value. `alloc` needed.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ValueRef<'a> {
    Nil,
    Bool(bool),
    Bin(&'a [u8]),
    Extension(ExtensionRef<'a>),
    Number(Number),
    String(&'a str),
    Array(Vec<ValueRef<'a>>),
    Map(Vec<(ValueRef<'a>, ValueRef<'a>)>),
}

impl ValueRef<'_> {
    pub fn is_nil(&self) -> bool {
        matches!(self, ValueRef::Nil)
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ValueRef::Bool(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_bin(&self) -> Option<&[u8]> {
        match self {
            ValueRef::Bin(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_extension(&self) -> Option<&ExtensionRef<'_>> {
        match self {
            ValueRef::Extension(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_number(&self) -> Option<Number> {
        match self {
            ValueRef::Number(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ValueRef::String(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_array(&self) -> Option<&[ValueRef<'_>]> {
        match self {
            ValueRef::Array(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_map(&self) -> Option<&[(ValueRef<'_>, ValueRef<'_>)]> {
        match self {
            ValueRef::Map(v) => Some(v),
            _ => None,
        }
    }
}

impl serde::Serialize for ValueRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ValueRef::Nil => serializer.serialize_none(),
            ValueRef::Bool(v) => serializer.serialize_bool(*v),
            ValueRef::Bin(items) => (*items).serialize(serializer),
            ValueRef::Extension(extension_ref) => extension_ref.serialize(serializer),
            ValueRef::Number(number) => number.serialize(serializer),
            ValueRef::String(s) => serializer.serialize_str(s),
            ValueRef::Array(value_refs) => (*value_refs).serialize(serializer),
            ValueRef::Map(items) => {
                let mut map = serializer.serialize_map(Some(items.len()))?;
                for (k, v) in items.iter() {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> serde::Deserialize<'de> for ValueRef<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;
        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = ValueRef<'de>;
            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("expect valid messagepack")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ValueRef::Bool(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let n = Number::UnsignedInt(v);
                Ok(ValueRef::Number(n))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let n = Number::SignedInt(v);
                Ok(ValueRef::Number(n))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let n = Number::Float(v);
                Ok(ValueRef::Number(n))
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ValueRef::String(v))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ValueRef::Nil)
            }

            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ValueRef::Bin(v))
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let ext = ExtensionRef::deserialize(deserializer)?;
                Ok(ValueRef::Extension(ext))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut buf = Vec::new();

                while let Some(v) = seq.next_element::<ValueRef>()? {
                    buf.push(v);
                }

                Ok(ValueRef::Array(buf))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut buf = Vec::new();

                while let Some(v) = map.next_entry()? {
                    buf.push(v);
                }

                Ok(ValueRef::Map(buf))
            }
        }
        deserializer.deserialize_any(ValueVisitor)
    }
}
