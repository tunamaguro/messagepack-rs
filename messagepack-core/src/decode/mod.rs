use crate::Format;

mod array;
pub use array::ArrayDecoder;
mod bin;
pub use bin::BinDecoder;
mod bool;
mod float;
mod int;
mod map;
pub use map::MapDecoder;
mod nil;
pub use nil::NilDecoder;
mod str;
pub use str::StrDecoder;
mod timestamp;

/// Messagepack Encode Error
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Error {
    /// Invalid data
    InvalidData,
    /// Unexpected format
    UnexpectedFormat,
    /// Eof while decode format
    EofFormat,
    /// Eof while decode data
    EofData,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::InvalidData => write!(f, "Cannot decode invalid data"),
            Error::UnexpectedFormat => write!(f, "Unexpected format found"),
            Error::EofFormat => write!(f, "EOF while parse format"),
            Error::EofData => write!(f, "EOF while parse data"),
        }
    }
}

impl core::error::Error for Error {}

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

pub struct NbyteReader<const NBYTE: usize>;

macro_rules! impl_read {
    ($ty:ty) => {
        pub fn read(buf: &[u8]) -> Result<(usize, &[u8])> {
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
