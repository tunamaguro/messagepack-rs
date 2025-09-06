mod enum_;
mod error;
mod seq;
use error::CoreError;
pub use error::Error;

use crate::value::extension::DeserializeExt;
use messagepack_core::{Decode, Format, decode::NbyteReader};
use serde::{
    Deserialize,
    de::{self, IntoDeserializer},
    forward_to_deserialize_any,
};

/// Deserialize from slice
pub fn from_slice<'de, T: Deserialize<'de>>(input: &'de [u8]) -> Result<T, Error> {
    let mut deserializer = Deserializer::from_slice(input);
    T::deserialize(&mut deserializer)
}

#[cfg(feature = "std")]
/// Deserialize from [std::io::Read]
pub fn from_reader<R, T>(reader: &mut R) -> std::io::Result<T>
where
    R: std::io::Read,
    T: for<'a> Deserialize<'a>,
{
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let mut deserializer = Deserializer::from_slice(&buf);
    T::deserialize(&mut deserializer).map_err(std::io::Error::other)
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
struct Deserializer<'de> {
    input: &'de [u8],
}

impl<'de> Deserializer<'de> {
    pub fn from_slice(input: &'de [u8]) -> Self {
        Deserializer { input }
    }

    fn decode<V: Decode<'de>>(&mut self) -> Result<V::Value, Error> {
        let (decoded, rest) = V::decode(self.input)?;
        self.input = rest;
        Ok(decoded)
    }

    fn decode_with_format<V: Decode<'de>>(&mut self, format: Format) -> Result<V::Value, Error> {
        let (decoded, rest) = V::decode_with_format(format, self.input)?;
        self.input = rest;
        Ok(decoded)
    }

    fn decode_seq_with_format<V>(&mut self, format: Format, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
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
        visitor.visit_seq(seq::FixLenAccess::new(self, n))
    }

    fn decode_map_with_format<V>(&mut self, format: Format, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        let n = match format {
            Format::FixMap(n) => n.into(),
            Format::Map16 => {
                let (n, buf) = NbyteReader::<2>::read(self.input)?;
                self.input = buf;
                n
            }
            Format::Map32 => {
                let (n, buf) = NbyteReader::<4>::read(self.input)?;
                self.input = buf;
                n
            }
            _ => return Err(CoreError::UnexpectedFormat.into()),
        };
        visitor.visit_map(seq::FixLenAccess::new(self, n))
    }
}

