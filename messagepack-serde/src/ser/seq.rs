use super::{Error, Serializer, num::NumEncoder};
use messagepack_core::{Encode as _, encode::array::ArrayFormatEncoder, io::IoWrite};
use serde::ser;

pub(super) enum SerializeSeq<'a, 'b, W, Num> {
    SeqWithLen {
        ser: &'a mut Serializer<'b, W, Num>,
    },
    #[cfg(feature = "alloc")]
    SeqWithOutLen {
        ser: &'a mut Serializer<'b, W, Num>,
        array_values: alloc::vec::Vec<crate::value::Value>,
    },
}

impl<'a, 'b, W, Num> SerializeSeq<'a, 'b, W, Num>
where
    'b: 'a,
    W: IoWrite,
    Num: NumEncoder<W>,
{
    pub fn new(
        ser: &'a mut Serializer<'b, W, Num>,
        len: Option<usize>,
    ) -> Result<Self, Error<W::Error>> {
        if let Some(len) = len {
            ser.current_length += ArrayFormatEncoder(len).encode(ser.writer)?;
            Ok(Self::SeqWithLen { ser })
        } else {
            #[cfg(feature = "alloc")]
            {
                Ok(Self::SeqWithOutLen {
                    ser,
                    array_values: alloc::vec::Vec::new(),
                })
            }

            #[cfg(not(feature = "alloc"))]
            {
                Err(Error::SeqLenNone)
            }
        }
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

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        match self {
            Self::SeqWithLen { ser, .. } => value.serialize(ser.as_mut()),
            #[cfg(feature = "alloc")]
            Self::SeqWithOutLen { array_values, .. } => {
                let val = crate::value::to_value(value).map_err(crate::ser::error::convert_error)?;
                array_values.push(val);
                Ok(())
            }
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::SeqWithLen { .. } => Ok(()),
            #[cfg(feature = "alloc")]
            Self::SeqWithOutLen { ser, array_values } => {
                use serde::Serialize;
                let array = crate::value::Value::Array(array_values);
                array.serialize(ser.as_mut())?;
                Ok(())
            }
        }
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

impl<'a, 'b, W, Num> ser::SerializeTupleStruct for SerializeSeq<'a, 'b, W, Num>
where
    'b: 'a,
    W: IoWrite,
    Num: NumEncoder<W>,
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

impl<'a, 'b, W, Num> ser::SerializeTupleVariant for SerializeSeq<'a, 'b, W, Num>
where
    'b: 'a,
    W: IoWrite,
    Num: NumEncoder<W>,
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
