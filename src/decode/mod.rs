use core::borrow::Borrow;

use crate::Format;

/// Messagepack Encode Error
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Error {
    /// Invalid format
    InvalidFormat,
    /// Invalid data
    InvalidData,
    /// Unexpected format
    UnexpectedFormat,
    /// Eof while decode format
    EofFormat,
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

impl Decode for Format {
    fn decode<I, B>(buf: &mut I) -> Result<Self>
    where
        I: Iterator<Item = B>,
        B: Borrow<u8>,
    {
        let binding = buf.next().ok_or(Error::EofFormat)?;
        let byte = binding.borrow();
        Ok(Self::from_byte(*byte))
    }
}
