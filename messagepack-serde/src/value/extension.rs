use messagepack_core::{Format, extension::ExtensionRef as CoreExtensionRef, io::IoWrite};
use serde::{
    Deserialize, Serialize, Serializer,
    de::Visitor,
    ser::{self, SerializeSeq},
};

use crate::ser::{CoreError, Error};

pub(crate) const EXTENSION_STRUCT_NAME: &str = "$__MSGPACK_EXTENSION_STRUCT";

pub(crate) struct SerializeExt<'a, W> {
    writer: &'a mut W,
    length: usize,
}

impl<W> AsMut<Self> for SerializeExt<'_, W> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<'a, W> SerializeExt<'a, W> {
    pub fn new(writer: &'a mut W) -> Self {
        Self { writer, length: 0 }
    }

    pub(crate) fn length(&self) -> usize {
        self.length
    }
}

impl<W: IoWrite> SerializeExt<'_, W> {
    fn unexpected(&self) -> Error<W::Error> {
        ser::Error::custom("unexpected value")
    }
}

impl<'a, 'b, W> ser::Serializer for &'a mut SerializeExt<'b, W>
where
    'b: 'a,
    W: IoWrite,
{
    type Ok = ();

    type Error = Error<W::Error>;

    type SerializeSeq = SerializeExtSeq<'a, 'b, W>;

    type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, _: bool) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(&v.to_be_bytes())
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(&v.to_be_bytes())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(&v.to_be_bytes())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(&v.to_be_bytes())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(&v.to_be_bytes())
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.writer.write(v).map_err(CoreError::Io)?;
        self.length += v.len();
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(self.unexpected())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(self.unexpected())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeExtSeq::new(self))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(self.unexpected())
    }
}

pub struct SerializeExtSeq<'a, 'b, W> {
    ser: &'a mut SerializeExt<'b, W>,
}

impl<'a, 'b, W> SerializeExtSeq<'a, 'b, W> {
    pub(crate) fn new(ser: &'a mut SerializeExt<'b, W>) -> Self {
        Self { ser }
    }
}

impl<'a, 'b, W> ser::SerializeSeq for SerializeExtSeq<'a, 'b, W>
where
    'b: 'a,
    W: IoWrite,
{
    type Ok = ();
    type Error = Error<W::Error>;
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self.ser.as_mut())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

struct Bytes<'a>(pub &'a [u8]);
impl ser::Serialize for Bytes<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.0)
    }
}

struct ExtInner<'a> {
    kind: i8,
    data: &'a [u8],
}

impl ser::Serialize for ExtInner<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoder = CoreExtensionRef::new(self.kind, self.data);
        let format = encoder
            .to_format::<core::convert::Infallible>()
            .map_err(|_| ser::Error::custom("Invalid data length"))?;

        let mut seq = serializer.serialize_seq(Some(4))?;

        seq.serialize_element(&Bytes(&format.as_slice()))?;

        match format {
            messagepack_core::Format::FixExt1
            | messagepack_core::Format::FixExt2
            | messagepack_core::Format::FixExt4
            | messagepack_core::Format::FixExt8
            | messagepack_core::Format::FixExt16 => {}

            messagepack_core::Format::Ext8 => {
                let len = (self.data.len() as u8).to_be_bytes();
                seq.serialize_element(&Bytes(&len))?;
            }
            messagepack_core::Format::Ext16 => {
                let len = (self.data.len() as u16).to_be_bytes();
                seq.serialize_element(&Bytes(&len))?;
            }
            messagepack_core::Format::Ext32 => {
                let len = (self.data.len() as u32).to_be_bytes();
                seq.serialize_element(&Bytes(&len))?;
            }
            _ => return Err(ser::Error::custom("unexpected format")),
        };
        seq.serialize_element(&Bytes(&self.kind.to_be_bytes()))?;
        seq.serialize_element(&Bytes(self.data))?;

        seq.end()
    }
}

pub(crate) struct DeserializeExt<'de> {
    data_len: usize,
    pub(crate) input: &'de [u8],
}

impl AsMut<Self> for DeserializeExt<'_> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<'de> DeserializeExt<'de> {
    pub(crate) fn new(format: Format, input: &'de [u8]) -> Result<Self, crate::de::Error> {
        let (data_len, rest) = match format {
            Format::FixExt1 => (1, input),
            Format::FixExt2 => (2, input),
            Format::FixExt4 => (4, input),
            Format::FixExt8 => (8, input),
            Format::FixExt16 => (16, input),
            Format::Ext8 => {
                let (first, rest) = input
                    .split_first_chunk::<1>()
                    .ok_or(messagepack_core::decode::Error::EofData)?;
                let val = u8::from_be_bytes(*first);
                (val.into(), rest)
            }
            Format::Ext16 => {
                let (first, rest) = input
                    .split_first_chunk::<2>()
                    .ok_or(messagepack_core::decode::Error::EofData)?;
                let val = u16::from_be_bytes(*first);
                (val.into(), rest)
            }
            Format::Ext32 => {
                let (first, rest) = input
                    .split_first_chunk::<4>()
                    .ok_or(messagepack_core::decode::Error::EofData)?;
                let val = u32::from_be_bytes(*first);
                (val as usize, rest)
            }
            _ => return Err(messagepack_core::decode::Error::UnexpectedFormat.into()),
        };
        Ok(DeserializeExt {
            data_len,
            input: rest,
        })
    }
}

