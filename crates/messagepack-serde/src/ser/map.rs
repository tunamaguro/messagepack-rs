use super::Error;
use super::Serializer;
use messagepack_core::io::IoWrite;
use serde::ser;

pub struct SerializeMap<'a, 'b, W> {
    ser: &'a mut Serializer<'b, W>,
}

impl<'a, 'b, W> SerializeMap<'a, 'b, W> {
    pub(crate) fn new(ser: &'a mut Serializer<'b, W>) -> Self {
        Self { ser }
    }
}

impl<'a, 'b, W> ser::SerializeMap for SerializeMap<'a, 'b, W>
where
    'b: 'a,
    W: IoWrite,
{
    type Ok = ();
    type Error = Error<W::Error>;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        key.serialize(self.ser.as_mut())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self.ser.as_mut())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, 'b, W> ser::SerializeStruct for SerializeMap<'a, 'b, W>
where
    'b: 'a,
    W: IoWrite,
{
    type Ok = ();
    type Error = Error<W::Error>;

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

impl<'a, 'b, W> ser::SerializeStructVariant for SerializeMap<'a, 'b, W>
where
    'b: 'a,
    W: IoWrite,
{
    type Ok = ();
    type Error = Error<W::Error>;

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
