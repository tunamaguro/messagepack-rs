mod bool;
mod float;
mod int;

/// Messagepack Encode Error
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Error {
    /// buffer is full
    BufferFull,
    /// Cannot mapped messagepack type
    InvalidType
}

type Result<T> = ::core::result::Result<T, Error>;

/// A type which can be encoded to MessagePack
pub trait Encode {
    /// encode to MessagePack
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>;

    /// encode to slice
    fn encode_to_slice(&self, buf: &mut [u8]) -> Result<usize>;
}
