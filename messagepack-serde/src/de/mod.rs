//! Deserialize support for messagepack

mod enum_;
mod error;
mod seq;
use error::CoreError;
pub use error::Error;

use messagepack_core::{
    Decode, Format,
    decode::NbyteReader,
    io::{IoRead, RError},
};
use serde::{
    Deserialize,
    de::{self, IntoDeserializer},
    forward_to_deserialize_any,
};

/// Deserialize from [messagepack_core::io::IoRead]
pub fn from_core_reader<'de, R, T>(reader: R) -> Result<T, Error<R::Error>>
where
    R: IoRead<'de>,
    T: Deserialize<'de>,
{
    let mut deserializer = Deserializer::new(reader);
    T::deserialize(&mut deserializer)
}

/// Deserialize from slice
#[inline]
pub fn from_slice<'de, T: Deserialize<'de>>(input: &'de [u8]) -> Result<T, Error<RError>> {
    use messagepack_core::io::SliceReader;
    let reader = SliceReader::new(input);
    from_core_reader(reader)
}

#[cfg(feature = "std")]
/// Deserialize from [std::io::Read]
#[inline]
pub fn from_reader<R, T>(reader: R) -> std::io::Result<T>
where
    R: std::io::Read,
    T: for<'a> Deserialize<'a>,
{
    use messagepack_core::io::StdReader;
    use std::io;
    let reader = StdReader::new(reader);
    let result = from_core_reader::<'_, StdReader<R>, T>(reader);
    match result {
        Ok(v) => Ok(v),
        Err(err) => match err {
            Error::Decode(err) => match err {
                messagepack_core::decode::Error::InvalidData
                | messagepack_core::decode::Error::UnexpectedFormat => {
                    Err(io::Error::new(io::ErrorKind::InvalidData, err))
                }
                messagepack_core::decode::Error::UnexpectedEof => {
                    Err(io::Error::new(io::ErrorKind::UnexpectedEof, err))
                }
                messagepack_core::decode::Error::Io(e) => Err(e),
            },
            _ => Err(io::Error::other(err)),
        },
    }
}

const MAX_RECURSION_DEPTH: usize = 256;

struct Deserializer<R> {
    reader: R,
    depth: usize,
    format: Option<Format>,
}

impl<'de, R> Deserializer<R>
where
    R: IoRead<'de>,
{
    fn new(reader: R) -> Self {
        Deserializer {
            reader,
            depth: 0,
            format: None,
        }
    }

    fn recurse<F, V>(&mut self, f: F) -> Result<V, Error<R::Error>>
    where
        F: FnOnce(&mut Self) -> V,
    {
        if self.depth == MAX_RECURSION_DEPTH {
            return Err(Error::RecursionLimitExceeded);
        }
        self.depth += 1;
        let result = f(self);
        self.depth -= 1;
        Ok(result)
    }

    fn decode_format(&mut self) -> Result<Format, Error<R::Error>> {
        match self.format.take() {
            Some(v) => Ok(v),
            None => {
                let v = Format::decode(&mut self.reader)?;
                Ok(v)
            }
        }
    }

    fn decode_seq_with_format<V>(
        &mut self,
        format: Format,
        visitor: V,
    ) -> Result<V::Value, Error<R::Error>>
    where
        V: de::Visitor<'de>,
    {
        let n = match format {
            Format::FixArray(n) => n.into(),
            Format::Array16 => NbyteReader::<2>::read(&mut self.reader)?,
            Format::Array32 => NbyteReader::<4>::read(&mut self.reader)?,
            _ => return Err(CoreError::UnexpectedFormat.into()),
        };
        self.recurse(move |des| visitor.visit_seq(seq::FixLenAccess::new(des, n)))?
    }

    fn decode_map_with_format<V>(
        &mut self,
        format: Format,
        visitor: V,
    ) -> Result<V::Value, Error<R::Error>>
    where
        V: de::Visitor<'de>,
    {
        let n = match format {
            Format::FixMap(n) => n.into(),
            Format::Map16 => NbyteReader::<2>::read(&mut self.reader)?,
            Format::Map32 => NbyteReader::<4>::read(&mut self.reader)?,
            _ => return Err(CoreError::UnexpectedFormat.into()),
        };
        self.recurse(move |des| visitor.visit_map(seq::FixLenAccess::new(des, n)))?
    }
}

