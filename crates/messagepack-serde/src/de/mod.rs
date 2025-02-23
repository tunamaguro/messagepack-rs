use messagepack_core::{
    Decode, Format,
    decode::{NbyteReader, NilDecoder},
};
use serde::{Deserialize, de, forward_to_deserialize_any};

pub mod error;
mod seq;

use error::{CoreError, Error};

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct Deserializer<'de> {
    input: &'de [u8],
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer { input }
    }

    fn decode<V: Decode<'de>>(&mut self) -> Result<V::Value, Error> {
        let (decoded, rest) = V::decode(self.input)?;
        self.input = rest;
        Ok(decoded)
    }
}

impl<'de> AsMut<Self> for Deserializer<'de> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

pub fn from_bytes<'de, T: Deserialize<'de>>(input: &'de [u8]) -> Result<T, Error> {
    let mut deserializer = Deserializer::from_bytes(input);
    T::deserialize(&mut deserializer)
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::AnyIsUnsupported)
    }

    fn deserialize_bool<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoded = self.decode::<bool>()?;
        visitor.visit_bool(decoded)
    }

    fn deserialize_i8<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoded = self.decode::<i8>()?;
        visitor.visit_i8(decoded)
    }

    fn deserialize_i16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoded = self.decode::<i16>()?;
        visitor.visit_i16(decoded)
    }

    fn deserialize_i32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoded = self.decode::<i32>()?;
        visitor.visit_i32(decoded)
    }

    fn deserialize_i64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoded = self.decode::<i64>()?;
        visitor.visit_i64(decoded)
    }

    fn deserialize_u8<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoded = self.decode::<u8>()?;
        visitor.visit_u8(decoded)
    }

    fn deserialize_u16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoded = self.decode::<u16>()?;
        visitor.visit_u16(decoded)
    }

    fn deserialize_u32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoded = self.decode::<u32>()?;
        visitor.visit_u32(decoded)
    }

    fn deserialize_u64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoded = self.decode::<u64>()?;
        visitor.visit_u64(decoded)
    }

    fn deserialize_f32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoded = self.decode::<f32>()?;
        visitor.visit_f32(decoded)
    }

    fn deserialize_f64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoded = self.decode::<f64>()?;
        visitor.visit_f64(decoded)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoded = self.decode::<&str>()?;
        visitor.visit_str(decoded)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoded = self.decode::<&[u8]>()?;
        visitor.visit_bytes(decoded)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let is_null = NilDecoder::decode(self.input).is_ok();
        if is_null {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.decode::<()>()?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let format = self.decode::<Format>()?;
        let n = match format {
            Format::FixArray(n) => n.into(),
            Format::Array16 => {
                let (n, buf) = NbyteReader::<2>::read(self.input)?;
                self.input = buf;
                n
            }
            Format::Array32 => {
                let (n, buf) = NbyteReader::<4>::read(self.input)?;
                self.input = buf;
                n
            }
            _ => return Err(CoreError::UnexpectedFormat.into()),
        };
        visitor.visit_seq(seq::ArrayDeserializer::new(self, n))
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    forward_to_deserialize_any! {
        map struct enum identifier ignored_any
    }
}

#[cfg(test)]
mod tests {
    use crate::de::error::CoreError;

    use super::*;

    #[test]
    fn decode_bool() {
        let buf = [0xc3];
        let decoded = from_bytes::<bool>(&buf).unwrap();
        assert!(decoded);

        let buf = [0xc2];
        let decoded = from_bytes::<bool>(&buf).unwrap();
        assert!(!decoded);
    }

    #[test]
    fn decode_uint8() {
        let buf = [0x05];
        let decoded = from_bytes::<u8>(&buf).unwrap();
        assert_eq!(decoded, 5);

        let buf = [0xcc, 0x80];
        let decoded = from_bytes::<u8>(&buf).unwrap();
        assert_eq!(decoded, 128);

        let buf = [0xcc, 0x80];
        let err = from_bytes::<u16>(&buf).unwrap_err();
        // not convert type
        assert_eq!(err, CoreError::UnexpectedFormat.into());
    }

    #[test]
    fn decode_float_vec() {
        // [1.1,1.2,1.3,1.4,1.5]
        let buf = [
            0x95, 0xcb, 0x3f, 0xf1, 0x99, 0x99, 0x99, 0x99, 0x99, 0x9a, 0xcb, 0x3f, 0xf3, 0x33,
            0x33, 0x33, 0x33, 0x33, 0x33, 0xcb, 0x3f, 0xf4, 0xcc, 0xcc, 0xcc, 0xcc, 0xcc, 0xcd,
            0xcb, 0x3f, 0xf6, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0xcb, 0x3f, 0xf8, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let decoded = from_bytes::<Vec<f64>>(&buf).unwrap();
        let expected = [1.1, 1.2, 1.3, 1.4, 1.5];

        assert_eq!(decoded, expected)
    }
}
