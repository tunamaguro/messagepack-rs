//! Decoding primitives for MessagePack.

use crate::{Format, io::IoRead};

mod array;
pub use array::ArrayDecoder;
mod bin;
pub use bin::ReferenceDecoder;
#[cfg(feature = "alloc")]
pub use bin::BinOwnedDecoder;
mod bool;
mod float;
mod int;
mod map;
pub use map::MapDecoder;
mod nil;
pub use nil::NilDecoder;
mod str;
pub use str::{ReferenceStr, ReferenceStrDecoder};
mod timestamp;

/// MessagePack decode error
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Error<E> {
    /// Invalid data
    InvalidData,
    /// Unexpected format
    UnexpectedFormat,
    /// Unexpected end of data
    UnexpectedEof,
    /// Io error while decode format
    Io(E),
}

impl<E> core::fmt::Display for Error<E>
where
    E: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::InvalidData => write!(f, "Cannot decode invalid data"),
            Error::UnexpectedFormat => write!(f, "Unexpected format found"),
            Error::UnexpectedEof => write!(f, "Unexpected end of data"),
            Error::Io(e) => e.fmt(f),
        }
    }
}

impl<E> core::error::Error for Error<E>
where
    E: core::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

/// Decode a value from MessagePack.
///
/// Returned values may borrow from the reader's buffer with lifetime
/// `'a` (bounded by `'de`).
pub trait Decode<'de> {
    /// The decoded value (may borrow for `'a`).
    type Value<'a>: Sized
    where
        Self: 'a,
        'de: 'a;
    /// Decode the next value.
    fn decode<'a, R>(reader: &'a mut R) -> Result<Self::Value<'a>, Error<R::Error>>
    where
        R: IoRead<'de>,
        'de: 'a,
    {
        // Avoid recursive call when `Self = Format` by decoding the marker
        // via `DecodeBorrowed` explicitly.
        let format = <Format as DecodeBorrowed<'de>>::decode_borrowed(reader)?;
        Self::decode_with_format(format, reader)
    }

    /// Decode with a previously read `Format`.
    fn decode_with_format<'a, R>(
        format: Format,
        reader: &'a mut R,
    ) -> Result<Self::Value<'a>, Error<R::Error>>
    where
        R: IoRead<'de>,
        'de: 'a;
}

/// Decode a value whose borrows are bounded by `'de`.
///
/// Implementations must not return references to the reader's transient buffer.
pub trait DecodeBorrowed<'de> {
    /// The decoded value.
    type Value;

    /// Decode the next value.
    fn decode_borrowed<R>(
        reader: &mut R,
    ) -> Result<<Self as DecodeBorrowed<'de>>::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        let format = <Format as DecodeBorrowed<'de>>::decode_borrowed(reader)?;
        Self::decode_borrowed_with_format(format, reader)
    }

    /// Decode with a previously read `Format`.
    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> Result<<Self as DecodeBorrowed<'de>>::Value, Error<R::Error>>
    where
        R: IoRead<'de>;
}

impl<'de, T> Decode<'de> for T
where
    T: DecodeBorrowed<'de>,
    T: 'de,
{
    type Value<'a>
        = <T as DecodeBorrowed<'de>>::Value
    where
        Self: 'a,
        'de: 'a;

    fn decode_with_format<'a, R>(
        format: Format,
        reader: &'a mut R,
    ) -> Result<Self::Value<'a>, Error<R::Error>>
    where
        R: IoRead<'de>,
        'de: 'a,
    {
        <T as DecodeBorrowed<'de>>::decode_borrowed_with_format(format, reader)
    }
}

impl<'de> DecodeBorrowed<'de> for Format {
    type Value = Self;
    fn decode_borrowed<R>(reader: &mut R) -> Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        let b = reader.read_slice(1).map_err(Error::Io)?;
        let byte: [u8; 1] = b.as_bytes().try_into().map_err(|_| Error::UnexpectedEof)?;

        Ok(Self::from_byte(byte[0]))
    }

    fn decode_borrowed_with_format<R>(
        format: Format,
        _reader: &mut R,
    ) -> Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        Ok(format)
    }
}

/// Helper to read a fixed number of big‑endian bytes and return them as `usize`.
pub struct NbyteReader<const NBYTE: usize>;

macro_rules! impl_read {
    ($ty:ty) => {
        /// Read the next big‑endian integer of type `$ty` and return it as
        /// `usize` from `reader`.
        pub fn read<'de, R>(reader: &mut R) -> core::result::Result<usize, Error<R::Error>>
        where
            R: IoRead<'de>,
        {
            const SIZE: usize = core::mem::size_of::<$ty>();
            let bytes = reader.read_slice(SIZE).map_err(Error::Io)?;
            let slice = bytes.as_bytes();
            let data: [u8; SIZE] = slice.try_into().map_err(|_| Error::UnexpectedEof)?;
            let val =
                usize::try_from(<$ty>::from_be_bytes(data)).map_err(|_| Error::InvalidData)?;
            Ok(val)
        }
    };
}

impl NbyteReader<1> {
    impl_read! {u8}
}

impl NbyteReader<2> {
    impl_read! {u16}
}
impl NbyteReader<4> {
    impl_read! {u32}
}