impl<'de> serde::Deserializer<'de> for &mut DeserializeExt<'de> {
    type Error = crate::de::Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(crate::de::Error::AnyIsUnsupported)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (first, rest) = self
            .input
            .split_first_chunk::<1>()
            .ok_or(messagepack_core::decode::Error::EofData)?;

        let val = i8::from_be_bytes(*first);
        self.input = rest;
        visitor.visit_i8(val)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (data, rest) = self
            .input
            .split_at_checked(self.data_len)
            .ok_or(messagepack_core::decode::Error::EofData)?;
        self.input = rest;
        visitor.visit_borrowed_bytes(data)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(&mut self)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    serde::forward_to_deserialize_any! {
        bool i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        byte_buf option unit unit_struct tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> serde::de::SeqAccess<'de> for &mut DeserializeExt<'de> {
    type Error = crate::de::Error;
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.as_mut()).map(Some)
    }
}

pub mod ext_ref {
    use super::*;

    pub fn serialize<S>(
        ext: &messagepack_core::extension::ExtensionRef<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct(
            EXTENSION_STRUCT_NAME,
            &ExtInner {
                kind: ext.r#type,
                data: ext.data,
            },
        )
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<messagepack_core::extension::ExtensionRef<'de>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ExtensionVisitor;

        impl<'de> Visitor<'de> for ExtensionVisitor {
            type Value = messagepack_core::extension::ExtensionRef<'de>;
            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("expect extension")
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_seq(self)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let kind = seq
                    .next_element::<i8>()?
                    .ok_or(serde::de::Error::custom("expect i8"))?;

                let data = seq
                    .next_element::<&[u8]>()?
                    .ok_or(serde::de::Error::custom("expect [u8]"))?;

                Ok(messagepack_core::extension::ExtensionRef::new(kind, data))
            }
        }
        deserializer.deserialize_any(ExtensionVisitor)
    }
}

pub mod ext_fixed {
    use serde::de;

    pub fn serialize<const N: usize, S>(
        ext: &messagepack_core::extension::FixedExtension<N>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        super::ext_ref::serialize(&ext.as_ref(), serializer)
    }

    pub fn deserialize<'de, const N: usize, D>(
        deserializer: D,
    ) -> Result<messagepack_core::extension::FixedExtension<N>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let r = super::ext_ref::deserialize(deserializer)?;

        let ext = messagepack_core::extension::FixedExtension::new(r.r#type, r.data)
            .ok_or_else(|| de::Error::custom("extension length is too long"))?;
        Ok(ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use messagepack_core::extension::{ExtensionRef, FixedExtension};
    use rstest::rstest;

    #[derive(Debug, Serialize, Deserialize)]
    struct WrapRef<'a>(
        #[serde(with = "ext_ref", borrow)] messagepack_core::extension::ExtensionRef<'a>,
    );

    #[rstest]
    fn encode_ext_ref() {
        let mut buf = [0_u8; 3];

        let kind: i8 = 123;

        let ext = WrapRef(ExtensionRef::new(kind, &[0x12]));
        let length = crate::to_slice(&ext, &mut buf).unwrap();

        assert_eq!(length, 3);
        assert_eq!(buf, [0xd4, kind.to_be_bytes()[0], 0x12]);
    }

    #[rstest]
    fn decode_ext_ref() {
        let buf = [0xd6, 0xff, 0x00, 0x00, 0x00, 0x00]; // timestamp ext type

        let ext = crate::from_slice::<WrapRef<'_>>(&buf).unwrap().0;
        assert_eq!(ext.r#type, -1);
        let seconds = u32::from_be_bytes(ext.data.try_into().unwrap());
        assert_eq!(seconds, 0);
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct WrapFixed<const N: usize>(
        #[serde(with = "ext_fixed")] messagepack_core::extension::FixedExtension<N>,
    );

    #[rstest]
    fn encode_ext_fixed() {
        let mut buf = [0u8; 3];
        let kind: i8 = 123;

        let ext = WrapFixed(FixedExtension::new_fixed(kind, [0x12]));
        let length = crate::to_slice(&ext, &mut buf).unwrap();

        assert_eq!(length, 3);
        assert_eq!(buf, [0xd4, kind.to_be_bytes()[0], 0x12]);
    }

    const TIMESTAMP32: &[u8] = &[0xd6, 0xff, 0x00, 0x00, 0x00, 0x00];

    #[rstest]
    fn decode_ext_fixed_bigger_will_success() {
        let ext = crate::from_slice::<WrapFixed<6>>(TIMESTAMP32).unwrap().0;
        assert_eq!(ext.r#type, -1);
        assert_eq!(ext.data(), &TIMESTAMP32[2..])
    }

    #[rstest]
    #[should_panic]
    fn decode_ext_fixed_smaller_will_failed() {
        let _ = crate::from_slice::<WrapFixed<3>>(TIMESTAMP32).unwrap();
    }
}
