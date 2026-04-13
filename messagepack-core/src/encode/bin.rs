//! Binary encoders.

use super::{Encode, Error, Result};
use crate::{formats::Format, io::IoWrite};

/// Encoder for MessagePack binary values (`bin8/16/32`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BinaryEncoder<'blob>(pub &'blob [u8]);

impl<'blob> core::ops::Deref for BinaryEncoder<'blob> {
    type Target = &'blob [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Encode for BinaryEncoder<'_> {
    fn encode<W: IoWrite>(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        let self_len = self.len();
        let format_len = match self_len {
            0x00..=0xff => {
                let mut buf = [0u8; 2];
                let [marker, rest @ ..] = &mut buf;
                *marker = Format::Bin8.as_byte();
                *rest = (self_len as u8).to_be_bytes();
                writer.write(&buf)?;

                Ok(buf.len())
            }
            0x100..=0xffff => {
                let mut buf = [0u8; 3];
                let [marker, rest @ ..] = &mut buf;
                *marker = Format::Bin16.as_byte();
                *rest = (self_len as u16).to_be_bytes();
                writer.write(&buf)?;
                Ok(buf.len())
            }
            0x10000..=0xffffffff => {
                let mut buf = [0u8; 5];
                let [marker, rest @ ..] = &mut buf;
                *marker = Format::Bin32.as_byte();
                *rest = (self_len as u32).to_be_bytes();
                writer.write(&buf)?;

                Ok(buf.len())
            }
            _ => Err(Error::InvalidFormat),
        }?;

        writer.write(self.0)?;
        Ok(format_len + self_len)
    }
}

/// Trait for encoding MessagePack binary data.
pub trait EncodeBytes {
    /// Encode the value as a MessagePack binary.
    fn encode_bytes<W: IoWrite>(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error>;
}

impl EncodeBytes for &[u8] {
    fn encode_bytes<W: IoWrite>(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        BinaryEncoder(self).encode(writer)
    }
}

impl<const N: usize> EncodeBytes for [u8; N] {
    fn encode_bytes<W: IoWrite>(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        self.as_slice().encode_bytes(writer)
    }
}

impl<T> EncodeBytes for Option<T>
where
    T: EncodeBytes,
{
    fn encode_bytes<W: IoWrite>(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        match self {
            Some(value) => value.encode_bytes(writer),
            None => ().encode(writer),
        }
    }
}

#[cfg(feature = "alloc")]
mod alloc_impl {
    use super::*;

    impl EncodeBytes for alloc::vec::Vec<u8> {
        fn encode_bytes<W: IoWrite>(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
            self.as_slice().encode_bytes(writer)
        }
    }

    impl EncodeBytes for alloc::boxed::Box<[u8]> {
        fn encode_bytes<W: IoWrite>(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
            self.as_ref().encode_bytes(writer)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0xc4, 255_u8.to_be_bytes(),[0x12;255])]
    #[case(0xc5, 65535_u16.to_be_bytes(),[0x34;65535])]
    #[case(0xc6, 65536_u32.to_be_bytes(),[0x56;65536])]
    fn encode_str_sized<S: AsRef<[u8]>, D: AsRef<[u8]>>(
        #[case] marker: u8,
        #[case] size: S,
        #[case] data: D,
    ) {
        let expected = marker
            .to_be_bytes()
            .iter()
            .chain(size.as_ref())
            .chain(data.as_ref())
            .cloned()
            .collect::<Vec<u8>>();

        let encoder = BinaryEncoder(data.as_ref());

        let mut buf = vec![];
        let n = encoder.encode(&mut buf).unwrap();

        assert_eq!(&buf, &expected);
        assert_eq!(n, expected.len());
    }
}