impl AsMut<Self> for Deserializer<'_> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<'de> de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let format = self.decode::<Format>()?;
        match format {
            Format::Nil => visitor.visit_none(),
            Format::False => visitor.visit_bool(false),
            Format::True => visitor.visit_bool(true),
            Format::PositiveFixInt(v) => visitor.visit_u8(v),
            Format::Uint8 => {
                let v = self.decode_with_format::<u8>(format)?;
                visitor.visit_u8(v)
            }
            Format::Uint16 => {
                let v = self.decode_with_format::<u16>(format)?;
                visitor.visit_u16(v)
            }
            Format::Uint32 => {
                let v = self.decode_with_format::<u32>(format)?;
                visitor.visit_u32(v)
            }
            Format::Uint64 => {
                let v = self.decode_with_format::<u64>(format)?;
                visitor.visit_u64(v)
            }
            Format::NegativeFixInt(v) => visitor.visit_i8(v),
            Format::Int8 => {
                let v = self.decode_with_format::<i8>(format)?;
                visitor.visit_i8(v)
            }
            Format::Int16 => {
                let v = self.decode_with_format::<i16>(format)?;
                visitor.visit_i16(v)
            }
            Format::Int32 => {
                let v = self.decode_with_format::<i32>(format)?;
                visitor.visit_i32(v)
            }
            Format::Int64 => {
                let v = self.decode_with_format::<i64>(format)?;
                visitor.visit_i64(v)
            }
            Format::Float32 => {
                let v = self.decode_with_format::<f32>(format)?;
                visitor.visit_f32(v)
            }
            Format::Float64 => {
                let v = self.decode_with_format::<f64>(format)?;
                visitor.visit_f64(v)
            }
            Format::FixStr(_) | Format::Str8 | Format::Str16 | Format::Str32 => {
                let v = self.decode_with_format::<&str>(format)?;
                visitor.visit_borrowed_str(v)
            }
            Format::FixArray(_) | Format::Array16 | Format::Array32 => {
                self.decode_seq_with_format(format, visitor)
            }
            Format::Bin8 | Format::Bin16 | Format::Bin32 => {
                let v = self.decode_with_format::<&[u8]>(format)?;
                visitor.visit_borrowed_bytes(v)
            }
            Format::FixMap(_) | Format::Map16 | Format::Map32 => {
                self.decode_map_with_format(format, visitor)
            }
            Format::Ext8
            | Format::Ext16
            | Format::Ext32
            | Format::FixExt1
            | Format::FixExt2
            | Format::FixExt4
            | Format::FixExt8
            | Format::FixExt16 => {
                let mut de_ext = DeserializeExt::new(format, self.input)?;
                let val = (&mut de_ext).deserialize_newtype_struct(
                    crate::value::extension::EXTENSION_STRUCT_NAME,
                    visitor,
                )?;
                self.input = de_ext.input;

                Ok(val)
            }
            Format::NeverUsed => Err(CoreError::UnexpectedFormat.into()),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let ident = self.decode::<&str>();
        match ident {
            Ok(ident) => visitor.visit_enum(ident.into_deserializer()),
            _ => {
                let (format, rest) = Format::decode(self.input)?;

                let mut des = Deserializer::from_slice(rest);
                let val = match format {
                    Format::FixMap(_)
                    | Format::Map16
                    | Format::Map32
                    | Format::FixArray(_)
                    | Format::Array16
                    | Format::Array32 => visitor.visit_enum(enum_::Enum::new(&mut des)),
                    _ => Err(CoreError::UnexpectedFormat.into()),
                }?;

                self.input = des.input;

                Ok(val)
            }
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case([0xc3],true)]
    #[case([0xc2],false)]
    fn decode_bool<Buf: AsRef<[u8]>>(#[case] buf: Buf, #[case] expected: bool) {
        let decoded = from_slice::<bool>(buf.as_ref()).unwrap();
        assert_eq!(decoded, expected);
    }

    #[rstest]
    #[case([0x05],5)]
    #[case([0xcc, 0x80],128)]
    fn decode_uint8<Buf: AsRef<[u8]>>(#[case] buf: Buf, #[case] expected: u8) {
        let decoded = from_slice::<u8>(buf.as_ref()).unwrap();
        assert_eq!(decoded, expected);
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

        let decoded = from_slice::<Vec<f64>>(&buf).unwrap();
        let expected = [1.1, 1.2, 1.3, 1.4, 1.5];

        assert_eq!(decoded, expected)
    }

    #[test]
    fn decode_struct() {
        #[derive(Deserialize)]
        struct S {
            compact: bool,
            schema: u8,
        }

        // {"super":1,"schema":0}
        let buf: &[u8] = &[
            0x82, 0xa7, 0x63, 0x6f, 0x6d, 0x70, 0x61, 0x63, 0x74, 0xc3, 0xa6, 0x73, 0x63, 0x68,
            0x65, 0x6d, 0x61, 0x00,
        ];

        let decoded = from_slice::<S>(buf).unwrap();
        assert!(decoded.compact);
        assert_eq!(decoded.schema, 0);
    }

    #[test]
    fn option_consumes_nil_in_sequence() {
        // [None, 5] as an array of two elements
        let buf: &[u8] = &[0x92, 0xc0, 0x05];

        let decoded = from_slice::<(Option<u8>, u8)>(buf).unwrap();
        assert_eq!(decoded, (None, 5));
    }

    #[derive(Deserialize, PartialEq, Debug)]
    enum E {
        Unit,
        Newtype(u8),
        Tuple(u8, bool),
        Struct { a: bool },
    }
    #[rstest]
    #[case([0xa4, 0x55, 0x6e, 0x69, 0x74],E::Unit)] // "Unit"
    #[case([0x81, 0xa7, 0x4e, 0x65, 0x77, 0x74, 0x79, 0x70, 0x65, 0x1b], E::Newtype(27))] // {"Newtype":27}
    #[case([0x81, 0xa5, 0x54, 0x75, 0x70, 0x6c, 0x65, 0x92, 0x03, 0xc3], E::Tuple(3, true))] // {"Tuple":[3,true]}
    #[case([0x81, 0xa6, 0x53, 0x74, 0x72, 0x75, 0x63, 0x74, 0x81, 0xa1, 0x61, 0xc2],E::Struct { a: false })] // {"Struct":{"a":false}}
    fn decode_enum<Buf: AsRef<[u8]>>(#[case] buf: Buf, #[case] expected: E) {
        let decoded = from_slice::<E>(buf.as_ref()).unwrap();
        assert_eq!(decoded, expected);
    }
}
