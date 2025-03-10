pub mod error;
mod map;
mod num;
mod seq;
pub use num::{AggressiveMinimize, Exact, LosslessMinimize, NumEncoder};

use core::marker::PhantomData;

use crate::value::extension::{EXTENSION_STRUCT_NAME, SerializeExt};
pub use error::Error;
use messagepack_core::{
    Encode, SliceWriter,
    encode::{ArrayFormatEncoder, BinaryEncoder, MapFormatEncoder, NilEncoder},
    io::{IoWrite, WError},
};

use serde::ser;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct Serializer<'a, W, Num> {
    writer: &'a mut W,
    current_length: usize,
    num_encoder: PhantomData<Num>,
}

impl<'a, W, Num> Serializer<'a, W, Num>
where
    W: IoWrite,
    Num: num::NumEncoder<W>,
{
    pub fn new(writer: &'a mut W, _num_encoder: Num) -> Self {
        Self {
            writer,
            current_length: 0,
            num_encoder: PhantomData,
        }
    }
}

impl<W, Num> AsMut<Self> for Serializer<'_, W, Num> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

pub fn to_slice<T>(value: &T, buf: &mut [u8]) -> Result<usize, Error<WError>>
where
    T: ser::Serialize + ?Sized,
{
    to_slice_with_config(value, buf, num::Exact)
}

pub fn to_slice_with_config<'a, T, C>(
    value: &T,
    buf: &'a mut [u8],
    config: C,
) -> Result<usize, Error<WError>>
where
    T: ser::Serialize + ?Sized,
    C: NumEncoder<SliceWriter<'a>>,
{
    let mut writer = SliceWriter::from_slice(buf);
    let mut ser = Serializer::new(&mut writer, config);
    value.serialize(&mut ser)?;
    Ok(ser.current_length)
}

#[cfg(feature = "std")]
pub fn to_writer<T, W>(value: &T, writer: &mut W) -> Result<usize, Error<std::io::Error>>
where
    T: ser::Serialize + ?Sized,
    W: std::io::Write,
{
    to_writer_with_config(value, writer, num::Exact)
}

#[cfg(feature = "std")]
pub fn to_writer_with_config<T, W, C>(
    value: &T,
    writer: &mut W,
    config: C,
) -> Result<usize, Error<std::io::Error>>
where
    T: ser::Serialize + ?Sized,
    W: std::io::Write,
    C: NumEncoder<W>,
{
    let mut ser = Serializer::new(writer, config);
    value.serialize(&mut ser)?;
    Ok(ser.current_length)
}

#[cfg(feature = "std")]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>, Error<std::io::Error>>
where
    T: ser::Serialize + ?Sized,
{
    let mut buf = Vec::new();

    to_writer(value, &mut buf)?;
    Ok(buf)
}

#[cfg(feature = "std")]
pub fn to_vec_with_config<T, C>(value: &T, config: C) -> Result<Vec<u8>, Error<std::io::Error>>
where
    T: ser::Serialize + ?Sized,
    C: NumEncoder<Vec<u8>>,
{
    let mut buf = Vec::new();

    to_writer_with_config(value, &mut buf, config)?;
    Ok(buf)
}

