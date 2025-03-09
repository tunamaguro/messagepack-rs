use extension::ExtensionRef;
use serde::ser::SerializeMap;

pub mod extension;
pub mod number;

pub enum ValueRef<'a> {
    Nil,
    Bool(bool),
    Bin(&'a [u8]),
    Extension(ExtensionRef<'a>),
    Number(number::Number),
    String(&'a str),
    Array(&'a [ValueRef<'a>]),
    Map(&'a [(ValueRef<'a>, ValueRef<'a>)]),
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
                for (k, v) in *items {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}
