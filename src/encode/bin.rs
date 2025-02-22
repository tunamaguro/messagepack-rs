use super::{Encode, Error, Result};
use crate::formats::Format;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BinaryEncoder<'blob>(pub &'blob [u8]);

impl<'blob> core::ops::Deref for BinaryEncoder<'blob> {
    type Target = &'blob [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Encode for BinaryEncoder<'_> {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let self_len = self.len();
        let format_len = match self_len {
            0x00..=0xff => {
                let cast = self_len as u8;
                let it = Format::Bin8.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(2)
            }
            0x100..=0xffff => {
                let cast = self_len as u16;
                let it = Format::Bin16.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(3)
            }
            0x10000..=0xffffffff => {
                let cast = self_len as u32;
                let it = Format::Bin32.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(5)
            }
            _ => Err(Error::InvalidFormat),
        }?;

        buf.extend(self.iter().cloned());
        Ok(format_len + self_len)
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        let self_len = self.len();
        let format_len = match self_len {
            0x00..=0xff => {
                const SIZE: usize = 2;
                let cast = self_len as u8;
                let it = &mut Format::Bin8.into_iter().chain(cast.to_be_bytes());
                for (byte, to) in it.zip(buf.by_ref()) {
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
                let it = &mut Format::Bin16.into_iter().chain(cast.to_be_bytes());
                for (byte, to) in it.zip(buf.by_ref()) {
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
                let it = &mut Format::Bin32.into_iter().chain(cast.to_be_bytes());

                for (byte, to) in it.zip(buf.by_ref()) {
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

        let it = &mut self.iter();

        for (byte, to) in it.zip(buf) {
            *to = *byte
        }

        if it.next().is_none() {
            Ok(format_len + self_len)
        } else {
            Err(Error::BufferFull)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0xc4_u8.to_be_bytes(), 255_u8.to_be_bytes(),[0x12;255])]
    #[case(0xc5_u8.to_be_bytes(), 65535_u16.to_be_bytes(),[0x34;65535])]
    #[case(0xc6_u8.to_be_bytes(), 65536_u32.to_be_bytes(),[0x56;65536])]
    fn encode_str_sized<M: AsRef<[u8]>, S: AsRef<[u8]>, D: AsRef<[u8]>>(
        #[case] marker: M,
        #[case] size: S,
        #[case] data: D,
    ) {
        let expected = marker
            .as_ref()
            .iter()
            .chain(size.as_ref())
            .chain(data.as_ref())
            .cloned()
            .collect::<Vec<u8>>();

        let encoder = BinaryEncoder(data.as_ref());
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
