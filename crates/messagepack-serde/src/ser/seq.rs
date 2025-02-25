use super::Serializer;
use messagepack_core::io::IoWrite;
use serde::ser;

use super::error::Error;

pub struct SerializeSeq<'a, 'b, W> {
    ser: &'a mut Serializer<'b, W>,
}

impl<'a, 'b, W> SerializeSeq<'a, 'b, W> {
    pub(crate) fn new(ser: &'a mut Serializer<'b, W>) -> Self {
        Self { ser }
    }
}

impl<'b, W> ser::SerializeSeq for SerializeSeq<'_, 'b, W>
where
    W: IoWrite,
{
    type Ok = ();
    type Error = Error<W::Error>;

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

impl<'b, W> ser::SerializeTuple for SerializeSeq<'_, 'b, W>
where
    W: IoWrite,
{
    type Ok = ();
    type Error = Error<W::Error>;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<'b, W> ser::SerializeTupleStruct for SerializeSeq<'_, 'b, W>
where
    W: IoWrite,
{
    type Ok = ();
    type Error = Error<W::Error>;
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<'b, W> ser::SerializeTupleVariant for SerializeSeq<'_, 'b, W>
where
    W: IoWrite,
{
    type Ok = ();
    type Error = Error<W::Error>;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}
