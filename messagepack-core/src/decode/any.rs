//! Decode arbitrary MessagePack values into a compact summary enum.

use super::{DecodeBorrowed, Error, NbyteReader};
use crate::{
    Format,
    io::{IoRead, Reference},
};

/// String value summary.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnyStr<'de> {
    /// Borrowed UTF-8 string.
    Borrowed(&'de str),
    /// Length only (bytes).
    Len(usize),
}

/// Binary value summary.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnyBin<'de> {
    /// Borrowed raw bytes.
    Borrowed(&'de [u8]),
    /// Length only (bytes).
    Len(usize),
}

/// Extension value summary.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnyExt<'de> {
    /// Borrowed extension payload.
    Borrowed {
        /// Extension type code.
        r#type: i8,
        /// Borrowed payload bytes.
        data: &'de [u8],
    },
    /// Type and payload length only.
    Len {
        /// Extension type code.
        r#type: i8,
        /// Payload length in bytes.
        len: usize,
    },
}

/// A compact, recursively-skipping representation of one decoded MessagePack value.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Any<'de> {
    /// `nil`
    Nil,
    /// `bool`
    Bool(bool),
    /// Unsigned integer value.
    UInt(u64),
    /// Signed integer value.
    Int(i64),
    /// 32-bit float.
    F32(f32),
    /// 64-bit float.
    F64(f64),
    /// String summary.
    Str(AnyStr<'de>),
    /// Binary summary.
    Bin(AnyBin<'de>),
    /// Number of elements in array.
    Array(usize),
    /// Number of entries in map.
    Map(usize),
    /// Extension summary.
    Ext(AnyExt<'de>),
}

#[cfg(feature = "alloc")]
impl<'de> DecodeBorrowed<'de> for Any<'de> {
    type Value = Self;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        match format {
            Format::Nil => Ok(Self::Nil),
            Format::False => Ok(Self::Bool(false)),
            Format::True => Ok(Self::Bool(true)),

            Format::PositiveFixInt(v) => Ok(Self::UInt(v.into())),
            Format::Uint8 => Ok(Self::UInt(read_u8(reader)?.into())),
            Format::Uint16 => Ok(Self::UInt(read_u16(reader)?.into())),
            Format::Uint32 => Ok(Self::UInt(read_u32(reader)?.into())),
            Format::Uint64 => Ok(Self::UInt(read_u64(reader)?)),

            Format::NegativeFixInt(v) => Ok(Self::Int(v.into())),
            Format::Int8 => Ok(Self::Int(read_i8(reader)?.into())),
            Format::Int16 => Ok(Self::Int(read_i16(reader)?.into())),
            Format::Int32 => Ok(Self::Int(read_i32(reader)?.into())),
            Format::Int64 => Ok(Self::Int(read_i64(reader)?)),

            Format::Float32 => Ok(Self::F32(f32::decode_borrowed_with_format(format, reader)?)),
            Format::Float64 => Ok(Self::F64(f64::decode_borrowed_with_format(format, reader)?)),

            Format::FixStr(_) | Format::Str8 | Format::Str16 | Format::Str32 => {
                decode_str(format, reader)
            }

            Format::Bin8 | Format::Bin16 | Format::Bin32 => {
                let len = bin_len(format, reader)?;
                match reader.read_slice(len).map_err(Error::Io)? {
                    Reference::Borrowed(v) => Ok(Self::Bin(AnyBin::Borrowed(v))),
                    Reference::Copied(_) => Ok(Self::Bin(AnyBin::Len(len))),
                }
            }

            Format::FixArray(_) | Format::Array16 | Format::Array32 => {
                let len = array_len(format, reader)?;
                for _ in 0..len {
                    let _ = Self::decode_borrowed(reader)?;
                }
                Ok(Self::Array(len))
            }

            Format::FixMap(_) | Format::Map16 | Format::Map32 => {
                let len = map_len(format, reader)?;
                for _ in 0..len {
                    let _ = Self::decode_borrowed(reader)?;
                    let _ = Self::decode_borrowed(reader)?;
                }
                Ok(Self::Map(len))
            }

            Format::FixExt1
            | Format::FixExt2
            | Format::FixExt4
            | Format::FixExt8
            | Format::FixExt16
            | Format::Ext8
            | Format::Ext16
            | Format::Ext32 => {
                let (len, r#type) = crate::extension::read_ext_header(format, reader)?;
                match reader.read_slice(len).map_err(Error::Io)? {
                    Reference::Borrowed(data) => Ok(Self::Ext(AnyExt::Borrowed { r#type, data })),
                    Reference::Copied(_) => Ok(Self::Ext(AnyExt::Len { r#type, len })),
                }
            }

            Format::NeverUsed => Err(Error::UnexpectedFormat),
        }
    }
}

#[cfg(feature = "alloc")]
fn decode_str<'de, R>(format: Format, reader: &mut R) -> Result<Any<'de>, Error<R::Error>>
where
    R: IoRead<'de>,
{
    let len = match format {
        Format::FixStr(n) => n.into(),
        Format::Str8 => NbyteReader::<1>::read(reader)?,
        Format::Str16 => NbyteReader::<2>::read(reader)?,
        Format::Str32 => NbyteReader::<4>::read(reader)?,
        _ => return Err(Error::UnexpectedFormat),
    };

    match reader.read_slice(len).map_err(Error::Io)? {
        Reference::Borrowed(bytes) => {
            let s = core::str::from_utf8(bytes).map_err(|_| Error::InvalidData)?;
            Ok(Any::Str(AnyStr::Borrowed(s)))
        }
        Reference::Copied(bytes) => {
            core::str::from_utf8(bytes).map_err(|_| Error::InvalidData)?;
            Ok(Any::Str(AnyStr::Len(len)))
        }
    }
}

#[cfg(feature = "alloc")]
fn bin_len<'de, R>(format: Format, reader: &mut R) -> Result<usize, Error<R::Error>>
where
    R: IoRead<'de>,
{
    match format {
        Format::Bin8 => NbyteReader::<1>::read(reader),
        Format::Bin16 => NbyteReader::<2>::read(reader),
        Format::Bin32 => NbyteReader::<4>::read(reader),
        _ => Err(Error::UnexpectedFormat),
    }
}

#[cfg(feature = "alloc")]
fn array_len<'de, R>(format: Format, reader: &mut R) -> Result<usize, Error<R::Error>>
where
    R: IoRead<'de>,
{
    match format {
        Format::FixArray(v) => Ok(v.into()),
        Format::Array16 => NbyteReader::<2>::read(reader),
        Format::Array32 => NbyteReader::<4>::read(reader),
        _ => Err(Error::UnexpectedFormat),
    }
}

#[cfg(feature = "alloc")]
fn map_len<'de, R>(format: Format, reader: &mut R) -> Result<usize, Error<R::Error>>
where
    R: IoRead<'de>,
{
    match format {
        Format::FixMap(v) => Ok(v.into()),
        Format::Map16 => NbyteReader::<2>::read(reader),
        Format::Map32 => NbyteReader::<4>::read(reader),
        _ => Err(Error::UnexpectedFormat),
    }
}

#[cfg(feature = "alloc")]
fn read_u8<'de, R>(reader: &mut R) -> Result<u8, Error<R::Error>>
where
    R: IoRead<'de>,
{
    let bytes = reader.read_slice(1).map_err(Error::Io)?;
    let data: [u8; 1] = bytes
        .as_bytes()
        .try_into()
        .map_err(|_| Error::UnexpectedEof)?;
    Ok(data[0])
}

#[cfg(feature = "alloc")]
fn read_i8<'de, R>(reader: &mut R) -> Result<i8, Error<R::Error>>
where
    R: IoRead<'de>,
{
    Ok(read_u8(reader)? as i8)
}

#[cfg(feature = "alloc")]
fn read_u16<'de, R>(reader: &mut R) -> Result<u16, Error<R::Error>>
where
    R: IoRead<'de>,
{
    let bytes = reader.read_slice(2).map_err(Error::Io)?;
    let data: [u8; 2] = bytes
        .as_bytes()
        .try_into()
        .map_err(|_| Error::UnexpectedEof)?;
    Ok(u16::from_be_bytes(data))
}

#[cfg(feature = "alloc")]
fn read_i16<'de, R>(reader: &mut R) -> Result<i16, Error<R::Error>>
where
    R: IoRead<'de>,
{
    Ok(read_u16(reader)? as i16)
}

#[cfg(feature = "alloc")]
fn read_u32<'de, R>(reader: &mut R) -> Result<u32, Error<R::Error>>
where
    R: IoRead<'de>,
{
    let bytes = reader.read_slice(4).map_err(Error::Io)?;
    let data: [u8; 4] = bytes
        .as_bytes()
        .try_into()
        .map_err(|_| Error::UnexpectedEof)?;
    Ok(u32::from_be_bytes(data))
}

#[cfg(feature = "alloc")]
fn read_i32<'de, R>(reader: &mut R) -> Result<i32, Error<R::Error>>
where
    R: IoRead<'de>,
{
    Ok(read_u32(reader)? as i32)
}

#[cfg(feature = "alloc")]
fn read_u64<'de, R>(reader: &mut R) -> Result<u64, Error<R::Error>>
where
    R: IoRead<'de>,
{
    let bytes = reader.read_slice(8).map_err(Error::Io)?;
    let data: [u8; 8] = bytes
        .as_bytes()
        .try_into()
        .map_err(|_| Error::UnexpectedEof)?;
    Ok(u64::from_be_bytes(data))
}

#[cfg(feature = "alloc")]
fn read_i64<'de, R>(reader: &mut R) -> Result<i64, Error<R::Error>>
where
    R: IoRead<'de>,
{
    Ok(read_u64(reader)? as i64)
}
