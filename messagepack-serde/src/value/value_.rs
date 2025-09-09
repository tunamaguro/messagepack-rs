use super::number::Number;
use alloc::vec::Vec;
use messagepack_core::extension::ExtensionRef;
use serde::{de::Visitor, ser::SerializeMap};

/// Represents any messagepack value. `alloc` needed.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ValueRef<'a> {
    /// Represents nil format
    Nil,
    /// Represents bool format family
    Bool(bool),
    /// Represents `bin 8`, `bin 16` and `bin 32`
    Bin(&'a [u8]),
    /// Represents ext format family
    Extension(ExtensionRef<'a>),
    /// Represents int format family and float format family
    Number(Number),
    /// Represents str format family
    String(&'a str),
    /// Represents array format family
    Array(Vec<ValueRef<'a>>),
    /// Represents map format family
    Map(Vec<(ValueRef<'a>, ValueRef<'a>)>),
}

impl ValueRef<'_> {
    /// Returns true if the `ValueRef` is nil
    pub fn is_nil(&self) -> bool {
        matches!(self, ValueRef::Nil)
    }

    /// If the `ValueRef` is boolean, returns contained value.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ValueRef::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// If the `ValueRef` is bin, returns contained value.
    pub fn as_bin(&self) -> Option<&[u8]> {
        match self {
            ValueRef::Bin(v) => Some(*v),
            _ => None,
        }
    }

    /// If the `ValueRef` is ext, returns contained value.
    pub fn as_extension(&self) -> Option<&ExtensionRef<'_>> {
        match self {
            ValueRef::Extension(v) => Some(v),
            _ => None,
        }
    }

    /// If the `ValueRef` is number, returns contained value.
    pub fn as_number(&self) -> Option<Number> {
        match self {
            ValueRef::Number(v) => Some(*v),
            _ => None,
        }
    }

    /// If the `ValueRef` is str, returns contained value.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ValueRef::String(v) => Some(*v),
            _ => None,
        }
    }

    /// If the `ValueRef` is array, returns contained value.
    pub fn as_array(&self) -> Option<&[ValueRef<'_>]> {
        match self {
            ValueRef::Array(v) => Some(v),
            _ => None,
        }
    }

    /// If the `ValueRef` is map, returns contained value.
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
            ValueRef::Bin(items) => serializer.serialize_bytes(*items),
            ValueRef::Extension(extension_ref) => {
                super::ext_ref::serialize(extension_ref, serializer)
            }
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
                let n = Number::from(v);
                Ok(ValueRef::Number(n))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let n = Number::from(v);
                Ok(ValueRef::Number(n))
            }

            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let n = Number::Float(v.into());
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

            fn visit_unit<E>(self) -> Result<Self::Value, E>
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
                let ext = super::ext_ref::deserialize(deserializer)?;
                Ok(ValueRef::Extension(ext))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut buf = Vec::new();
                if let Some(size) = seq.size_hint() {
                    buf.reserve(size);
                }

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
                if let Some(size) = map.size_hint() {
                    buf.reserve(size);
                }

                while let Some(v) = map.next_entry()? {
                    buf.push(v);
                }

                Ok(ValueRef::Map(buf))
            }
        }
        deserializer.deserialize_any(ValueVisitor)
    }
}

impl From<()> for ValueRef<'_> {
    fn from(_: ()) -> Self {
        ValueRef::Nil
    }
}

impl From<bool> for ValueRef<'_> {
    fn from(v: bool) -> Self {
        ValueRef::Bool(v)
    }
}

impl From<u8> for ValueRef<'_> {
    fn from(v: u8) -> Self {
        ValueRef::Number(Number::from(v))
    }
}

impl From<u16> for ValueRef<'_> {
    fn from(v: u16) -> Self {
        ValueRef::Number(Number::from(v))
    }
}

impl From<u32> for ValueRef<'_> {
    fn from(v: u32) -> Self {
        ValueRef::Number(Number::from(v))
    }
}

impl From<u64> for ValueRef<'_> {
    fn from(v: u64) -> Self {
        ValueRef::Number(Number::from(v))
    }
}

impl From<i8> for ValueRef<'_> {
    fn from(v: i8) -> Self {
        ValueRef::Number(Number::from(v))
    }
}

