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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    // Nil
    #[case(&[0xc0], Any::Nil)]
    // Bool
    #[case(&[0xc2], Any::Bool(false))]
    #[case(&[0xc3], Any::Bool(true))]
    // Positive FixInt (u8)
    #[case(&[0x00], Any::U8(0))]
    #[case(&[0x7f], Any::U8(127))]
    // Uint8
    #[case(&[0xcc, 0x80], Any::U8(128))]
    #[case(&[0xcc, 0xff], Any::U8(255))]
    // Uint16
    #[case(&[0xcd, 0x01, 0x00], Any::U16(256))]
    #[case(&[0xcd, 0xff, 0xff], Any::U16(65535))]
    // Uint32
    #[case(&[0xce, 0x00, 0x01, 0x00, 0x00], Any::U32(65536))]
    // Uint64
    #[case(&[0xcf, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00], Any::U64(4294967296))]
    // Negative FixInt (i8)
    #[case(&[0xff], Any::I8(-1))]
    #[case(&[0xe0], Any::I8(-32))]
    // Int8
    #[case(&[0xd0, 0xdf], Any::I8(-33))]
    #[case(&[0xd0, 0x80], Any::I8(-128))]
    // Int16
    #[case(&[0xd1, 0xff, 0x00], Any::I16(-256))]
    #[case(&[0xd1, 0x80, 0x00], Any::I16(-32768))]
    // Int32
    #[case(&[0xd2, 0xff, 0xff, 0x00, 0x00], Any::I32(-65536))]
    // Int64
    #[case(&[0xd3, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00], Any::I64(-4294967296))]
    // Float32
    #[case(&[0xca, 0x41, 0x20, 0x00, 0x00], Any::F32(10.0))]
    // Float64
    #[case(&[0xcb, 0x40, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], Any::F64(10.0))]
    // FixStr (empty)
    #[case(&[0xa0], Any::StrBorrowed(""))]
    // FixStr ("hi")
    #[case(&[0xa2, 0x68, 0x69], Any::StrBorrowed("hi"))]
    // Str8
    #[case(&[0xd9, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f], Any::StrBorrowed("hello"))]
    // Bin8 (empty)
    #[case(&[0xc4, 0x00], Any::BinBorrowed(&[]))]
    // Bin8
    #[case(&[0xc4, 0x03, 0x01, 0x02, 0x03], Any::BinBorrowed(&[1, 2, 3]))]
    // FixArray (empty)
    #[case(&[0x90], Any::Array(0))]
    // FixArray with 2 elements (nil, true)
    #[case(&[0x92, 0xc0, 0xc3], Any::Array(2))]
    // FixArray with nested array
    #[case(&[0x91, 0x91, 0xc0], Any::Array(1))]
    // FixMap (empty)
    #[case(&[0x80], Any::Map(0))]
    // FixMap with 1 pair (fixint 1 => true)
    #[case(&[0x81, 0x01, 0xc3], Any::Map(1))]
    // FixExt1
    #[case(&[0xd4, 0x01, 0xaa], Any::ExtBorrowed { r#type: 1, data: &[0xaa] })]
    // FixExt2
    #[case(&[0xd5, 0x02, 0xaa, 0xbb], Any::ExtBorrowed { r#type: 2, data: &[0xaa, 0xbb] })]
    // FixExt4
    #[case(&[0xd6, 0x03, 0x01, 0x02, 0x03, 0x04], Any::ExtBorrowed { r#type: 3, data: &[1, 2, 3, 4] })]
    // Ext8 (0-length)
    #[case(&[0xc7, 0x00, 0x05], Any::ExtBorrowed { r#type: 5, data: &[] })]
    // Ext8
    #[case(&[0xc7, 0x03, 0x0a, 0x01, 0x02, 0x03], Any::ExtBorrowed { r#type: 10, data: &[1, 2, 3] })]
    // NeverUsed
    fn decode_any_ok(#[case] input: &[u8], #[case] expected: Any<'_>) {
        let mut reader = crate::io::SliceReader::new(input);
        let value = Any::decode(&mut reader).unwrap();
        assert_eq!(value, expected);
    }
    #[rstest]
    #[case::never_used(&[0xc1])]
    fn decode_any_err(#[case] input: &[u8]) {
        let mut reader = crate::io::SliceReader::new(input);
        assert!(Any::decode(&mut reader).is_err());
    }
}
