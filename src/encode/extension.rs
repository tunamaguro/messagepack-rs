use core::iter;

use super::{Encode, Error, Result};
use crate::formats::Format;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExtensionEncoder<'data> {
    r#type: u8,
    data: &'data [u8],
}

impl<'data> ExtensionEncoder<'data> {
    pub fn new(r#type: u8, data: &'data [u8]) -> Self {
        Self { r#type, data }
    }
}

impl Encode for ExtensionEncoder<'_> {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let data_len = self.data.len();

        match data_len {
            1 => {
                let it = &mut Format::FixExt1
                    .into_iter()
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());
                buf.extend(it);
                Ok(2 + data_len)
            }
            2 => {
                let it = &mut Format::FixExt2
                    .into_iter()
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());
                buf.extend(it);
                Ok(2 + data_len)
            }
            4 => {
                let it = &mut Format::FixExt4
                    .into_iter()
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());
                buf.extend(it);
                Ok(2 + data_len)
            }
            8 => {
                let it = &mut Format::FixExt8
                    .into_iter()
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());
                buf.extend(it);
                Ok(2 + data_len)
            }
            16 => {
                let it = &mut Format::FixExt16
                    .into_iter()
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());
                buf.extend(it);
                Ok(2 + data_len)
            }
            0x00..=0xff => {
                let cast = data_len as u8;
                let it = &mut Format::Ext8
                    .into_iter()
                    .chain(cast.to_be_bytes())
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());
                buf.extend(it);
                Ok(3 + data_len)
            }
            0x100..=0xffff => {
                let cast = data_len as u16;
                let it = &mut Format::Ext16
                    .into_iter()
                    .chain(cast.to_be_bytes())
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());
                buf.extend(it);
                Ok(4 + data_len)
            }
            0x10000..0xffffffff => {
                let cast = data_len as u32;
                let it = &mut Format::Ext32
                    .into_iter()
                    .chain(cast.to_be_bytes())
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());
                buf.extend(it);
                Ok(6 + data_len)
            }
            _ => Err(Error::InvalidFormat),
        }
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        let data_len = self.data.len();

        match data_len {
            1 => {
                const SIZE: usize = 2;
                let it = &mut Format::FixExt1
                    .into_iter()
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());

                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(SIZE + 1)
                } else {
                    Err(Error::BufferFull)
                }
            }
            2 => {
                const SIZE: usize = 2;
                let it = &mut Format::FixExt2
                    .into_iter()
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());

                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(SIZE + 2)
                } else {
                    Err(Error::BufferFull)
                }
            }
            4 => {
                const SIZE: usize = 2;
                let it = &mut Format::FixExt4
                    .into_iter()
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());

                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(SIZE + 4)
                } else {
                    Err(Error::BufferFull)
                }
            }
            8 => {
                const SIZE: usize = 2;
                let it = &mut Format::FixExt8
                    .into_iter()
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());

                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(SIZE + 8)
                } else {
                    Err(Error::BufferFull)
                }
            }
            16 => {
                const SIZE: usize = 2;
                let it = &mut Format::FixExt16
                    .into_iter()
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());

                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(SIZE + 16)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0x00..=0xff => {
                const SIZE: usize = 3;
                let cast = data_len as u8;

                let it = &mut Format::Ext8
                    .into_iter()
                    .chain(cast.to_be_bytes())
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());

                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(SIZE + data_len)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0x100..=0xffff => {
                const SIZE: usize = 4;
                let cast = data_len as u16;

                let it = &mut Format::Ext16
                    .into_iter()
                    .chain(cast.to_be_bytes())
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());

                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(SIZE + data_len)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0x10000..0xffffffff => {
                const SIZE: usize = 6;
                let cast = data_len as u32;

                let it = &mut Format::Ext32
                    .into_iter()
                    .chain(cast.to_be_bytes())
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());

                for (byte, to) in it.zip(buf) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(SIZE + data_len)
                } else {
                    Err(Error::BufferFull)
                }
            }
            _ => Err(Error::InvalidFormat),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0xd4_u8.to_be_bytes(),123,[0x12])]
    #[case(0xd5_u8.to_be_bytes(),123,[0x12,0x34])]
    #[case(0xd6_u8.to_be_bytes(),123,[0x12,0x34,0x56,0x78])]
    #[case(0xd7_u8.to_be_bytes(),123,[0x12;8])]
    #[case(0xd8_u8.to_be_bytes(),123,[0x12;16])]
    fn encode_ext_fixed<M: AsRef<[u8]>, D: AsRef<[u8]>>(
        #[case] marker: M,
        #[case] ty: u8,
        #[case] data: D,
    ) {
        let expected = marker
            .as_ref()
            .iter()
            .chain(ty.to_be_bytes().iter())
            .chain(data.as_ref())
            .cloned()
            .collect::<Vec<_>>();

        let encoder = ExtensionEncoder::new(ty, data.as_ref());
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

    #[rstest]
    #[case(0xc7_u8.to_be_bytes(),123,5u8.to_be_bytes(),[0x12;5])]
    #[case(0xc8_u8.to_be_bytes(),123,65535_u16.to_be_bytes(),[0x34;65535])]
    #[case(0xc9_u8.to_be_bytes(),123,65536_u32.to_be_bytes(),[0x56;65536])]
    fn encode_ext_sized<M: AsRef<[u8]>, S: AsRef<[u8]>, D: AsRef<[u8]>>(
        #[case] marker: M,
        #[case] ty: u8,
        #[case] size: S,
        #[case] data: D,
    ) {
        let expected = marker
            .as_ref()
            .iter()
            .chain(size.as_ref())
            .chain(ty.to_be_bytes().iter())
            .chain(data.as_ref())
            .cloned()
            .collect::<Vec<_>>();

        let encoder = ExtensionEncoder::new(ty, data.as_ref());
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
