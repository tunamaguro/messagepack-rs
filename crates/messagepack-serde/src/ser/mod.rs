use core::marker::PhantomData;

use error::{CoreError, Error};
use messagepack_core::{
    Encode,
    encode::{ArrayFormatEncoder, BinaryEncoder, MapEncoder, MapFormatEncoder, NilEncoder},
};
use serde::ser;

pub mod error;
mod map;
mod seq;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct Serializer<'a, Buf> {
    buf: Buf,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, Buf> Serializer<'a, Buf>
where
    Buf: Iterator<Item = &'a mut u8>,
{
    pub fn new(buf: Buf) -> Self {
        Self {
            buf,
            _phantom: Default::default(),
        }
    }
}

impl<Buf> AsMut<Self> for Serializer<'_, Buf> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<'a, Buf> Serializer<'a, Buf>
where
    Buf: Iterator<Item = &'a mut u8>,
{
    pub(crate) fn take_byte(&mut self) -> Result<&'a mut u8, Error> {
        let b = self.buf.next().ok_or(CoreError::BufferFull)?;
        Ok(b)
    }
}

impl<'a, 'b: 'a, Buf> ser::Serializer for &'a mut Serializer<'b, Buf>
where
    Buf: Iterator<Item = &'b mut u8>,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = seq::SerializeSeq<'a, 'b, Buf>;
    type SerializeTuple = seq::SerializeSeq<'a, 'b, Buf>;
    type SerializeTupleStruct = seq::SerializeSeq<'a, 'b, Buf>;
    type SerializeTupleVariant = seq::SerializeSeq<'a, 'b, Buf>;
    type SerializeMap = map::SerializeMap<'a, 'b, Buf>;
    type SerializeStruct = map::SerializeMap<'a, 'b, Buf>;
    type SerializeStructVariant = map::SerializeMap<'a, 'b, Buf>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        v.encode_to_iter_mut(self.buf.by_ref())?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        v.encode_to_iter_mut(self.buf.by_ref())?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        v.encode_to_iter_mut(self.buf.by_ref())?;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        v.encode_to_iter_mut(self.buf.by_ref())?;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        v.encode_to_iter_mut(self.buf.by_ref())?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        v.encode_to_iter_mut(self.buf.by_ref())?;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        v.encode_to_iter_mut(self.buf.by_ref())?;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        v.encode_to_iter_mut(self.buf.by_ref())?;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        v.encode_to_iter_mut(self.buf.by_ref())?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        v.encode_to_iter_mut(self.buf.by_ref())?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        v.encode_to_iter_mut(self.buf.by_ref())?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        v.encode_to_iter_mut(self.buf.by_ref())?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        BinaryEncoder(v).encode_to_iter_mut(self.buf.by_ref())?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        NilEncoder.encode_to_iter_mut(self.buf.by_ref())?;
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(name)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self.as_mut())
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        MapFormatEncoder::new(1).encode_to_iter_mut(self.buf.by_ref())?;
        self.serialize_str(variant)?;
        value.serialize(self.as_mut())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let len = len.ok_or(Error::SeqLenNone)?;
        ArrayFormatEncoder::new(len).encode_to_iter_mut(self.buf.by_ref())?;
        Ok(seq::SerializeSeq::new(len.into(), self))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        ArrayFormatEncoder::new(len).encode_to_iter_mut(self.buf.by_ref())?;
        Ok(seq::SerializeSeq::new(len.into(), self))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        ArrayFormatEncoder::new(len).encode_to_iter_mut(self.buf.by_ref())?;
        Ok(seq::SerializeSeq::new(len.into(), self))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        MapFormatEncoder::new(1).encode_to_iter_mut(self.buf.by_ref())?;
        self.serialize_str(variant)?;
        ArrayFormatEncoder::new(len).encode_to_iter_mut(self.buf.by_ref())?;
        Ok(seq::SerializeSeq::new(len.into(), self))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let len = len.ok_or(Error::SeqLenNone)?;
        MapFormatEncoder::new(len).encode_to_iter_mut(self.buf.by_ref())?;
        Ok(map::SerializeMap::new(len.into(), self))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        MapFormatEncoder::new(len).encode_to_iter_mut(self.buf.by_ref())?;
        Ok(map::SerializeMap::new(len.into(), self))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        MapFormatEncoder::new(1).encode_to_iter_mut(self.buf.by_ref())?;
        self.serialize_str(variant)?;
        self.serialize_struct(name, len)
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + core::fmt::Display,
    {
        #[cfg(not(feature = "std"))]
        {
            unreachable!()
        }
        #[cfg(feature = "std")]
        {
            let s = value.to_string();
            self.serialize_str(&s)
        }
    }
}
