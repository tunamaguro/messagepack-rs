//! MessagePack format markers.
//!
//! See <https://github.com/msgpack/msgpack/blob/master/spec.md#formats>

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

/// MessagePack format marker.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum Format {
    /// Positive fixint (0xxxxxxx): stores a positive 7‑bit integer in the marker.
    PositiveFixInt(u8),
    /// Fixmap (1000xxxx): small map with length encoded in the marker.
    FixMap(u8),
    /// Fixarray (1001xxxx): small array with length encoded in the marker.
    FixArray(u8),
    /// Fixstr (101xxxxx): small string with byte length in the marker.
    FixStr(u8),
    /// Nil (0xc0).
    Nil,
    /// Reserved (0xc1): never used.
    NeverUsed,
    /// False (0xc2).
    False,
    /// True (0xc3).
    True,
    /// Binary with 8‑bit length (0xc4).
    Bin8,
    /// Binary with 16‑bit length (0xc5).
    Bin16,
    /// Binary with 32‑bit length (0xc6).
    Bin32,
    /// Extension with 8‑bit length (0xc7).
    Ext8,
    /// Extension with 16‑bit length (0xc8).
    Ext16,
    /// Extension with 32‑bit length (0xc9).
    Ext32,
    /// Float32 (0xca).
    Float32,
    /// Float64 (0xcb).
    Float64,
    /// Unsigned 8‑bit integer (0xcc).
    Uint8,
    /// Unsigned 16‑bit integer (0xcd).
    Uint16,
    /// Unsigned 32‑bit integer (0xce).
    Uint32,
    /// Unsigned 64‑bit integer (0xcf).
    Uint64,
    /// Signed 8‑bit integer (0xd0).
    Int8,
    /// Signed 16‑bit integer (0xd1).
    Int16,
    /// Signed 32‑bit integer (0xd2).
    Int32,
    /// Signed 64‑bit integer (0xd3).
    Int64,
    /// Fixext 1 (0xd4).
    FixExt1,
    /// Fixext 2 (0xd5).
    FixExt2,
    /// Fixext 4 (0xd6).
    FixExt4,
    /// Fixext 8 (0xd7).
    FixExt8,
    /// Fixext 16 (0xd8).
    FixExt16,
    /// Str8: UTF‑8 string with 8‑bit length (0xd9).
    Str8,
    /// Str16: UTF‑8 string with 16‑bit length (0xda).
    Str16,
    /// Str32: UTF‑8 string with 32‑bit length (0xdb).
    Str32,
    /// Array16: array with 16‑bit length (0xdc).
    Array16,
    /// Array32: array with 32‑bit length (0xdd).
    Array32,
    /// Map16: map with 16‑bit length (0xde).
    Map16,
    /// Map32: map with 32‑bit length (0xdf).
    Map32,
    /// Negative fixint (111xxxxx): stores a negative 5‑bit integer in the marker.
    NegativeFixInt(i8),
}

impl Format {
    /// Return the marker byte for this format.
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

    /// Parse a marker byte into a [`Format`] value.
    pub const fn from_byte(byte: u8) -> Self {
        match byte {
            0x00..=0x7f => Self::PositiveFixInt(byte - POSITIVE_FIXINT),
            0x80..=0x8f => Self::FixMap(byte - FIX_MAP),
            0x90..=0x9f => Self::FixArray(byte - FIX_ARRAY),
            0xa0..=0xbf => Self::FixStr(byte - FIX_STR),
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

    /// Return the marker byte wrapped in a single‑byte array.
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
