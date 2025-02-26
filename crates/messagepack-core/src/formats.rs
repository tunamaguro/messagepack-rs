//! See https://github.com/msgpack/msgpack/blob/master/spec.md#formats

const POSITIVE_FIXINT: u8 = 0x00;
#[allow(dead_code)]
const NEGATIVE_FIXINT: u8 = 0xe0;
const FIX_MAP: u8 = 0x80;
const FIX_ARRAY: u8 = 0x90;
const FIX_STR: u8 = 0xa0;

const NIL: u8 = 0xc0;
const NEVER_USED: u8 = 0xc1;
const FALSE: u8 = 0xc2;
const TRUE: u8 = 0xc3;
const BIN8: u8 = 0xc4;
const BIN16: u8 = 0xc5;
const BIN32: u8 = 0xc6;
const EXT8: u8 = 0xc7;
const EXT16: u8 = 0xc8;
const EXT32: u8 = 0xc9;
const FLOAT32: u8 = 0xca;
const FLOAT64: u8 = 0xcb;
const UINT8: u8 = 0xcc;
const UINT16: u8 = 0xcd;
const UINT32: u8 = 0xce;
const UINT64: u8 = 0xcf;
const INT8: u8 = 0xd0;
const INT16: u8 = 0xd1;
const INT32: u8 = 0xd2;
const INT64: u8 = 0xd3;
const FIXEXT1: u8 = 0xd4;
const FIXEXT2: u8 = 0xd5;
const FIXEXT4: u8 = 0xd6;
const FIXEXT8: u8 = 0xd7;
const FIXEXT16: u8 = 0xd8;
const STR8: u8 = 0xd9;
const STR16: u8 = 0xda;
const STR32: u8 = 0xdb;
const ARRAY16: u8 = 0xdc;
const ARRAY32: u8 = 0xdd;
const MAP16: u8 = 0xde;
const MAP32: u8 = 0xdf;

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Format {
    PositiveFixInt(u8),
    FixMap(u8),
    FixArray(u8),
    FixStr(u8),
    Nil,
    NeverUsed,
    False,
    True,
    Bin8,
    Bin16,
    Bin32,
    Ext8,
    Ext16,
    Ext32,
    Float32,
    Float64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Int8,
    Int16,
    Int32,
    Int64,
    FixExt1,
    FixExt2,
    FixExt4,
    FixExt8,
    FixExt16,
    Str8,
    Str16,
    Str32,
    Array16,
    Array32,
    Map16,
    Map32,
    NegativeFixInt(i8),
}

impl Format {
    pub const fn as_byte(&self) -> u8 {
        match self {
            Format::PositiveFixInt(v) => POSITIVE_FIXINT | *v,
            Format::FixMap(l) => FIX_MAP | *l,
            Format::FixArray(l) => FIX_ARRAY | *l,
            Format::FixStr(l) => FIX_STR | *l,
            Format::Nil => NIL,
            Format::NeverUsed => NEVER_USED,
            Format::False => FALSE,
            Format::True => TRUE,
            Format::Bin8 => BIN8,
            Format::Bin16 => BIN16,
            Format::Bin32 => BIN32,
            Format::Ext8 => EXT8,
            Format::Ext16 => EXT16,
            Format::Ext32 => EXT32,
            Format::Float32 => FLOAT32,
            Format::Float64 => FLOAT64,
            Format::Uint8 => UINT8,
            Format::Uint16 => UINT16,
            Format::Uint32 => UINT32,
            Format::Uint64 => UINT64,
            Format::Int8 => INT8,
            Format::Int16 => INT16,
            Format::Int32 => INT32,
            Format::Int64 => INT64,
            Format::FixExt1 => FIXEXT1,
            Format::FixExt2 => FIXEXT2,
            Format::FixExt4 => FIXEXT4,
            Format::FixExt8 => FIXEXT8,
            Format::FixExt16 => FIXEXT16,
            Format::Str8 => STR8,
            Format::Str16 => STR16,
            Format::Str32 => STR32,
            Format::Array16 => ARRAY16,
            Format::Array32 => ARRAY32,
            Format::Map16 => MAP16,
            Format::Map32 => MAP32,
            Format::NegativeFixInt(v) => *v as u8,
        }
    }

    pub const fn from_byte(byte: u8) -> Self {
        match byte {
            0x00..=0x7f => Self::PositiveFixInt(byte & !POSITIVE_FIXINT),
            0x80..=0x8f => Self::FixMap(byte & !FIX_MAP),
            0x90..=0x9f => Self::FixArray(byte & !FIX_ARRAY),
            0xa0..=0xbf => Self::FixStr(byte & !FIX_STR),
            NIL => Self::Nil,
            NEVER_USED => Self::NeverUsed,
            FALSE => Self::False,
            TRUE => Self::True,
            BIN8 => Self::Bin8,
            BIN16 => Self::Bin16,
            BIN32 => Self::Bin32,
            EXT8 => Self::Ext8,
            EXT16 => Self::Ext16,
            EXT32 => Self::Ext32,
            FLOAT32 => Self::Float32,
            FLOAT64 => Self::Float64,
            UINT8 => Self::Uint8,
            UINT16 => Self::Uint16,
            UINT32 => Self::Uint32,
            UINT64 => Self::Uint64,
            INT8 => Self::Int8,
            INT16 => Self::Int16,
            INT32 => Self::Int32,
            INT64 => Self::Int64,
            FIXEXT1 => Self::FixExt1,
            FIXEXT2 => Self::FixExt2,
            FIXEXT4 => Self::FixExt4,
            FIXEXT8 => Self::FixExt8,
            FIXEXT16 => Self::FixExt16,
            STR8 => Self::Str8,
            STR16 => Self::Str16,
            STR32 => Self::Str32,
            ARRAY16 => Self::Array16,
            ARRAY32 => Self::Array32,
            MAP16 => Self::Map16,
            MAP32 => Self::Map32,
            0xe0..=0xff => Self::NegativeFixInt(byte as i8),
        }
    }

    pub const fn as_slice(&self) -> [u8; 1] {
        self.as_byte().to_be_bytes()
    }
}

impl IntoIterator for Format {
    type Item = u8;
    type IntoIter = core::array::IntoIter<u8, 1>;
    fn into_iter(self) -> Self::IntoIter {
        [self.as_byte()].into_iter()
    }
}
