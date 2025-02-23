use super::Serializer;
use serde::ser;

use super::error::{CoreError, Error};

pub struct SerializeSeq<'a, 'b, Buf> {
    len: Option<usize>,
    ser: &'a mut Serializer<'b, Buf>,
}

impl<'a, 'b, Buf> SerializeSeq<'a, 'b, Buf> {
    pub(crate) fn new(len: Option<usize>, ser: &'a mut Serializer<'b, Buf>) -> Self {
        Self { len, ser }
    }
}

impl<'a, 'b, Buf> ser::SerializeSeq for SerializeSeq<'a, 'b, Buf>
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

impl<'a, 'b, Buf> ser::SerializeTuple for SerializeSeq<'a, 'b, Buf>
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

impl<'a, 'b, Buf> ser::SerializeTupleStruct for SerializeSeq<'a, 'b, Buf>
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

impl<'a, 'b, Buf> ser::SerializeTupleVariant for SerializeSeq<'a, 'b, Buf>
where
    Buf: Iterator<Item = &'b mut u8>,
{
    type Ok = ();
    type Error = Error;
    
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize {
        todo!()
    }
    
    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}