impl From<i16> for ValueRef<'_> {
    fn from(v: i16) -> Self {
        ValueRef::Number(Number::from(v))
    }
}

impl From<i32> for ValueRef<'_> {
    fn from(v: i32) -> Self {
        ValueRef::Number(Number::from(v))
    }
}

impl From<i64> for ValueRef<'_> {
    fn from(v: i64) -> Self {
        ValueRef::Number(Number::from(v))
    }
}

impl From<f32> for ValueRef<'_> {
    fn from(v: f32) -> Self {
        ValueRef::Number(Number::from(v))
    }
}

impl From<f64> for ValueRef<'_> {
    fn from(v: f64) -> Self {
        ValueRef::Number(Number::from(v))
    }
}

impl From<Number> for ValueRef<'_> {
    fn from(v: Number) -> Self {
        ValueRef::Number(v)
    }
}

impl<'a> From<&'a str> for ValueRef<'a> {
    fn from(v: &'a str) -> Self {
        ValueRef::String(v)
    }
}

impl<'a> From<&'a [u8]> for ValueRef<'a> {
    fn from(v: &'a [u8]) -> Self {
        ValueRef::Bin(v)
    }
}
impl<'a> From<ExtensionRef<'a>> for ValueRef<'a> {
    fn from(v: ExtensionRef<'a>) -> Self {
        ValueRef::Extension(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_slice, to_slice};
    use messagepack_core::extension::ExtensionRef;
    use rstest::rstest;

    // Verify serialization of ValueRef scalars and simple composites using rstest.
    #[rstest]
    #[case(ValueRef::Nil, vec![0xc0])]
    #[case(ValueRef::Bool(true), vec![0xc3])]
    #[case(ValueRef::Number(Number::PositiveInt(5)), vec![0x05])]
    // -33 encoded as int8: 0xd0, 0xdf
    #[case(ValueRef::Number(Number::NegativeInt(-33)), vec![0xd0, 0xdf])]
    // 1.5 can be represented as f32 => 0xca 3f c0 00 00
    #[case(ValueRef::Number(Number::Float(1.5)), vec![0xca, 0x3f, 0xc0, 0x00, 0x00])]
    #[case(ValueRef::String("a"), vec![0xa1, b'a'])]
    // Bin encodes as MessagePack bin8 here
    #[case(ValueRef::Bin(&[0x01, 0x02]), vec![0xc4, 0x02, 0x01, 0x02])]
    #[case(
        ValueRef::Array(vec![ValueRef::Bool(true), ValueRef::Nil]),
        vec![0x92, 0xc3, 0xc0]
    )]
    #[case(
        ValueRef::Map(vec![
            (ValueRef::String("a"), ValueRef::Number(Number::NegativeInt(-1)))
        ]),
        vec![0x81, 0xa1, b'a', 0xff]
    )]
    fn encode_value_ref_cases(#[case] v: ValueRef<'_>, #[case] expected: Vec<u8>) {
        let mut buf = vec![0u8; expected.len() + 8];
        let len = to_slice(&v, &mut buf).unwrap();
        assert_eq!(buf[..len], expected);
    }

    // Verify deserialization of ValueRef scalars and simple composites.
    #[rstest]
    #[case(&[0xc0], ValueRef::Nil)]
    #[case(&[0xc3], ValueRef::Bool(true))]
    #[case(&[0x05], ValueRef::Number(Number::PositiveInt(5)))]
    #[case(&[0xd0, 0xdf], ValueRef::Number(Number::NegativeInt(-33)))]
    #[case(&[0xca, 0x3f, 0xc0, 0x00, 0x00], ValueRef::Number(Number::Float(1.5)))]
    #[case(&[0xa1, b'a'], ValueRef::String("a"))]
    #[case(&[0xc4, 0x02, 0x01, 0x02], ValueRef::Bin(&[0x01, 0x02]))]
    #[case(&[0x92, 0xc3, 0xc0], ValueRef::Array(vec![ValueRef::Bool(true), ValueRef::Nil]))]
    #[case(
        &[0x81, 0xa1, b'a', 0xff],
        ValueRef::Map(vec![
            (ValueRef::String("a"), ValueRef::Number(Number::NegativeInt(-1)))
        ])
    )]
    fn decode_value_ref_cases(#[case] input: &[u8], #[case] expected: ValueRef<'_>) {
        let v = from_slice::<ValueRef<'_>>(input).unwrap();
        assert_eq!(v, expected);
    }

    // Verify extension encoding/decoding via ValueRef::Extension.
    #[test]
    fn encode_value_ref_extension_fixext1() {
        let kind: i8 = 10;
        let v = ValueRef::Extension(ExtensionRef::new(kind, &[0x12]));
        let mut buf = [0u8; 3];
        let len = to_slice(&v, &mut buf).unwrap();
        assert_eq!(len, 3);
        assert_eq!(buf, [0xd4, kind as u8, 0x12]);
    }

    // Round-trip timestamp 32 (ext type -1, 4-byte seconds field).
    #[test]
    fn decode_then_reencode_value_ref_extension_timestamp32_roundtrip() {
        let ts32: &[u8] = &[0xd6, 0xff, 0x00, 0x00, 0x00, 0x00];
        let v = from_slice::<ValueRef<'_>>(ts32).unwrap();
        match v {
            ValueRef::Extension(ext) => {
                assert_eq!(ext.r#type, -1);
                assert_eq!(ext.data, &[0x00, 0x00, 0x00, 0x00]);

                // Re-encode and compare with original bytes
                let mut buf = [0u8; 6];
                let len = to_slice(&ValueRef::Extension(ext), &mut buf).unwrap();
                assert_eq!(&buf[..len], ts32);
            }
            _ => panic!("expected extension"),
        }
    }

    // {
    //   "meta": {
    //     "id": 1001,
    //     "tags": ["sample", null, 42, {"extra": "yes"}]
    //   },
    //   "users": [
    //     {
    //       "name": "Alice",
    //       "attributes": {
    //         "age": 29,
    //         "preferences": [
    //           "coffee",
    //           null,
    //           {"music": ["jazz", "rock", {"genres": ["classical", 123]}]}
    //         ]
    //       }
    //     },
    //     {
    //       "name": "Bob",
    //       "attributes": {
    //         "age": null,
    //         "preferences": [
    //           {"food": ["pizza", "sushi", null]},
    //           [true, false, {"nested": [0, {"inner": "value"}]}]
    //         ]
    //       }
    //     }
    //   ]
    // }
    const COMPLEX: &[u8] = &[
        0x82, 0xa4, 0x6d, 0x65, 0x74, 0x61, 0x82, 0xa2, 0x69, 0x64, 0xcd, 0x03, 0xe9, 0xa4, 0x74,
        0x61, 0x67, 0x73, 0x94, 0xa6, 0x73, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0xc0, 0x2a, 0x81, 0xa5,
        0x65, 0x78, 0x74, 0x72, 0x61, 0xa3, 0x79, 0x65, 0x73, 0xa5, 0x75, 0x73, 0x65, 0x72, 0x73,
        0x92, 0x82, 0xa4, 0x6e, 0x61, 0x6d, 0x65, 0xa5, 0x41, 0x6c, 0x69, 0x63, 0x65, 0xaa, 0x61,
        0x74, 0x74, 0x72, 0x69, 0x62, 0x75, 0x74, 0x65, 0x73, 0x82, 0xa3, 0x61, 0x67, 0x65, 0x1d,
        0xab, 0x70, 0x72, 0x65, 0x66, 0x65, 0x72, 0x65, 0x6e, 0x63, 0x65, 0x73, 0x93, 0xa6, 0x63,
        0x6f, 0x66, 0x66, 0x65, 0x65, 0xc0, 0x81, 0xa5, 0x6d, 0x75, 0x73, 0x69, 0x63, 0x93, 0xa4,
        0x6a, 0x61, 0x7a, 0x7a, 0xa4, 0x72, 0x6f, 0x63, 0x6b, 0x81, 0xa6, 0x67, 0x65, 0x6e, 0x72,
        0x65, 0x73, 0x92, 0xa9, 0x63, 0x6c, 0x61, 0x73, 0x73, 0x69, 0x63, 0x61, 0x6c, 0x7b, 0x82,
        0xa4, 0x6e, 0x61, 0x6d, 0x65, 0xa3, 0x42, 0x6f, 0x62, 0xaa, 0x61, 0x74, 0x74, 0x72, 0x69,
        0x62, 0x75, 0x74, 0x65, 0x73, 0x82, 0xa3, 0x61, 0x67, 0x65, 0xc0, 0xab, 0x70, 0x72, 0x65,
        0x66, 0x65, 0x72, 0x65, 0x6e, 0x63, 0x65, 0x73, 0x92, 0x81, 0xa4, 0x66, 0x6f, 0x6f, 0x64,
        0x93, 0xa5, 0x70, 0x69, 0x7a, 0x7a, 0x61, 0xa5, 0x73, 0x75, 0x73, 0x68, 0x69, 0xc0, 0x93,
        0xc3, 0xc2, 0x81, 0xa6, 0x6e, 0x65, 0x73, 0x74, 0x65, 0x64, 0x92, 0x00, 0x81, 0xa5, 0x69,
        0x6e, 0x6e, 0x65, 0x72, 0xa5, 0x76, 0x61, 0x6c, 0x75, 0x65,
    ];
    #[test]
    fn roundtrip_complex() {
        let meta = ValueRef::Map(vec![
            (ValueRef::String("id"), ValueRef::from(1001)),
            (
                ValueRef::String("tags"),
                ValueRef::Array(vec![
                    ValueRef::String("sample"),
                    ValueRef::Nil,
                    ValueRef::from(42),
                    ValueRef::Map(vec![(ValueRef::String("extra"), ValueRef::String("yes"))]),
                ]),
            ),
        ]);
        let alice = ValueRef::Map(vec![
            (ValueRef::String("name"), ValueRef::String("Alice")),
            (
                ValueRef::String("attributes"),
                ValueRef::Map(vec![
                    (ValueRef::String("age"), ValueRef::from(29)),
                    (
                        ValueRef::String("preferences"),
                        ValueRef::Array(vec![
                            ValueRef::String("coffee"),
                            ValueRef::Nil,
                            ValueRef::Map(vec![(
                                ValueRef::String("music"),
                                ValueRef::Array(vec![
                                    ValueRef::String("jazz"),
                                    ValueRef::String("rock"),
                                    ValueRef::Map(vec![(
                                        ValueRef::String("genres"),
                                        ValueRef::Array(vec![
                                            ValueRef::String("classical"),
                                            ValueRef::from(123),
                                        ]),
                                    )]),
                                ]),
                            )]),
                        ]),
                    ),
                ]),
            ),
        ]);
        let bob = ValueRef::Map(vec![
            (ValueRef::String("name"), ValueRef::String("Bob")),
            (
                ValueRef::String("attributes"),
                ValueRef::Map(vec![
                    (ValueRef::String("age"), ValueRef::Nil),
                    (
                        ValueRef::String("preferences"),
                        ValueRef::Array(vec![
                            ValueRef::Map(vec![(
                                ValueRef::String("food"),
                                ValueRef::Array(vec![
                                    ValueRef::String("pizza"),
                                    ValueRef::String("sushi"),
                                    ValueRef::Nil,
                                ]),
                            )]),
                            ValueRef::Array(vec![
                                ValueRef::Bool(true),
                                ValueRef::Bool(false),
                                ValueRef::Map(vec![(
                                    ValueRef::String("nested"),
                                    ValueRef::Array(vec![
                                        ValueRef::from(0),
                                        ValueRef::Map(vec![(
                                            ValueRef::String("inner"),
                                            ValueRef::String("value"),
                                        )]),
                                    ]),
                                )]),
                            ]),
                        ]),
                    ),
                ]),
            ),
        ]);
        let v = ValueRef::Map(vec![
            (ValueRef::String("meta"), meta),
            (ValueRef::String("users"), ValueRef::Array(vec![alice, bob])),
        ]);

        let deserialized = from_slice::<ValueRef<'_>>(COMPLEX).unwrap();
        assert_eq!(deserialized, v);

        let mut buf = [0u8; COMPLEX.len()];
        let len = to_slice(&v, &mut buf).unwrap();
        assert_eq!(len, COMPLEX.len());
        assert_eq!(&buf, COMPLEX);
    }
}
