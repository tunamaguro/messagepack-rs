pub(crate) mod array;
pub(crate) mod bin;
pub(crate) mod bool;
pub(crate) mod extension;
pub(crate) mod float;
pub(crate) mod int;
pub(crate) mod map;
pub(crate) mod nil;
pub(crate) mod str;

pub use array::ArrayEncoder;
pub use bin::BinaryEncoder;
pub use extension::ExtensionEncoder;
pub use map::{MapEncoder, MapSliceEncoder};

/// Messagepack Encode Error
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Error {
    /// buffer is full
    BufferFull,
    /// Cannot mapped messagepack format
    InvalidFormat,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::BufferFull => write!(f, "Buffer is full"),
            Error::InvalidFormat => write!(f, "Cannot encode value"),
        }
    }
}

impl core::error::Error for Error {}

type Result<T> = ::core::result::Result<T, Error>;

/// A type which can be encoded to MessagePack
pub trait Encode {
    /// encode to MessagePack
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>;

    /// encode to IterMut
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize>;

    /// encode to slice
    fn encode_to_slice(&self, buf: &mut [u8]) -> Result<usize> {
        self.encode_to_iter_mut(&mut buf.iter_mut())
    }
}

impl<V> Encode for &V
where
    V: Encode,
{
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        V::encode(self, buf)
    }

    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        V::encode_to_iter_mut(self, buf)
    }
}

impl<V> Encode for &mut V
where
    V: Encode,
{
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        V::encode(self, buf)
    }

    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        V::encode_to_iter_mut(self, buf)
    }
}
