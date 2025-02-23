use super::Serializer;
use serde::ser;

use super::error::Error;

pub struct SerializeSeq<'a, 'b, Buf> {
    len: Option<usize>,
    ser: &'a mut Serializer<'b, Buf>,
}

impl<'a, 'b, Buf> SerializeSeq<'a, 'b, Buf> {
    pub(crate) fn new(len: Option<usize>, ser: &'a mut Serializer<'b, Buf>) -> Self {
        Self { len, ser }
    }
}

impl<'b, Buf> ser::SerializeSeq for SerializeSeq<'_, 'b, Buf>
where
    Buf: Iterator<Item = &'b mut u8>,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self.ser.as_mut())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'b, Buf> ser::SerializeTuple for SerializeSeq<'_, 'b, Buf>
where
    Buf: Iterator<Item = &'b mut u8>,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self.ser.as_mut())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'b, Buf> ser::SerializeTupleStruct for SerializeSeq<'_, 'b, Buf>
where
    Buf: Iterator<Item = &'b mut u8>,
{
    type Ok = ();
    type Error = Error;
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self.ser.as_mut())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'b, Buf> ser::SerializeTupleVariant for SerializeSeq<'_, 'b, Buf>
where
    Buf: Iterator<Item = &'b mut u8>,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}
