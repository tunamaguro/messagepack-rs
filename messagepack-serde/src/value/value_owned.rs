use super::number::Number;
use super::value_ref::ValueRef;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use messagepack_core::extension::{ExtensionOwned, ExtensionRef};
use serde::{de::Visitor, ser::SerializeMap};

/// Owned representation of any MessagePack value.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    /// Represents nil format.
    Nil,
    /// Represents bool format family.
    Bool(bool),
    /// Represents `bin 8`, `bin 16` and `bin 32`.
    Bin(Vec<u8>),
    /// Represents ext format family as owned bytes.
    Extension(ExtensionOwned),
    /// Represents int format family and float format family.
    Number(Number),
    /// Represents str format family.
    String(String),
    /// Represents array format family.
    Array(Vec<Value>),
    /// Represents map format family.
    Map(Vec<(Value, Value)>),
}

impl Value {
    /// Returns true if the `Value` is nil.
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    /// If the `Value` is boolean, returns contained value.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// If the `Value` is bin, returns contained slice.
    pub fn as_bin(&self) -> Option<&[u8]> {
        match self {
            Value::Bin(v) => Some(v.as_slice()),
            _ => None,
        }
    }

    /// If the `Value` is ext, returns `(type, data)` as tuple.
    pub fn as_extension(&self) -> Option<ExtensionRef<'_>> {
        match self {
            Value::Extension(ext) => Some(ext.as_ref()),
            _ => None,
        }
    }

    /// If the `Value` is number, returns contained value.
    pub fn as_number(&self) -> Option<Number> {
        match self {
            Value::Number(v) => Some(*v),
            _ => None,
        }
    }

    /// If the `Value` is str, returns contained slice.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(v) => Some(v.as_str()),
            _ => None,
        }
    }

    /// If the `Value` is array, returns contained slice.
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(v) => Some(v.as_slice()),
            _ => None,
        }
    }

    /// If the `Value` is map, returns contained slice of pairs.
    pub fn as_map(&self) -> Option<&[(Value, Value)]> {
        match self {
            Value::Map(v) => Some(v.as_slice()),
            _ => None,
        }
    }
}

impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Nil => serializer.serialize_none(),
            Value::Bool(v) => serializer.serialize_bool(*v),
            Value::Bin(b) => serializer.serialize_bytes(b),
            Value::Extension(ext) => {
                crate::extension::ext_ref::serialize(&ext.as_ref(), serializer)
            }
            Value::Number(n) => n.serialize(serializer),
            Value::String(s) => serializer.serialize_str(s),
            Value::Array(vs) => vs.serialize(serializer),
            Value::Map(items) => {
                let mut map = serializer.serialize_map(Some(items.len()))?;
                for (k, v) in items.iter() {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;
        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;
            fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.write_str("expect valid messagepack")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Bool(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::from(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::from(v))
            }

            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::from(v))
            }
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::from(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::from(v))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::from(v))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Nil)
            }
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Nil)
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Bin(v.to_vec()))
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Bin(v))
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let ext = crate::extension::ext_owned::deserialize(deserializer)?;
                Ok(Value::Extension(ext))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut buf = Vec::new();
                if let Some(size) = seq.size_hint() {
                    buf.reserve(size);
                }

                while let Some(v) = seq.next_element::<Value>()? {
                    buf.push(v);
                }
                Ok(Value::Array(buf))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut buf = Vec::new();
                if let Some(size) = map.size_hint() {
                    buf.reserve(size);
                }

                while let Some(v) = map.next_entry::<Value, Value>()? {
                    buf.push(v);
                }
                Ok(Value::Map(buf))
            }
        }
        deserializer.deserialize_any(ValueVisitor)
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Nil
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<u8> for Value {
    fn from(v: u8) -> Self {
        Value::Number(Number::from(v))
    }
}
impl From<u16> for Value {
    fn from(v: u16) -> Self {
        Value::Number(Number::from(v))
    }
}
impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::Number(Number::from(v))
    }
}
impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Value::Number(Number::from(v))
    }
}

