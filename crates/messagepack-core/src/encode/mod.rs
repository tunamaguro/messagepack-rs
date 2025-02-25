pub mod array;
pub mod bin;
pub mod bool;
pub mod extension;
pub mod float;
pub mod int;
pub mod map;
pub mod nil;
pub mod str;

pub use array::{ArrayDataEncoder, ArrayEncoder, ArrayFormatEncoder};
pub use bin::BinaryEncoder;
pub use extension::ExtensionEncoder;
pub use map::{MapDataEncoder, MapEncoder, MapFormatEncoder, MapSliceEncoder};
pub use nil::NilEncoder;

use crate::{Format, io::IoWrite};

/// Messagepack Encode Error
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Error<T> {
    // io error
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

/// A type which can be encoded to MessagePack
pub trait Encode<W>
where
    W: IoWrite,
{
    /// encode to MessagePack
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error>;
}

impl<V, W> Encode<W> for &V
where
    V: Encode<W>,
    W: IoWrite,
{
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        Encode::encode(*self, writer)
    }
}

impl<V, W> Encode<W> for &mut V
where
    V: Encode<W>,
    W: IoWrite,
{
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        Encode::encode(*self, writer)
    }
}

impl<W> Encode<W> for Format
where
    W: IoWrite,
{
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        writer.write_iter(self.as_byte().to_be_bytes())?;
        Ok(1)
    }
}
