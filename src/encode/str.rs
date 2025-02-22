use core::ops::Deref;

use super::{Encode, Error, Result};
use crate::formats::Format;

struct StrEncoder<'s>(pub &'s str);

impl<'s> Deref for StrEncoder<'s> {
    type Target = &'s str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'s> Encode for StrEncoder<'s> {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let self_len = self.len();
        let format_len = match self_len {
            0x00..=31 => {
                let cast = self_len as u8;
                let it = Format::FixStr(cast).into_iter();
                buf.extend(it);
                Ok(1)
            }
            32..=0xff => {
                let cast = self_len as u8;
                let it = Format::Str8.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(2)
            }
            0x100..=0xffff => {
                let cast = self_len as u16;
                let it = Format::Str16.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(3)
            }
            0x10000..=0xffffffff => {
                let cast = self_len as u32;
                let it = Format::Str32.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(5)
            }
            _ => Err(Error::InvalidFormat),
        }?;

        buf.extend(self.as_bytes().iter().cloned());
        Ok(format_len + self_len)
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        let self_len = self.len();
        let data_it = self.bytes();
        let format_len = match self_len {
            0x00..=31 => {
                const SIZE: usize = 1;
                let cast = self_len as u8;
                let it = &mut Format::FixStr(cast).into_iter().chain(data_it);
                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            32..=0xff => {
                const SIZE: usize = 2;
                let cast = self_len as u8;
                let it = &mut Format::Str8
                    .into_iter()
                    .chain(cast.to_be_bytes())
                    .chain(data_it);
                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0x100..=0xffff => {
                const SIZE: usize = 3;
                let cast = self_len as u16;
                let it = &mut Format::Str16
                    .into_iter()
                    .chain(cast.to_be_bytes())
                    .chain(data_it);
                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0x10000..=0xffffffff => {
                const SIZE: usize = 5;
                let cast = self_len as u32;
                let it = &mut Format::Str32
                    .into_iter()
                    .chain(cast.to_be_bytes())
                    .chain(data_it);
                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            _ => Err(Error::InvalidFormat),
        }?;
        Ok(format_len + self_len)
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
    #[case(0xd9_u8.to_be_bytes(), 255_u8.to_be_bytes(),255)]
    #[case(0xda_u8.to_be_bytes(), 65535_u16.to_be_bytes(),65535)]
    #[case(0xdb_u8.to_be_bytes(), 65536_u32.to_be_bytes(),65536)]
    fn encode_str_sized<M: AsRef<[u8]>, L: AsRef<[u8]>>(
        #[case] marker: M,
        #[case] size: L,
        #[case] len: usize,
    ) {
        let value = core::iter::repeat_n("a", len).collect::<String>();
        let expected = marker
            .as_ref()
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
