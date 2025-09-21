//! Encoding primitives for MessagePack.
//!
//! This module exposes the `Encode` trait and a number of small helper
//! encoders for arrays, maps, strings and binary data.

pub mod array;
pub mod bin;
pub mod bool;
pub mod float;
pub mod int;
pub mod map;
pub mod nil;
pub mod str;

/// Helper to encode raw binary blobs using `bin8/16/32` formats.
pub use bin::BinaryEncoder;
/// Helpers to encode MessagePack maps from various sources.
pub use map::{MapDataEncoder, MapEncoder, MapFormatEncoder, MapSliceEncoder};
/// Encode the MessagePack `nil` value.
pub use nil::NilEncoder;

use crate::{Format, io::IoWrite};

/// MessagePack encode error
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Error<T> {
    /// Error produced by the underlying writer.
    Io(T),
    /// Cannot mapped messagepack format
    InvalidFormat,
}

impl<T> From<T> for Error<T> {
    fn from(value: T) -> Self {
        Error::Io(value)
    }
}

impl<T: core::fmt::Display> core::fmt::Display for Error<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "{}", e),
            Error::InvalidFormat => write!(f, "Cannot encode value"),
        }
    }
}

impl<T: core::error::Error> core::error::Error for Error<T> {}

type Result<T, E> = ::core::result::Result<T, Error<E>>;

/// A type which can be encoded to MessagePack.
pub trait Encode<W>
where
    W: IoWrite,
{
    /// Encode this value to MessagePack and write bytes to `writer`.
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error>;
}

macro_rules! deref_impl {
    (
        $(#[$attr:meta])*
        <$($desc:tt)+
    ) => {
        $(#[$attr])*
        impl<$($desc)+
        {
            fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
                (**self).encode(writer)
            }
        }
    };
}

deref_impl! {
    <V, W> Encode<W> for &V
    where
        V: Encode<W>,
        W: IoWrite,
}

deref_impl! {
    <V, W> Encode<W> for &mut V
    where
        V: Encode<W>,
        W: IoWrite,
}

impl<W> Encode<W> for Format
where
    W: IoWrite,
{
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        writer.write(&self.as_slice())?;
        Ok(1)
    }
}
