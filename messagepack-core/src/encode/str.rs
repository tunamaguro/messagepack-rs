use core::ops::Deref;

use super::{Encode, Error, Result};
use crate::{formats::Format, io::IoWrite};

pub struct StrFormatEncoder(pub usize);
impl<W: IoWrite> Encode<W> for StrFormatEncoder {
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        match self.0 {
            0x00..=31 => {
                let cast = self.0 as u8;
                writer.write(&Format::FixStr(cast).as_slice())?;
                Ok(1)
            }
            32..=0xff => {
                let cast = self.0 as u8;
                writer.write(&Format::Str8.as_slice())?;
                writer.write(&cast.to_be_bytes())?;
                Ok(2)
            }
            0x100..=0xffff => {
                let cast = self.0 as u16;
                writer.write(&Format::Str16.as_slice())?;
                writer.write(&cast.to_be_bytes())?;
                Ok(3)
            }
            0x10000..=0xffffffff => {
                let cast = self.0 as u32;
                writer.write(&Format::Str32.as_slice())?;
                writer.write(&cast.to_be_bytes())?;
                Ok(5)
            }
            _ => Err(Error::InvalidFormat),
        }
    }
}

pub struct StrDataEncoder<'a>(pub &'a str);
impl<W: IoWrite> Encode<W> for StrDataEncoder<'_> {
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        let data = self.0.as_bytes();
        writer.write(data)?;
        Ok(self.0.len())
    }
}
pub struct StrEncoder<'s>(pub &'s str);

impl<'s> Deref for StrEncoder<'s> {
    type Target = &'s str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<W: IoWrite> Encode<W> for StrEncoder<'_> {
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        let self_len = self.len();
        let format_len = StrFormatEncoder(self_len).encode(writer)?;
        let data_len = StrDataEncoder(self.0).encode(writer)?;

        Ok(format_len + data_len)
    }
}

impl<W: IoWrite> Encode<W> for &str {
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        StrEncoder(self).encode(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("Today",[0xa5, 0x54, 0x6f, 0x64, 0x61, 0x79])]
    #[case("MessagePack",[0xab,0x4d,0x65,0x73,0x73,0x61,0x67,0x65,0x50,0x61,0x63,0x6b])]
    fn encode_fixed_str<E: AsRef<[u8]> + Sized>(#[case] value: &str, #[case] expected: E) {
        let expected = expected.as_ref();
        let encoder = StrEncoder(value);

        let mut buf = vec![];
        let n = encoder.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }

    #[rstest]
    #[case(0xd9, 255_u8.to_be_bytes(),255)]
    #[case(0xda, 65535_u16.to_be_bytes(),65535)]
    #[case(0xdb, 65536_u32.to_be_bytes(),65536)]
    fn encode_str_sized<L: AsRef<[u8]>>(#[case] marker: u8, #[case] size: L, #[case] len: usize) {
        let value = core::iter::repeat_n("a", len).collect::<String>();
        let expected = marker
            .to_be_bytes()
            .iter()
            .chain(size.as_ref())
            .cloned()
            .chain(value.chars().map(|c| c as u8))
            .collect::<Vec<u8>>();

        let encoder = StrEncoder(&value);

        let mut buf = vec![];
        let n = encoder.encode(&mut buf).unwrap();

        assert_eq!(&buf, &expected);
        assert_eq!(n, expected.len());
    }
}
