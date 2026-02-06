use super::{Serializer, num::NumEncoder};
use messagepack_core::io::IoWrite;
use serde::ser;

use super::error::Error;

pub struct SerializeSeq<'a, 'b, W, Num> {
    ser: &'a mut Serializer<'b, W, Num>,
}

impl<'a, 'b, W, Num> SerializeSeq<'a, 'b, W, Num> {
    pub(super) fn new(ser: &'a mut Serializer<'b, W, Num>) -> Self {
        Self { ser }
    }
}

impl<'a, 'b, W, Num> ser::SerializeSeq for SerializeSeq<'a, 'b, W, Num>
where
    'b: 'a,
    W: IoWrite,
    Num: NumEncoder<W>,
{
    type Ok = ();
    type Error = Error<W::Error>;

    #[inline]
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

impl<'a, 'b, W, Num> ser::SerializeTuple for SerializeSeq<'a, 'b, W, Num>
where
    'b: 'a,
    W: IoWrite,
    Num: NumEncoder<W>,
{
    type Ok = ();
    type Error = Error<W::Error>;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, 'b, W, Num> ser::SerializeTupleStruct for SerializeSeq<'a, 'b, W, Num>
where
    'b: 'a,
    W: IoWrite,
    Num: NumEncoder<W>,
{
    type Ok = ();
    type Error = Error<W::Error>;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, 'b, W, Num> ser::SerializeTupleVariant for SerializeSeq<'a, 'b, W, Num>
where
    'b: 'a,
    W: IoWrite,
    Num: NumEncoder<W>,
{
    type Ok = ();
    type Error = Error<W::Error>;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}
