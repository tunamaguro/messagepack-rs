use messagepack_core::{
    decode::{Decode, DecodeBorrowed, Error as DecodeError, NilDecoder},
    encode::{BinaryEncoder, Encode, Error as EncodeError, NilEncoder},
    extension::ExtensionOwned,
    io::{IoRead, IoWrite, SliceReader, VecRefWriter},
};
use proptest::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Integer {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
}

impl<W: IoWrite> Encode<W> for Integer {
    fn encode(&self, writer: &mut W) -> Result<usize, EncodeError<W::Error>> {
        match self {
            Integer::U8(v) => v.encode(writer),
            Integer::U16(v) => v.encode(writer),
            Integer::U32(v) => v.encode(writer),
            Integer::U64(v) => v.encode(writer),
            Integer::I8(v) => v.encode(writer),
            Integer::I16(v) => v.encode(writer),
            Integer::I32(v) => v.encode(writer),
            Integer::I64(v) => v.encode(writer),
        }
    }
}

impl<'de> DecodeBorrowed<'de> for Integer {
    type Value = Self;

    fn decode_borrowed_with_format<R>(
        format: messagepack_core::Format,
        reader: &mut R,
    ) -> Result<<Self as DecodeBorrowed<'de>>::Value, DecodeError<R::Error>>
    where
        R: IoRead<'de>,
    {
        u8::decode_with_format(format, reader)
            .map(Self::U8)
            .or_else(|_| u16::decode_with_format(format, reader).map(Self::U16))
            .or_else(|_| u32::decode_with_format(format, reader).map(Self::U32))
            .or_else(|_| u64::decode_with_format(format, reader).map(Self::U64))
            .or_else(|_| i8::decode_with_format(format, reader).map(Self::I8))
            .or_else(|_| i16::decode_with_format(format, reader).map(Self::I16))
            .or_else(|_| i32::decode_with_format(format, reader).map(Self::I32))
            .or_else(|_| i64::decode_with_format(format, reader).map(Self::I64))
    }
}

fn integer_arb() -> impl Strategy<Value = Integer> {
    prop_oneof![
        any::<u8>().prop_map(Integer::U8),
        any::<u16>().prop_map(Integer::U16),
        any::<u32>().prop_map(Integer::U32),
        any::<u64>().prop_map(Integer::U64),
        any::<i8>().prop_map(Integer::I8),
        any::<i16>().prop_map(Integer::I16),
        any::<i32>().prop_map(Integer::I32),
        any::<i64>().prop_map(Integer::I64),
    ]
}

#[derive(Debug, Clone, Copy, PartialOrd)]
enum Float {
    F32(f32),
    F64(f64),
}

impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Float::F32(a), Float::F32(b)) => (a.is_nan() && b.is_nan()) || (a == b),
            (Float::F64(a), Float::F64(b)) => (a.is_nan() && b.is_nan()) || (a == b),
            _ => false,
        }
    }
}

impl<W: IoWrite> Encode<W> for Float {
    fn encode(&self, writer: &mut W) -> Result<usize, EncodeError<W::Error>> {
        match self {
            Float::F32(v) => v.encode(writer),
            Float::F64(v) => v.encode(writer),
        }
    }
}

impl<'de> DecodeBorrowed<'de> for Float {
    type Value = Self;

    fn decode_borrowed_with_format<R>(
        format: messagepack_core::Format,
        reader: &mut R,
    ) -> Result<<Self as DecodeBorrowed<'de>>::Value, DecodeError<R::Error>>
    where
        R: IoRead<'de>,
    {
        f32::decode_with_format(format, reader)
            .map(Self::F32)
            .or_else(|_| f64::decode_with_format(format, reader).map(Self::F64))
    }
}

fn float_arb() -> impl Strategy<Value = Float> {
    prop_oneof![
        any::<f32>().prop_map(Float::F32),
        any::<f64>().prop_map(Float::F64),
    ]
}

fn extension_arb() -> impl Strategy<Value = ExtensionOwned> {
    (any::<i8>(), any::<Vec<u8>>()).prop_map(|(ext_type, data)| ExtensionOwned::new(ext_type, data))
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum MessagePackType {
    Nil,
    Bool(bool),
    Integer(Integer),
    Float(Float),
    Str(String),
    Bin(Vec<u8>),
    Array(Vec<Self>),
    Map(std::collections::BTreeMap<String, Self>),
    Ext(ExtensionOwned),
}

impl<W: IoWrite> Encode<W> for MessagePackType {
    fn encode(&self, writer: &mut W) -> Result<usize, EncodeError<W::Error>> {
        match self {
            MessagePackType::Nil => NilEncoder.encode(writer),
            MessagePackType::Bool(v) => v.encode(writer),
            MessagePackType::Integer(integer) => integer.encode(writer),
            MessagePackType::Float(float) => float.encode(writer),
            MessagePackType::Str(s) => s.encode(writer),
            MessagePackType::Bin(bin) => BinaryEncoder(bin.as_slice()).encode(writer),
            MessagePackType::Array(array) => array.encode(writer),
            MessagePackType::Map(map) => map.encode(writer),
            MessagePackType::Ext(ext) => ext.encode(writer),
        }
    }
}

impl<'de> DecodeBorrowed<'de> for MessagePackType {
    type Value = Self;

    fn decode_borrowed_with_format<R>(
        format: messagepack_core::Format,
        reader: &mut R,
    ) -> Result<<Self as DecodeBorrowed<'de>>::Value, DecodeError<R::Error>>
    where
        R: IoRead<'de>,
    {
        NilDecoder::decode_with_format(format, reader)
            .map(|_| Self::Nil)
            .or_else(|_| bool::decode_with_format(format, reader).map(Self::Bool))
            .or_else(|_| Integer::decode_with_format(format, reader).map(Self::Integer))
            .or_else(|_| Float::decode_with_format(format, reader).map(Self::Float))
            .or_else(|_| String::decode_with_format(format, reader).map(Self::Str))
            .or_else(|_| {
                <&[u8]>::decode_with_format(format, reader).map(|bin| Self::Bin(bin.into()))
            })
            .or_else(|_| Vec::<Self>::decode_with_format(format, reader).map(Self::Array))
            .or_else(|_| {
                std::collections::BTreeMap::<String, Self>::decode_with_format(format, reader)
                    .map(Self::Map)
            })
            .or_else(|_| ExtensionOwned::decode_with_format(format, reader).map(Self::Ext))
    }
}

fn messagepack_arb() -> impl Strategy<Value = MessagePackType> {
    let leaf = prop_oneof![
        Just(MessagePackType::Nil),
        any::<bool>().prop_map(MessagePackType::Bool),
        integer_arb().prop_map(MessagePackType::Integer),
        float_arb().prop_map(MessagePackType::Float),
        any::<String>().prop_map(MessagePackType::Str),
        any::<Vec<u8>>().prop_map(MessagePackType::Bin),
        extension_arb().prop_map(MessagePackType::Ext)
    ];

    leaf.prop_recursive(6, 64, 8, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..=16).prop_map(MessagePackType::Array),
            prop::collection::btree_map(any::<String>(), inner, 0..=16)
                .prop_map(MessagePackType::Map),
        ]
    })
}

proptest! {
    #[test]
    fn roundtrip_any(x in messagepack_arb()) {
        let mut buf = vec![];
        let mut writer = VecRefWriter::new(&mut buf);
        let written_length = x.encode(&mut writer).unwrap();

        let mut reader = SliceReader::new(&buf[..written_length]);
        let y = MessagePackType::decode(&mut reader).unwrap();

        assert_eq!(x,y);
    }
}