impl From<i8> for Value {
    fn from(v: i8) -> Self {
        Value::Number(Number::from(v))
    }
}
impl From<i16> for Value {
    fn from(v: i16) -> Self {
        Value::Number(Number::from(v))
    }
}
impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Number(Number::from(v))
    }
}
impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Number(Number::from(v))
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::Number(Number::Float(v.into()))
    }
}
impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Number(Number::Float(v))
    }
}

impl From<Number> for Value {
    fn from(v: Number) -> Self {
        Value::Number(v)
    }
}

impl TryFrom<usize> for Value {
    type Error = core::num::TryFromIntError;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Number::try_from(value).map(Self::from)
    }
}

impl TryFrom<isize> for Value {
    type Error = core::num::TryFromIntError;
    fn try_from(value: isize) -> Result<Self, Self::Error> {
        Number::try_from(value).map(Self::from)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_string())
    }
}
impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}
impl From<&[u8]> for Value {
    fn from(v: &[u8]) -> Self {
        Value::Bin(v.to_vec())
    }
}
impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::Bin(v)
    }
}

impl From<ExtensionOwned> for Value {
    fn from(v: ExtensionOwned) -> Self {
        Value::Extension(v)
    }
}

impl From<ValueRef<'_>> for Value {
    fn from(v: ValueRef<'_>) -> Self {
        match v {
            ValueRef::Nil => Value::Nil,
            ValueRef::Bool(b) => Value::Bool(b),
            ValueRef::Bin(b) => Value::Bin(b.to_vec()),
            ValueRef::Extension(ext) => Value::Extension(ext.into()),
            ValueRef::Number(n) => Value::Number(n),
            ValueRef::String(s) => Value::String(s.to_string()),
            ValueRef::Array(items) => Value::Array(items.into_iter().map(Value::from).collect()),
            ValueRef::Map(items) => Value::Map(
                items
                    .into_iter()
                    .map(|(k, v)| (Value::from(k), Value::from(v)))
                    .collect(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_slice, to_slice};

    #[test]
    fn owned_roundtrip_primitives() {
        let cases = [
            Value::Nil,
            Value::Bool(true),
            Value::Number(Number::from(7u64)),
            Value::Number(Number::from(-3i64)),
            Value::Number(Number::Float(1.25)),
            Value::String("hi".to_string()),
            Value::Bin(vec![0x01, 0x02]),
        ];
        for v in cases.iter() {
            let mut buf = [0u8; 32];
            let len = to_slice(v, &mut buf).unwrap();
            let decoded = from_slice::<Value>(&buf[..len]).unwrap();
            assert_eq!(&decoded, v);
        }
    }

    #[test]
    fn owned_nested_roundtrip() {
        let v = Value::Array(vec![
            Value::Map(vec![
                (Value::String("k".into()), Value::Bool(false)),
                (Value::Number(1u64.into()), Value::String("v".into())),
            ]),
            Value::Extension(ExtensionOwned {
                r#type: 1,
                data: vec![0x12, 0x34],
            }),
        ]);
        let mut buf = [0u8; 128];
        let len = to_slice(&v, &mut buf).unwrap();
        let decoded = from_slice::<Value>(&buf[..len]).unwrap();
        assert_eq!(decoded, v);
    }
}

#[cfg(all(test, feature = "std"))]
mod reader_tests {
    use super::*;
    use crate::from_reader;

    #[test]
    fn value_owned_from_reader_str_and_bin() {
        // fixstr "hi"
        let mut r1 = std::io::Cursor::new([0xa2, 0x68, 0x69]);
        let v1: Value = from_reader(&mut r1).unwrap();
        assert_eq!(v1.as_string(), Some("hi"));

        // bin8 [0x01, 0x02, 0x03]
        let mut r2 = std::io::Cursor::new([0xc4, 0x03, 0x01, 0x02, 0x03]);
        let v2: Value = from_reader(&mut r2).unwrap();
        assert_eq!(v2.as_bin(), Some(&[1u8, 2, 3][..]));
    }
}
