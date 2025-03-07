mod enum_;
mod error;
mod num;
mod seq;
pub use num::{AggressiveLenient, Exact, Lenient, NumDecoder};

use core::marker::PhantomData;

pub use error::Error;

use error::CoreError;
use messagepack_core::{
    Decode, Format,
    decode::{NbyteReader, NilDecoder},
};
use serde::{
    Deserialize,
    de::{self, IntoDeserializer},
};

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct Deserializer<'de, Num> {
    input: &'de [u8],
    _phantom: PhantomData<Num>,
}

impl<'de, Num: NumDecoder<'de>> Deserializer<'de, Num> {
    pub fn from_slice(input: &'de [u8], _num: Num) -> Self {
        Deserializer {
            input,
            _phantom: Default::default(),
        }
    }

    fn decode<V: Decode<'de>>(&mut self) -> Result<V::Value, Error> {
        let (decoded, rest) = V::decode(self.input)?;
        self.input = rest;
        Ok(decoded)
    }
}

impl<Num> AsMut<Self> for Deserializer<'_, Num> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

pub fn from_slice<'de, T: Deserialize<'de>>(input: &'de [u8]) -> Result<T, Error> {
    from_slice_with_config(input, num::Exact)
}

pub fn from_slice_with_config<'de, T: Deserialize<'de>, C: NumDecoder<'de>>(
    input: &'de [u8],
    config: C,
) -> Result<T, Error> {
    let mut deserializer = Deserializer::from_slice(input, config);
    T::deserialize(&mut deserializer)
}

#[cfg(feature = "std")]
pub fn from_reader<R, T>(reader: &mut R) -> std::io::Result<T>
where
    R: std::io::Read,
    T: for<'a> Deserialize<'a>,
{
    from_reader_with_config(reader, num::Exact)
}

#[cfg(feature = "std")]
pub fn from_reader_with_config<R, T, C>(reader: &mut R, config: C) -> std::io::Result<T>
where
    R: std::io::Read,
    T: for<'a> Deserialize<'a>,
    C: for<'a> NumDecoder<'a>,
{
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let mut deserializer = Deserializer::from_slice(&buf, config);
    T::deserialize(&mut deserializer).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

impl<'de, Num: NumDecoder<'de>> de::Deserializer<'de> for &mut Deserializer<'de, Num> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::AnyIsUnsupported)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoded = self.decode::<bool>()?;
        visitor.visit_bool(decoded)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (decoded, rest) = Num::decode_i8(self.input)?;
        self.input = rest;
        visitor.visit_i8(decoded)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (decoded, rest) = Num::decode_i16(self.input)?;
        self.input = rest;
        visitor.visit_i16(decoded)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (decoded, rest) = Num::decode_i32(self.input)?;
        self.input = rest;
        visitor.visit_i32(decoded)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (decoded, rest) = Num::decode_i64(self.input)?;
        self.input = rest;
        visitor.visit_i64(decoded)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (decoded, rest) = Num::decode_i128(self.input)?;
        self.input = rest;
        visitor.visit_i128(decoded)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (decoded, rest) = Num::decode_u8(self.input)?;
        self.input = rest;
        visitor.visit_u8(decoded)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (decoded, rest) = Num::decode_u16(self.input)?;
        self.input = rest;
        visitor.visit_u16(decoded)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (decoded, rest) = Num::decode_u32(self.input)?;
        self.input = rest;
        visitor.visit_u32(decoded)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (decoded, rest) = Num::decode_u64(self.input)?;
        self.input = rest;
        visitor.visit_u64(decoded)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (decoded, rest) = Num::decode_u128(self.input)?;
        self.input = rest;
        visitor.visit_u128(decoded)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (decoded, rest) = Num::decode_f32(self.input)?;
        self.input = rest;
        visitor.visit_f32(decoded)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let (decoded, rest) = Num::decode_f64(self.input)?;
        self.input = rest;
        visitor.visit_f64(decoded)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoded = self.decode::<&str>()?;
        visitor.visit_borrowed_str(decoded)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let decoded = self.decode::<&[u8]>()?;
        visitor.visit_borrowed_bytes(decoded)
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
        visitor.visit_seq(seq::FixLenAccess::new(self, n))
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

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let format = self.decode::<Format>()?;
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

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
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
                let format = self.decode::<Format>()?;
                match format {
                    Format::FixMap(_)
                    | Format::Map16
                    | Format::Map32
                    | Format::FixArray(_)
                    | Format::Array16
                    | Format::Array32 => visitor.visit_enum(enum_::Enum::new(self)),
                    _ => Err(CoreError::UnexpectedFormat.into()),
                }
            }
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
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