impl<'a, 'b: 'a, W, Num> ser::Serializer for &'a mut Serializer<'b, W, Num>
where
    W: IoWrite,
    Num: NumEncoder<W>,
{
    type Ok = ();
    type Error = Error<W::Error>;

    type SerializeSeq = seq::SerializeSeq<'a, 'b, W, Num>;
    type SerializeTuple = seq::SerializeSeq<'a, 'b, W, Num>;
    type SerializeTupleStruct = seq::SerializeSeq<'a, 'b, W, Num>;
    type SerializeTupleVariant = seq::SerializeSeq<'a, 'b, W, Num>;
    type SerializeMap = map::SerializeMap<'a, 'b, W, Num>;
    type SerializeStruct = map::SerializeMap<'a, 'b, W, Num>;
    type SerializeStructVariant = map::SerializeMap<'a, 'b, W, Num>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.current_length += v.encode(self.writer)?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.current_length += Num::encode_i8(v, self.writer)?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.current_length += Num::encode_i16(v, self.writer)?;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.current_length += Num::encode_i32(v, self.writer)?;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.current_length += Num::encode_i64(v, self.writer)?;
        Ok(())
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        self.current_length += Num::encode_i128(v, self.writer)?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.current_length += Num::encode_u8(v, self.writer)?;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.current_length += Num::encode_u16(v, self.writer)?;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.current_length += Num::encode_u32(v, self.writer)?;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.current_length += Num::encode_u64(v, self.writer)?;
        Ok(())
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        self.current_length += Num::encode_u128(v, self.writer)?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.current_length += Num::encode_f32(v, self.writer)?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.current_length += Num::encode_f64(v, self.writer)?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        // char takes max 4 bytes
        let mut buf = [0u8; 4];
        let s = v.encode_utf8(&mut buf);
        self.serialize_str(s)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.current_length += v.encode(self.writer)?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.current_length += BinaryEncoder(v).encode(self.writer)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.current_length += NilEncoder.encode(self.writer)?;
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        match name {
            EXTENSION_STRUCT_NAME => {
                let mut ser = SerializeExt::new(self.writer, &mut self.current_length);
                value.serialize(&mut ser)
            }
            _ => value.serialize(self.as_mut()),
        }
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.current_length += MapFormatEncoder::new(1).encode(self.writer)?;
        self.serialize_str(variant)?;
        value.serialize(self.as_mut())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let len = len.ok_or(Error::SeqLenNone)?;
        self.current_length += ArrayFormatEncoder::new(len).encode(self.writer)?;
        Ok(seq::SerializeSeq::new(self))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.current_length += MapFormatEncoder::new(1).encode(self.writer)?;
        self.serialize_str(variant)?;
        self.current_length += ArrayFormatEncoder::new(len).encode(self.writer)?;
        Ok(seq::SerializeSeq::new(self))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let len = len.ok_or(Error::SeqLenNone)?;
        self.current_length += MapFormatEncoder::new(len).encode(self.writer)?;
        Ok(map::SerializeMap::new(self))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.current_length += MapFormatEncoder::new(len).encode(self.writer)?;
        Ok(map::SerializeMap::new(self))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.current_length += MapFormatEncoder::new(1).encode(self.writer)?;
        self.serialize_str(variant)?;
        self.serialize_struct(name, len)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::*;

    #[test]
    fn encode_nil() {
        let v: Option<()> = None;
        let buf = &mut [0u8; 128];
        let len = to_slice(&v, buf).unwrap();
        assert_eq!(buf[..len], [0xc0]);
    }

    #[test]
    fn encode_unit() {
        let buf = &mut [0u8; 128];
        let len = to_slice(&(), buf).unwrap();
        assert_eq!(buf[..len], [0xc0]);
    }

    #[test]
    fn encode_unit_struct() {
        #[derive(Serialize)]
        struct Unit;
        let buf = &mut [0u8; 128];
        let len = to_slice(&Unit, buf).unwrap();
        assert_eq!(buf[..len], [0xc0]);
    }

    #[test]
    fn encode_false() {
        let v = false;
        let buf = &mut [0u8; 128];
        let len = to_slice(&v, buf).unwrap();
        assert_eq!(buf[..len], [0xc2]);
    }

    #[test]
    fn encode_true() {
        let v = true;
        let buf = &mut [0u8; 128];
        let len = to_slice(&v, buf).unwrap();
        assert_eq!(buf[..len], [0xc3]);
    }

    #[test]
    fn encode_bytes() {
        // default &[u8] not call serialize_bytes
        let v = serde_bytes::Bytes::new(&[5, 4, 3, 2, 1, 0]);

        let buf = &mut [0u8; 128];
        let len = to_slice(&v, buf).unwrap();
        assert_eq!(buf[..len], [0xc4, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00]);
    }

    #[test]
    fn encode_enum() {
        #[derive(Serialize)]
        enum Type {
            Bool,
            Int,
            Float,
        }
        let buf = &mut [0u8; 128];
        {
            let len = to_slice(&Type::Bool, buf).unwrap();
            assert_eq!(buf[..len], [0xa4, b'B', b'o', b'o', b'l'])
        }
        {
            let len = to_slice(&Type::Int, buf).unwrap();
            assert_eq!(buf[..len], [0xa3, b'I', b'n', b't'])
        }
        {
            let len = to_slice(&Type::Float, buf).unwrap();
            assert_eq!(buf[..len], [0xa5, b'F', b'l', b'o', b'a', b't'])
        }
    }

    #[test]
    fn encode_newtype_struct() {
        #[derive(Serialize)]
        struct B(#[serde(with = "serde_bytes")] [u8; 5]);

        let buf = &mut [0u8; 128];

        let len = to_slice(&B([5, 4, 3, 2, 1]), buf).unwrap();
        assert_eq!(buf[..len], [0xc4, 0x05, 0x05, 0x04, 0x03, 0x02, 0x01])
    }

    #[test]
    fn encode_newtype_variant() {
        #[derive(Serialize)]
        enum Type {
            Bool(bool),
            Int(u8),
            Float(f32),
        }

        let buf = &mut [0u8; 128];
        {
            let len = to_slice(&Type::Bool(true), buf).unwrap();
            assert_eq!(
                buf[..len],
                [
                    0x81, // fixmap len = 1
                    0xa4, // fixstr len = 4
                    b'B', b'o', b'o', b'l', 0xc3 // true
                ]
            )
        }
        {
            let len = to_slice(&Type::Int(128), buf).unwrap();
            assert_eq!(
                buf[..len],
                [
                    0x81, // fixmap len = 1
                    0xa3, b'I', b'n', b't', // fixstr "Int"
                    0xcc, 0x80 // uint8 128
                ]
            )
        }

        {
            let len = to_slice(&Type::Float(12.34), buf).unwrap();
            assert_eq!(
                buf[..len],
                [
                    0x81, // fixmap len = 1
                    0xa5, b'F', b'l', b'o', b'a', b't', // fixstr "Float"
                    0xca, 0x41, 0x45, 0x70, 0xa4 // float32 12.34
                ]
            )
        }
    }

    #[test]
    fn encode_struct_variant() {
        #[derive(Serialize)]
        enum Type {
            Bool { flag: bool, msg: &'static str },
        }

        // expect
        // {
        //   "Bool":{
        //       "flag": bool,
        //       "msg": "Some message"
        //    }
        // }

        let buf = &mut [0u8; 128];
        {
            let len = to_slice(
                &Type::Bool {
                    flag: false,
                    msg: "hi",
                },
                buf,
            )
            .unwrap();
            assert_eq!(
                buf[..len],
                [
                    0x81, // fixmap len = 1
                    0xa4, // fixstr len = 4
                    b'B', b'o', b'o', b'l', // top
                    0x82, // fixmap len = 2
                    0xa4, // fixstr len = 4
                    b'f', b'l', b'a', b'g', // key
                    0xc2, // false
                    0xa3, // fixstr len = 3
                    b'm', b's', b'g', //key
                    0xa2, // fixstr len = 2
                    b'h', b'i',
                ]
            )
        }
    }

    #[test]
    fn encode_tuple_struct() {
        #[derive(Serialize)]
        struct V(i16, u32, i32);

        let buf = &mut [0u8; 128];
        let len = to_slice(&V(1, 2, 3), buf).unwrap();
        assert_eq!(
            buf[..len],
            [
                0x93, // fixarr len = 3
                0xd1, 0x00, 0x01, // int16
                0xce, 0x00, 0x00, 0x00, 0x02, // uint32
                0xd2, 0x00, 0x00, 0x00, 0x03, // uint32
            ]
        );
    }

    #[test]
    fn encode_pos_fix_int() {
        let v = 127_u8;
        let buf = &mut [0u8; 128];
        let len = to_slice(&v, buf).unwrap();
        assert_eq!(buf[..len], [0x7f]);
    }

    #[test]
    fn encode_neg_fix_int() {
        let v = -32_i8;
        let buf = &mut [0u8; 128];
        let len = to_slice(&v, buf).unwrap();
        assert_eq!(buf[..len], [0xe0]);
    }

    #[test]
    fn encode_extension() {
        use crate::value::extension::ExtensionRef;
        let kind: i8 = 123;
        let ext = ExtensionRef::new(kind, &[0x12]);
        let buf = &mut [0_u8; 3];

        let len = to_slice(&ext, buf).unwrap();
        assert_eq!(buf[..len], [0xd4, kind.to_be_bytes()[0], 0x12]);
    }

    #[cfg(feature = "std")]
    #[test]
    fn encode_with_writer() {
        #[derive(Serialize)]
        struct V(i16, u32, i32);

        let mut buf = vec![];
        let len = to_writer(&V(1, 2, 3), &mut buf).unwrap();
        assert_eq!(
            buf[..len],
            [
                0x93, // fixarr len = 3
                0xd1, 0x00, 0x01, // int16
                0xce, 0x00, 0x00, 0x00, 0x02, // uint32
                0xd2, 0x00, 0x00, 0x00, 0x03, // uint32
            ]
        );
    }
}
