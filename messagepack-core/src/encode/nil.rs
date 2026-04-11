//! Nil encoder.

use super::{Encode, Result};
use crate::{formats::Format, io::IoWrite};

/// Encode the MessagePack `nil` value.
pub struct NilEncoder;

impl<W: IoWrite> Encode<W> for NilEncoder {
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        writer.write(&Format::Nil.as_slice())?;
        Ok(1)
    }
}

impl<W: IoWrite> Encode<W> for () {
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        NilEncoder.encode(writer)
    }
}

impl<W, V> Encode<W> for Option<V>
where
    W: IoWrite,
    V: Encode<W>,
{
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        match self {
            Some(other) => other.encode(writer),
            _ => NilEncoder.encode(writer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_nil() {
        let mut buf = vec![];
        NilEncoder.encode(&mut buf).unwrap();

        let expected: &[u8] = &[0xc0];
        assert_eq!(&buf, expected);
    }

    #[test]
    fn encode_none() {
        let mut buf = vec![];
        let option: Option<i32> = None;
        option.encode(&mut buf).unwrap();

        let expected: &[u8] = &[0xc0];
        assert_eq!(&buf, expected);
    }

    #[test]
    fn encode_some() {
        let mut buf = vec![];
        let option: Option<u8> = Some(0x80);
        option.encode(&mut buf).unwrap();

        let expected: &[u8] = &[0xcc, 0x80];
        assert_eq!(&buf, expected);
    }
}
