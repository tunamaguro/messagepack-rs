use super::Error;
use super::Serializer;
use super::num::NumEncoder;
use messagepack_core::io::IoWrite;
use serde::ser;

pub struct SerializeMap<'a, 'b, W, Num> {
    ser: &'a mut Serializer<'b, W, Num>,
}

impl<'a, 'b, W, Num> SerializeMap<'a, 'b, W, Num> {
    pub(super) fn new(ser: &'a mut Serializer<'b, W, Num>) -> Self {
        Self { ser }
    }
}

impl<'a, 'b, W, Num> ser::SerializeMap for SerializeMap<'a, 'b, W, Num>
where
    'b: 'a,
    W: IoWrite,
    Num: NumEncoder<W>,
{
    type Ok = ();
    type Error = Error<W::Error>;

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        key.serialize(self.ser.as_mut())
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self.ser.as_mut())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, 'b, W, Num> ser::SerializeStruct for SerializeMap<'a, 'b, W, Num>
where
    'b: 'a,
    W: IoWrite,
    Num: NumEncoder<W>,
{
    type Ok = ();
    type Error = Error<W::Error>;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeMap::serialize_entry(self, key, value)
    }


    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeMap::end(self)
    }
}

impl<'a, 'b, W, Num> ser::SerializeStructVariant for SerializeMap<'a, 'b, W, Num>
where
    'b: 'a,
    W: IoWrite,
    Num: NumEncoder<W>,
{
    type Ok = ();
    type Error = Error<W::Error>;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeMap::serialize_entry(self, key, value)
    }

    #[inline]
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeMap::end(self)
    }
}
