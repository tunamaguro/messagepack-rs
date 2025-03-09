use messagepack_core::{encode::ExtensionEncoder, io::IoWrite};
use serde::{
    Serialize, Serializer,
    ser::{self, SerializeTupleVariant},
};

use crate::ser::error::{CoreError, Error};

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

pub struct ExtensionRef<'a> {
    kind: i8,
    data: &'a [u8],
}

impl<'a> ExtensionRef<'a> {
    pub fn new(kind: i8, data: &'a [u8]) -> Self {
        Self { kind, data }
    }
}

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
