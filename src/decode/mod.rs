use crate::Format;

mod array;
mod bool;
mod float;
mod int;
mod map;
mod nil;
mod str;

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

pub trait Decode<'a> {
    type Value: Sized;
    // decode from buf and return (value,rest)
    fn decode(buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])>;

    // decode with format
    fn decode_with_format(format: Format, buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])>;
}

impl<'a> Decode<'a> for Format {
    type Value = Self;
    fn decode(buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (first, rest) = buf.split_first().ok_or(Error::EofFormat)?;

        Ok((Self::from_byte(*first), rest))
    }

    fn decode_with_format(format: Format, buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let _ = (format, buf);
        unreachable!()
    }
}

struct NbyteReader<const NBYTE: usize>;

macro_rules! impl_read {
    ($ty:ty) => {
        fn read(buf: &[u8]) -> Result<(usize, &[u8])> {
            const SIZE: usize = core::mem::size_of::<$ty>();
            let (data, rest) = buf.split_at_checked(SIZE).ok_or(Error::EofData)?;
            let data: [u8; SIZE] = data.try_into().map_err(|_| Error::EofData)?;
            let val =
                usize::try_from(<$ty>::from_be_bytes(data)).map_err(|_| Error::InvalidData)?;
            Ok((val, rest))
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
