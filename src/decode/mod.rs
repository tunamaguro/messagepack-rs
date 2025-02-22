use core::borrow::Borrow;

use crate::Format;

mod array;
mod bool;
mod float;
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

struct NbyteReader<const NBYTE: usize>;

macro_rules! impl_read {
    ($ty:ty,$size:expr) => {
        fn read<I, B>(buf: &mut I) -> Result<usize>
        where
            I: Iterator<Item = B>,
            B: Borrow<u8>,
        {
            let mut bytes = [0_u8; $size];
            let mut bytes_mut = bytes.iter_mut();

            for (to, byte) in bytes_mut.by_ref().zip(buf) {
                *to = *byte.borrow();
            }

            if bytes_mut.next().is_some() {
                return Err(Error::EofData);
            };
            usize::try_from(<$ty>::from_be_bytes(bytes)).map_err(|_| Error::InvalidData)
        }
    };
}

impl NbyteReader<1> {
    impl_read! {u8,1}
}

impl NbyteReader<2> {
    impl_read! {u16,2}
}
impl NbyteReader<4> {
    impl_read! {u32,4}
}