impl<R> AsMut<Self> for Deserializer<R> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<'de, R> de::Deserializer<'de> for &mut Deserializer<R>
where
    R: IoRead<'de>,
{
    type Error = Error<R::Error>;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let format = self.decode_format()?;
        match format {
            Format::Nil => visitor.visit_unit(),
            Format::False => visitor.visit_bool(false),
            Format::True => visitor.visit_bool(true),
            Format::PositiveFixInt(v) => visitor.visit_u8(v),
            Format::Uint8 => {
                let v = u8::decode_with_format(format, &mut self.reader)?;
                visitor.visit_u8(v)
            }
            Format::Uint16 => {
                let v = u16::decode_with_format(format, &mut self.reader)?;
                visitor.visit_u16(v)
            }
            Format::Uint32 => {
                let v = u32::decode_with_format(format, &mut self.reader)?;
                visitor.visit_u32(v)
            }
            Format::Uint64 => {
                let v = u64::decode_with_format(format, &mut self.reader)?;
                visitor.visit_u64(v)
            }
            Format::NegativeFixInt(v) => visitor.visit_i8(v),
            Format::Int8 => {
                let v = i8::decode_with_format(format, &mut self.reader)?;
                visitor.visit_i8(v)
            }
            Format::Int16 => {
                let v = i16::decode_with_format(format, &mut self.reader)?;
                visitor.visit_i16(v)
            }
            Format::Int32 => {
                let v = i32::decode_with_format(format, &mut self.reader)?;
                visitor.visit_i32(v)
            }
            Format::Int64 => {
                let v = i64::decode_with_format(format, &mut self.reader)?;
                visitor.visit_i64(v)
            }
            Format::Float32 => {
                let v = f32::decode_with_format(format, &mut self.reader)?;
                visitor.visit_f32(v)
            }
            Format::Float64 => {
                let v = f64::decode_with_format(format, &mut self.reader)?;
                visitor.visit_f64(v)
            }
            Format::FixStr(_) | Format::Str8 | Format::Str16 | Format::Str32 => {
                use messagepack_core::decode::ReferenceStrDecoder;
                let data = ReferenceStrDecoder::decode_with_format(format, &mut self.reader)?;
                match data {
                    messagepack_core::decode::ReferenceStr::Borrowed(s) => {
                        visitor.visit_borrowed_str(s)
                    }
                    messagepack_core::decode::ReferenceStr::Copied(s) => visitor.visit_str(s),
                }
            }
            Format::FixArray(_) | Format::Array16 | Format::Array32 => {
                self.decode_seq_with_format(format, visitor)
            }
            Format::Bin8 | Format::Bin16 | Format::Bin32 => {
                use messagepack_core::decode::ReferenceDecoder;
                let data = ReferenceDecoder::decode_with_format(format, &mut self.reader)?;
                match data {
                    messagepack_core::io::Reference::Borrowed(items) => {
                        visitor.visit_borrowed_bytes(items)
                    }
                    messagepack_core::io::Reference::Copied(items) => visitor.visit_bytes(items),
                }
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
                let mut de_ext =
                    crate::extension::de::DeserializeExt::new(format, &mut self.reader)?;
                let val = de::Deserializer::deserialize_newtype_struct(
                    &mut de_ext,
                    crate::extension::EXTENSION_STRUCT_NAME,
                    visitor,
                )?;

                Ok(val)
            }
            Format::NeverUsed => Err(CoreError::UnexpectedFormat.into()),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let format = self.decode_format()?;
        match format {
            Format::Nil => visitor.visit_none(),
            _ => {
                self.format = Some(format);
                visitor.visit_some(self.as_mut())
            }
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
        let format = self.decode_format()?;
        match format {
            Format::FixStr(_) | Format::Str8 | Format::Str16 | Format::Str32 => {
                let s = <&str>::decode_with_format(format, &mut self.reader)?;
                visitor.visit_enum(s.into_deserializer())
            }
            Format::FixMap(_)
            | Format::Map16
            | Format::Map32
            | Format::FixArray(_)
            | Format::Array16
            | Format::Array32 => {
                let enum_access = enum_::Enum::new(self);
                visitor.visit_enum(enum_access)
            }
            _ => Err(CoreError::UnexpectedFormat.into()),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
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
    use serde::de::IgnoredAny;

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
    fn decode_struct_from_array() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct S {
            compact: bool,
            schema: u8,
        }

        // [true, 0] where fields are in declaration order
        let buf: &[u8] = &[0x92, 0xc3, 0x00];

        let decoded = from_slice::<S>(buf).unwrap();
        assert_eq!(
            decoded,
            S {
                compact: true,
                schema: 0
            }
        );
    }

    #[test]
    fn option_consumes_nil_in_sequence() {
        // [None, 5] as an array of two elements
        let buf: &[u8] = &[0x92, 0xc0, 0x05];

        let decoded = from_slice::<(Option<u8>, u8)>(buf).unwrap();
        assert_eq!(decoded, (None, 5));
    }

    #[test]
    fn option_some_simple() {
        let buf: &[u8] = &[0x05];
        let decoded = from_slice::<Option<u8>>(buf).unwrap();
        assert_eq!(decoded, Some(5));
    }

    #[test]
    fn unit_from_nil() {
        let buf: &[u8] = &[0xc0];
        from_slice::<()>(buf).unwrap();
    }

    #[test]
    fn unit_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct U;

        let buf: &[u8] = &[0xc0];
        let decoded = from_slice::<U>(buf).unwrap();
        assert_eq!(decoded, U);
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

    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(untagged)]
    enum Untagged {
        Bool(bool),
        U8(u8),
        Pair(u8, bool),
        Struct { a: bool },
        Nested(E),
    }

    #[rstest]
    #[case([0xc3],Untagged::Bool(true))]
    #[case([0x05],Untagged::U8(5))]
    #[case([0x92, 0x02, 0xc3],Untagged::Pair(2,true))]
    #[case([0x81, 0xa1, 0x61, 0xc2],Untagged::Struct { a: false })]
    #[case([0xa4,0x55,0x6e,0x69,0x74],Untagged::Nested(E::Unit))] // "Unit"
    fn decode_untagged_enum<Buf: AsRef<[u8]>>(#[case] buf: Buf, #[case] expected: Untagged) {
        let decoded = from_slice::<Untagged>(buf.as_ref()).unwrap();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn recursion_limit_ok_at_256() {
        // [[[[...]]]] 256 nested array
        let mut buf = vec![0x91u8; 256];
        buf.push(0xc0);

        let _ = from_slice::<IgnoredAny>(&buf).unwrap();
    }

    #[test]
    fn recursion_limit_err_over_256() {
        // [[[[...]]]] 257 nested array
        let mut buf = vec![0x91u8; 257];
        buf.push(0xc0);

        let err = from_slice::<IgnoredAny>(&buf).unwrap_err();
        assert!(matches!(err, Error::RecursionLimitExceeded));
    }

    #[cfg(feature = "std")]
    #[rstest]
    // nil -> unit
    #[case([0xc0],())]
    // bool
    #[case([0xc3],true)]
    #[case([0xc2],false)]
    // positive integers (fixint/uint*)
    #[case([0x2a],42u8)]
    #[case([0xcc, 0x80],128u8)]
    #[case([0xcd, 0x01, 0x00],256u16)]
    #[case([0xce, 0x00, 0x01, 0x00, 0x00],65536u32)]
    #[case([0xcf, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],4294967296u64)]
    // negative integers (fixint/int*)
    #[case([0xff],-1i8)]
    #[case([0xd0, 0x80],-128i8)]
    #[case([0xd1, 0x80, 0x00],-32768i16)]
    #[case([0xd2, 0x80, 0x00, 0x00, 0x00],-2147483648i32)]
    #[case([0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],i64::MIN)]
    // floats
    #[case([0xca, 0x41, 0x45, 0x70, 0xa4],12.34f32)]
    #[case([0xcb, 0x3f, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],1.0f64)]
    // strings (fixstr/str8)
    #[case([0xa1, 0x61],"a".to_string())]
    #[case([0xd9, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f],"hello".to_string())]
    // binary (bin8) `bin` family need like `serde_bytes`
    #[case([0xc4, 0x03, 0x01, 0x02, 0x03],serde_bytes::ByteBuf::from(vec![1u8, 2, 3]))]
    // array (fixarray)
    #[case([0x93, 0x01, 0x02, 0x03],vec![1u8, 2, 3])]
    // map (fixmap) with 2 entries: {"a":1, "b":2}
    #[case([0x82, 0xa1, 0x61, 0x01, 0xa1, 0x62, 0x02],{
        let mut m = std::collections::BTreeMap::<String, u8>::new();
        m.insert("a".to_string(), 1u8);
        m.insert("b".to_string(), 2u8);
        m
    })]
    fn decode_success_from_reader_when_owned<
        Buf: AsRef<[u8]>,
        T: serde::de::DeserializeOwned + core::fmt::Debug + PartialEq,
    >(
        #[case] buf: Buf,
        #[case] expected: T,
    ) {
        use super::from_reader;
        let mut reader = std::io::Cursor::new(buf.as_ref());
        let val = from_reader::<_, T>(&mut reader).unwrap();
        assert_eq!(val, expected)
    }
}
