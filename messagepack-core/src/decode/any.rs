use crate::{
    Format,
    decode::{
        ArrayDecoder, Decode, DecodeBorrowed, Error, MapDecoder, ReferenceDecoder, ReferenceStr,
        ReferenceStrDecoder,
    },
    io::IoRead,
};

/// Skip a single MessagePack value from the reader.
///
/// This reads and discards one complete value (including nested containers).
/// Useful when encountering unknown map keys during decoding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Any<'de> {
    /// Nil
    Nil,
    /// true / false
    Bool(bool),
    /// positive fixint / uint8
    U8(u8),
    /// uint16
    U16(u16),
    /// uint32
    U32(u32),
    /// uint64
    U64(u64),
    /// negative fixint / int8
    I8(i8),
    /// int16
    I16(i16),
    /// int32
    I32(i32),
    /// int64
    I64(i64),
    /// float32
    F32(f32),
    /// float64
    F64(f64),
    /// fixstr / str8 / str16 / str32 (borrowed)
    StrBorrowed(&'de str),
    /// fixstr / str8 / str16 / str32 (copied)
    StrCopied(usize),
    /// bin8 / bin16 / bin32 (borrowed)
    BinBorrowed(&'de [u8]),
    /// bin8 / bin16 / bin32 (copied)
    BinCopied(usize),
    /// fixarray / array16 / array32
    Array(usize),
    /// fixmap / map16 / map32
    Map(usize),
    /// fixext1 / fixext2 / fixext4 / fixext8 / fixext16 / ext8 / ext16 / ext32 (borrowed)
    ExtBorrowed {
        /// extension type
        r#type: i8,
        /// extension data
        data: &'de [u8],
    },
    /// fixext1 / fixext2 / fixext4 / fixext8 / fixext16 / ext8 / ext16 / ext32 (copied)
    ExtCopied {
        /// extension type
        r#type: i8,
        /// extension data length
        len: usize,
    },
}

impl<'de> DecodeBorrowed<'de> for Any<'de> {
    type Value = Self;
    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> Result<<Self as DecodeBorrowed<'de>>::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        match format {
            Format::Nil => Ok(Any::Nil),
            Format::False | Format::True => bool::decode_with_format(format, reader).map(Any::Bool),

            Format::PositiveFixInt(_) | Format::Uint8 => {
                u8::decode_with_format(format, reader).map(Any::U8)
            }
            Format::Uint16 => u16::decode_with_format(format, reader).map(Any::U16),
            Format::Uint32 => u32::decode_with_format(format, reader).map(Any::U32),
            Format::Uint64 => u64::decode_with_format(format, reader).map(Any::U64),
            Format::NegativeFixInt(_) | Format::Int8 => {
                i8::decode_with_format(format, reader).map(Any::I8)
            }
            Format::Int16 => i16::decode_with_format(format, reader).map(Any::I16),
            Format::Int32 => i32::decode_with_format(format, reader).map(Any::I32),
            Format::Int64 => i64::decode_with_format(format, reader).map(Any::I64),

            Format::Float32 => f32::decode_with_format(format, reader).map(Any::F32),
            Format::Float64 => f64::decode_with_format(format, reader).map(Any::F64),

            Format::FixStr(_) | Format::Str8 | Format::Str16 | Format::Str32 => {
                ReferenceStrDecoder::decode_with_format(format, reader).map(|s| match s {
                    ReferenceStr::Borrowed(s) => Any::StrBorrowed(s),
                    ReferenceStr::Copied(s) => Any::StrCopied(s.len()),
                })
            }

            Format::Bin8 | Format::Bin16 | Format::Bin32 => {
                ReferenceDecoder::decode_with_format(format, reader).map(|b| match b {
                    crate::io::Reference::Borrowed(items) => Any::BinBorrowed(items),
                    crate::io::Reference::Copied(items) => Any::BinCopied(items.len()),
                })
            }

            Format::FixExt1
            | Format::FixExt2
            | Format::FixExt4
            | Format::FixExt8
            | Format::FixExt16
            | Format::Ext8
            | Format::Ext16
            | Format::Ext32 => {
                let (length, r#type) = crate::extension::read_ext_header(format, reader)?;
                reader
                    .read_slice(length)
                    .map_err(Error::Io)
                    .map(|data| match data {
                        crate::io::Reference::Borrowed(items) => Any::ExtBorrowed {
                            r#type,
                            data: items,
                        },
                        crate::io::Reference::Copied(items) => Any::ExtCopied {
                            r#type,
                            len: items.len(),
                        },
                    })
            }
            Format::FixArray(_) | Format::Array16 | Format::Array32 => {
                ArrayDecoder::<IterCounter, Any>::decode_with_format(format, reader)
                    .map(|counter| Any::Array(counter.count))
            }

            Format::FixMap(_) | Format::Map16 | Format::Map32 => {
                MapDecoder::<IterCounter, Any, Any>::decode_with_format(format, reader)
                    .map(|counter| Any::Map(counter.count))
            }
            Format::NeverUsed => Err(Error::UnexpectedFormat),
        }
    }
}

struct IterCounter {
    count: usize,
}

impl<'de> FromIterator<Any<'de>> for IterCounter {
    fn from_iter<T: IntoIterator<Item = Any<'de>>>(iter: T) -> Self {
        let count = iter.into_iter().count();
        Self { count }
    }
}

impl<'de> FromIterator<(Any<'de>, Any<'de>)> for IterCounter {
    fn from_iter<T: IntoIterator<Item = (Any<'de>, Any<'de>)>>(iter: T) -> Self {
        let count = iter.into_iter().count();
        Self { count }
    }
}
