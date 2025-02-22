use core::borrow::Borrow;

use crate::Format;

mod bool;
mod int;
mod nil;

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

pub trait Decode {
    type Value: Sized;
    // decode from iter
    fn decode<I, B>(buf: &mut I) -> Result<Self::Value>
    where
        I: Iterator<Item = B>,
        B: Borrow<u8>;

    // decode with format
    fn decode_with_format<I, B>(format: Format, buf: &mut I) -> Result<Self::Value>
    where
        I: Iterator<Item = B>,
        B: Borrow<u8>;
}

impl Decode for Format {
    type Value = Self;
    fn decode<I, B>(buf: &mut I) -> Result<Self::Value>
    where
        I: Iterator<Item = B>,
        B: Borrow<u8>,
    {
        let binding = buf.next().ok_or(Error::EofFormat)?;
        let byte = binding.borrow();
        Ok(Self::from_byte(*byte))
    }

    fn decode_with_format<I, B>(format: Format, buf: &mut I) -> Result<Self::Value>
    where
        I: Iterator<Item = B>,
        B: Borrow<u8>,
    {
        let _ = (format, buf);
        unreachable!()
    }
}
