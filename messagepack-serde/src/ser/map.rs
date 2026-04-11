use super::Error;
use super::Serializer;
use super::num::NumEncoder;
use messagepack_core::{Encode as _, encode::MapFormatEncoder, io::IoWrite};
use serde::ser;

pub(super) enum SerializeMap<'a, 'b, W, Num> {
    MapWithLen {
        ser: &'a mut Serializer<'b, W, Num>,
    },
    #[cfg(feature = "alloc")]
    MapWithOutLen {
        ser: &'a mut Serializer<'b, W, Num>,
        key_value: Option<crate::value::Value>,
        map_values: alloc::vec::Vec<(crate::value::Value, crate::value::Value)>,
    },
}

impl<'a, 'b, W, Num> SerializeMap<'a, 'b, W, Num>
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
            ser.current_length += MapFormatEncoder::new(len).encode(ser.writer)?;
            Ok(Self::MapWithLen { ser })
        } else {
            #[cfg(feature = "alloc")]
            {
                Ok(Self::MapWithOutLen {
                    ser,
                    key_value: None,
                    map_values: alloc::vec::Vec::new(),
                })
            }

            #[cfg(not(feature = "alloc"))]
            {
                Err(Error::SeqLenNone)
            }
        }
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

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        match self {
            Self::MapWithLen { ser } => key.serialize(ser.as_mut()),
            #[cfg(feature = "alloc")]
            Self::MapWithOutLen { key_value, .. } => {
                *key_value = Some(crate::value::to_value(key).map_err(convert_error)?);
                Ok(())
            }
        }
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        match self {
            Self::MapWithLen { ser } => value.serialize(ser.as_mut()),
            #[cfg(feature = "alloc")]
            Self::MapWithOutLen {
                key_value,
                map_values,
                ..
            } => {
                let key = key_value.take().ok_or_else(|| -> Self::Error {
                    serde::ser::Error::custom("`serialize_value` called before `serialize_key`")
                })?;
                let value = crate::value::to_value(value).map_err(convert_error)?;
                map_values.push((key, value));
                Ok(())
            }
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            Self::MapWithLen { .. } => Ok(()),
            #[cfg(feature = "alloc")]
            Self::MapWithOutLen {
                ser, map_values, ..
            } => {
                use serde::Serialize;
                let map = crate::value::Value::Map(map_values);
                map.serialize(ser.as_mut())?;
                Ok(())
            }
        }
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
fn convert_error<T>(err: Error<core::convert::Infallible>) -> crate::ser::Error<T> {
    match err {
        Error::Encode(e) => match e {
            messagepack_core::encode::Error::Io(_e) => {
                unreachable!("infallible error should never occur")
            }
            messagepack_core::encode::Error::InvalidFormat => {
                messagepack_core::encode::Error::InvalidFormat.into()
            }
        },
        Error::SeqLenNone => crate::ser::Error::SeqLenNone,
        #[cfg(not(feature = "alloc"))]
        Error::Custom => crate::ser::Error::Custom,
        #[cfg(feature = "alloc")]
        Error::Custom(msg) => crate::ser::Error::Custom(msg),
    }
}
