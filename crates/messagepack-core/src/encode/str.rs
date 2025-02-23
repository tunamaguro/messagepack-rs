use core::ops::Deref;

use super::{Encode, Error, Result};
use crate::formats::Format;

pub struct StrFormatEncoder(pub usize);
impl Encode for StrFormatEncoder {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        match self.0 {
            0x00..=31 => {
                let cast = self.0 as u8;
                let it = Format::FixStr(cast).into_iter();
                buf.extend(it);
                Ok(1)
            }
            32..=0xff => {
                let cast = self.0 as u8;
                let it = Format::Str8.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(2)
            }
            0x100..=0xffff => {
                let cast = self.0 as u16;
                let it = Format::Str16.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(3)
            }
            0x10000..=0xffffffff => {
                let cast = self.0 as u32;
                let it = Format::Str32.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(5)
            }
            _ => Err(Error::InvalidFormat),
        }
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        match self.0 {
            0x00..=31 => {
                let cast = self.0 as u8;
                let it = Format::FixStr(cast).into_iter();
                let it = &mut Format::FixStr(cast).into_iter();
                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(1)
                } else {
                    Err(Error::BufferFull)
                }
            }
            32..=0xff => {
                let cast = self.0 as u8;
                let it = &mut Format::Str8.into_iter().chain(cast.to_be_bytes());
                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(2)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0x100..=0xffff => {
                let cast = self.0 as u16;
                let it = &mut Format::Str16.into_iter().chain(cast.to_be_bytes());
                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(3)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0x10000..=0xffffffff => {
                let cast = self.0 as u32;
                let it = &mut Format::Str32.into_iter().chain(cast.to_be_bytes());
                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(5)
                } else {
                    Err(Error::BufferFull)
                }
            }
            _ => Err(Error::InvalidFormat),
        }
    }
}

pub struct StrDataEncoder<'a>(pub &'a str);
impl Encode for StrDataEncoder<'_> {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let data_it = self.0.bytes();
        buf.extend(data_it);
        Ok(self.0.len())
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        let data_it = &mut self.0.bytes();
        for (byte, to) in data_it.zip(buf) {
            *to = byte
        }
        if data_it.next().is_none() {
            Ok(self.0.len())
        } else {
            Err(Error::BufferFull)
        }
    }
}
pub struct StrEncoder<'s>(pub &'s str);

impl<'s> Deref for StrEncoder<'s> {
    type Target = &'s str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Encode for StrEncoder<'_> {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let self_len = self.len();
        let format_len = StrFormatEncoder(self_len).encode(buf)?;
        let data_len = StrDataEncoder(self.0).encode(buf)?;

        Ok(format_len + data_len)
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        let self_len = self.len();
        let format_len = StrFormatEncoder(self_len).encode_to_iter_mut(buf)?;
        let data_len = StrDataEncoder(self.0).encode_to_iter_mut(buf)?;

        Ok(format_len + data_len)
    }
}

impl Encode for &str {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        StrEncoder(self).encode(buf)
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        StrEncoder(self).encode_to_iter_mut(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("Today",[0xa5, 0x54, 0x6f, 0x64, 0x61, 0x79])]
    fn encode_fixed_str<E: AsRef<[u8]> + Sized>(#[case] value: &str, #[case] expected: E) {
        let expected = expected.as_ref();
        let encoder = StrEncoder(value);
        {
            let mut buf = vec![];
            let n = encoder.encode(&mut buf).unwrap();
            assert_eq!(buf, expected);
            assert_eq!(n, expected.len());
        }

        {
            let mut buf = vec![0xff; core::mem::size_of::<E>()];
            let n = encoder.encode_to_slice(buf.as_mut_slice()).unwrap();
            assert_eq!(&buf, expected);
            assert_eq!(n, expected.len());
        }
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
        {
            let mut buf = vec![];
            let n = encoder.encode(&mut buf).unwrap();

            assert_eq!(&buf, &expected);
            assert_eq!(n, expected.len());
        }

        {
            let mut buf = vec![0xff; expected.len()];
            let n = encoder.encode_to_slice(buf.as_mut_slice()).unwrap();
            assert_eq!(&buf, &expected);
            assert_eq!(n, expected.len());
        }
    }
}
