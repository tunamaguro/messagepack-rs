use messagepack_core::{Format, encode::ExtensionEncoder, io::IoWrite};
use serde::{
    Deserialize, Serialize, Serializer,
    de::Visitor,
    ser::{self, SerializeTupleVariant},
};

use crate::ser::error::{CoreError, Error};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct ExtensionRef<'a> {
    kind: i8,
    data: &'a [u8],
}

impl<'a> ExtensionRef<'a> {
    pub fn new(kind: i8, data: &'a [u8]) -> Self {
        Self { kind, data }
    }

    pub fn kind(&self) -> i8 {
        self.kind
    }

    pub fn data(&self) -> &[u8] {
        self.data
    }
}

pub(crate) struct SerializeExt<'a, W> {
    writer: &'a mut W,
    length: &'a mut usize,
}

impl<W> AsMut<Self> for SerializeExt<'_, W> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<'a, W> SerializeExt<'a, W> {
    pub fn new(writer: &'a mut W, length: &'a mut usize) -> Self {
        Self { writer, length }
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

    type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeTupleVariant = SerializeExtSeq<'a, W>;

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
        self.writer.write_bytes(v).map_err(CoreError::Io)?;
        *self.length += v.len();
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
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(self.unexpected())
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
        Err(self.unexpected())
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
        Ok(SerializeExtSeq::new(self))
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

pub struct SerializeExtSeq<'a, W> {
    writer: &'a mut W,
    length: &'a mut usize,
}

impl<'a, W> SerializeExtSeq<'a, W> {
    pub(crate) fn new(ser: &'a mut SerializeExt<'_, W>) -> Self {
        Self::from_ref(ser.writer, ser.length)
    }
    pub(crate) fn from_ref(writer: &'a mut W, length: &'a mut usize) -> Self {
        Self { writer, length }
    }
}

impl<W> ser::SerializeSeq for SerializeExtSeq<'_, W>
where
    W: IoWrite,
{
    type Ok = ();
    type Error = Error<W::Error>;
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let mut ser = SerializeExt::new(self.writer, self.length);
        value.serialize(&mut ser)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W> ser::SerializeTupleVariant for SerializeExtSeq<'_, W>
where
    W: IoWrite,
{
    type Ok = ();
    type Error = Error<W::Error>;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

pub const EXTENSION_SER_ENUM_NAME: &str = "ExtensionSer";
pub const EXTENSION_SER_VARIANT_NAME: &str = "Extension";

impl ser::Serialize for ExtensionRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoder = ExtensionEncoder::new(self.kind, self.data);
        let format = encoder
            .to_format::<()>()
            .map_err(|_| ser::Error::custom("Invalid data length"))?;

        let mut seq = serializer.serialize_tuple_variant(
            EXTENSION_SER_ENUM_NAME,
            0,
            EXTENSION_SER_VARIANT_NAME,
            4,
        )?;

        seq.serialize_field(serde_bytes::Bytes::new(&format.as_slice()))?;

        const EMPTY: &[u8] = &[];

        match format {
            messagepack_core::Format::FixExt1 => {
                seq.serialize_field(serde_bytes::Bytes::new(EMPTY))
            }
            messagepack_core::Format::FixExt2 => {
                seq.serialize_field(serde_bytes::Bytes::new(EMPTY))
            }
            messagepack_core::Format::FixExt4 => {
                seq.serialize_field(serde_bytes::Bytes::new(EMPTY))
            }
            messagepack_core::Format::FixExt8 => {
                seq.serialize_field(serde_bytes::Bytes::new(EMPTY))
            }
            messagepack_core::Format::FixExt16 => {
                seq.serialize_field(serde_bytes::Bytes::new(EMPTY))
            }
            messagepack_core::Format::Ext8 => {
                let len = (self.data.len() as u8).to_be_bytes();
                seq.serialize_field(serde_bytes::Bytes::new(&len))
            }
            messagepack_core::Format::Ext16 => {
                let len = (self.data.len() as u16).to_be_bytes();
                seq.serialize_field(serde_bytes::Bytes::new(&len))
            }
            messagepack_core::Format::Ext32 => {
                let len = (self.data.len() as u32).to_be_bytes();
                seq.serialize_field(serde_bytes::Bytes::new(&len))
            }
            _ => unreachable!(),
        }?;
        seq.serialize_field(serde_bytes::Bytes::new(&self.kind.to_be_bytes()))?;
        seq.serialize_field(serde_bytes::Bytes::new(self.data))?;

        seq.end()
    }
}

pub(crate) struct DeserializeExt<'de> {
    data_len: usize,
    pub(crate) input: &'de [u8],
}

impl<'de> AsMut<Self> for DeserializeExt<'de> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

pub const EXTENSION_DER_NAME: &str = "$__MSGPACK_EXTENSION_DER";

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

impl<'de> Deserialize<'de> for ExtensionRef<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ExtensionVisitor;

        impl<'de> Visitor<'de> for ExtensionVisitor {
            type Value = ExtensionRef<'de>;
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

                Ok(ExtensionRef::new(kind, data))
            }
        }
        deserializer.deserialize_any(ExtensionVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use messagepack_core::SliceWriter;
    use rstest::rstest;

    #[rstest]
    fn encode_ext() {
        let mut buf = [0_u8; 3];
        let mut writer = SliceWriter::from_slice(&mut buf);
        let mut length = 0;
        let mut ser = SerializeExt::new(&mut writer, &mut length);

        let kind: i8 = 123;

        let ext = ExtensionRef::new(kind, &[0x12]);

        ext.serialize(&mut ser).unwrap();

        assert_eq!(length, 3);
        assert_eq!(buf, [0xd4, kind.to_be_bytes()[0], 0x12]);
    }
}
