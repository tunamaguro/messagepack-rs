//! See https://github.com/msgpack/msgpack/blob/master/spec.md#formats

pub const POSITIVE_FIXINT: u8 = 0x00;
pub const NEGATIVE_FIXINT: u8 = 0xe0;
pub const FIX_MAP: u8 = 0x80;
pub const FIX_ARRAY: u8 = 0x90;
pub const FIX_STR: u8 = 0xa0;

pub const NIL: u8 = 0xc0;
pub const NEVER_USED: u8 = 0xc1;
pub const FALSE: u8 = 0xc2;
pub const TRUE: u8 = 0xc3;
pub const BIN8: u8 = 0xc4;
pub const BIN16: u8 = 0xc5;
pub const BIN32: u8 = 0xc6;
pub const EXT8: u8 = 0xc7;
pub const EXT16: u8 = 0xc8;
pub const EXT32: u8 = 0xc9;
pub const FLOAT32: u8 = 0xca;
pub const FLOAT64: u8 = 0xcb;
pub const UINT8: u8 = 0xcc;
pub const UINT16: u8 = 0xcd;
pub const UINT32: u8 = 0xce;
pub const UINT64: u8 = 0xcf;
pub const INT8: u8 = 0xd0;
pub const INT16: u8 = 0xd1;
pub const INT32: u8 = 0xd2;
pub const INT64: u8 = 0xd3;
pub const FIXEXT1: u8 = 0xd4;
pub const FIXEXT2: u8 = 0xd5;
pub const FIXEXT4: u8 = 0xd6;
pub const FIXEXT8: u8 = 0xd7;
pub const FIXEXT16: u8 = 0xd8;
pub const STR8: u8 = 0xd9;
pub const STR16: u8 = 0xda;
pub const STR32: u8 = 0xdb;
pub const ARRAY16: u8 = 0xdc;
pub const ARRAY32: u8 = 0xdd;
pub const MAP16: u8 = 0xde;
pub const MAP32: u8 = 0xdf;

pub(crate) const fn format_type(format: u8) -> u8 {
    match format {
        0x00..=0x7f => POSITIVE_FIXINT,
        0x80..=0x8f => FIX_MAP,
        0x90..=0x9f => FIX_ARRAY,
        0xa0..=0xbf => FIX_STR,
        0xe0..=0xff => NEGATIVE_FIXINT,
        NIL => NIL,
        NEVER_USED => NEVER_USED,
        FALSE => FALSE,
        TRUE => TRUE,
        BIN8 => BIN8,
        BIN16 => BIN16,
        BIN32 => BIN32,
        EXT8 => EXT8,
        EXT16 => EXT16,
        EXT32 => EXT32,
        FLOAT32 => FLOAT32,
        FLOAT64 => FLOAT64,
        UINT8 => UINT8,
        UINT16 => UINT16,
        UINT32 => UINT32,
        UINT64 => UINT64,
        INT8 => INT8,
        INT16 => INT16,
        INT32 => INT32,
        INT64 => INT64,
        FIXEXT1 => FIXEXT1,
        FIXEXT2 => FIXEXT2,
        FIXEXT4 => FIXEXT4,
        FIXEXT8 => FIXEXT8,
        FIXEXT16 => FIXEXT16,
        STR8 => STR8,
        STR16 => STR16,
        STR32 => STR32,
        ARRAY16 => ARRAY16,
        ARRAY32 => ARRAY32,
        MAP16 => MAP16,
        MAP32 => MAP32,
    }
}
