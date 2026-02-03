//!  Serialize support for messagepack
//!
//! ## Limitation
//!
//! MessagePack requires the length header of arrays and maps to be written
//! before any elements are encoded. Therefore this serializer needs serde
//! to provide the exact length up front. If serde calls
//! `serialize_seq(None)` or `serialize_map(None)`, this serializer returns
//! `Error::SeqLenNone`.
//!
//! Examples with `serde(flatten)`:
//!
//! ```rust
//! use serde::Serialize;
//! use std::collections::HashMap;
//!
//! // Fails
//! #[derive(Serialize)]
//! struct Inner { b: u8, c: u8 }
//!
//! #[derive(Serialize)]
//! struct Outer {
//!     a: u8,
//!     #[serde(flatten)]
//!     extra: Inner,
//! }
//!
//! let mut buf = [0u8; 32];
//! let v = Outer { a: 1, extra: Inner { b: 2, c: 3 } };
//! let err = messagepack_serde::ser::to_slice(&v, &mut buf).unwrap_err();
//! assert_eq!(err, messagepack_serde::ser::Error::SeqLenNone);
//! ```
//!

mod error;
mod map;
mod num;
mod seq;
pub use num::{AggressiveMinimize, Exact, LosslessMinimize, NumEncoder};

use core::marker::PhantomData;

pub use error::Error;

use messagepack_core::{
    Encode,
    encode::{BinaryEncoder, MapFormatEncoder, NilEncoder, array::ArrayFormatEncoder},
    io::{IoWrite, SliceWriter, WError},
};

use serde::ser;

/// Serialize value to [messagepack_core::io::IoWrite] with config.
pub fn to_core_writer_with_config<T, W, C>(
    value: &T,
    writer: &mut W,
    config: C,
) -> Result<usize, Error<W::Error>>
where
    T: ser::Serialize + ?Sized,
    W: IoWrite,
    C: NumEncoder<W>,
{
    let mut ser = Serializer::new(writer, config);
    value.serialize(&mut ser)?;
    Ok(ser.current_length)
}

/// Serialize value to [messagepack_core::io::IoWrite].
pub fn to_core_writer<T, W>(value: &T, writer: &mut W) -> Result<usize, Error<W::Error>>
where
    T: ser::Serialize + ?Sized,
    W: IoWrite,
{
    to_core_writer_with_config(value, writer, num::LosslessMinimize)
}

/// Serialize value to slice with config.
pub fn to_slice_with_config<'a, T, C>(
    value: &T,
    buf: &'a mut [u8],
    config: C,
) -> Result<usize, Error<WError>>
where
    T: ser::Serialize + ?Sized,
    C: NumEncoder<SliceWriter<'a>>,
{
    let mut writer = SliceWriter::new(buf);
    to_core_writer_with_config(value, &mut writer, config)
}

/// Serialize value to slice
pub fn to_slice<T>(value: &T, buf: &mut [u8]) -> Result<usize, Error<WError>>
where
    T: ser::Serialize + ?Sized,
{
    to_slice_with_config(value, buf, num::LosslessMinimize)
}

/// Serialize value as messagepack byte vector with config
#[cfg(feature = "alloc")]
pub fn to_vec_with_config<T, C>(
    value: &T,
    config: C,
) -> Result<alloc::vec::Vec<u8>, Error<core::convert::Infallible>>
where
    T: ser::Serialize + ?Sized,
    C: for<'a> NumEncoder<messagepack_core::io::VecRefWriter<'a>>,
{
    let mut buf = Vec::new();
    let mut writer = messagepack_core::io::VecRefWriter::new(&mut buf);
    to_core_writer_with_config(value, &mut writer, config)?;
    Ok(buf)
}

/// Serialize value as messagepack byte vector
#[cfg(feature = "alloc")]
pub fn to_vec<T>(value: &T) -> Result<alloc::vec::Vec<u8>, Error<core::convert::Infallible>>
where
    T: ser::Serialize + ?Sized,
{
    to_vec_with_config(value, num::LosslessMinimize)
}

#[cfg(feature = "std")]
/// Serialize value to [std::io::Write] with config.
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
    to_core_writer_with_config(value, writer, config)
}

#[cfg(feature = "std")]
/// Serialize value to [std::io::Write]
pub fn to_writer<T, W>(value: &T, writer: &mut W) -> Result<usize, Error<std::io::Error>>
where
    T: ser::Serialize + ?Sized,
    W: std::io::Write,
{
    to_writer_with_config(value, writer, num::LosslessMinimize)
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
struct Serializer<'a, W, Num> {
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
            crate::extension::EXTENSION_STRUCT_NAME => {
                let mut ser = crate::extension::ser::SerializeExt::new(self.writer);
                value.serialize(&mut ser)?;
                self.current_length += ser.length();
                Ok(())
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
        self.current_length += ArrayFormatEncoder(len).encode(self.writer)?;
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
        self.current_length += ArrayFormatEncoder(len).encode(self.writer)?;
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

    #[cfg(not(any(feature = "alloc", feature = "std")))]
    fn collect_str<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + core::fmt::Display,
    {
        Err(ser::Error::custom("`collect_str` is not supported"))
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
                0x01, // positive fixint 1
                0x02, // positive fixint 2
                0x03, // positive fixint 3
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
                0x01, // positive fixint 1
                0x02, // positive fixint 2
                0x03  // positive fixint 3
            ]
        );
    }
}
