use core::borrow::Borrow;

/// Messagepack Encode Error
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Error {
    /// Invalid Type
    InvalidType,
    /// Invalid data
    InvalidData,
    /// Unexpected type
    UnexpectedType,
    /// Eof while decode type
    EofType,
    /// Eof while decode data
    EofData,
}

type Result<T> = ::core::result::Result<T, Error>;

pub trait Decode: Sized {
    fn decode<I, B>(buf: &mut I) -> Result<Self>
    where
        I: Iterator<Item = B>,
        B: Borrow<u8>;
}
