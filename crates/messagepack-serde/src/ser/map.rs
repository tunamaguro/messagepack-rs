use super::Error;
use super::Serializer;
use serde::ser;

pub struct SerializeMap<'a, 'b, Buf> {
    len: Option<usize>,
    ser: &'a mut Serializer<'b, Buf>,
}

impl<'a, 'b, Buf> SerializeMap<'a, 'b, Buf> {
    pub(crate) fn new(len: Option<usize>, ser: &'a mut Serializer<'b, Buf>) -> Self {
        Self { len, ser }
    }
}

impl<'a, 'b, Buf> ser::SerializeMap for SerializeMap<'a, 'b, Buf>
where
    Buf: Iterator<Item = &'b mut u8>,
{
    type Ok = ();
    type Error = Error;

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

impl<'a, 'b, Buf> ser::SerializeStruct for SerializeMap<'a, 'b, Buf>
where
    Buf: Iterator<Item = &'b mut u8>,
{
    type Ok = ();
    type Error = Error;

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

impl<'a, 'b, Buf> ser::SerializeStructVariant for SerializeMap<'a, 'b, Buf>
where
    Buf: Iterator<Item = &'b mut u8>,
{
    type Ok = ();
    type Error = Error;

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
