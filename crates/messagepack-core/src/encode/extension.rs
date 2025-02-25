use core::iter;

use super::{Encode, Error, Result};
use crate::{formats::Format, io::IoWrite};

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

impl<W: IoWrite> Encode<W> for ExtensionEncoder<'_> {
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
        let data_len = self.data.len();

        match data_len {
            1 => {
                let it = &mut Format::FixExt1
                    .into_iter()
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());
                writer.write_iter(it)?;
                Ok(2 + data_len)
            }
            2 => {
                let it = &mut Format::FixExt2
                    .into_iter()
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());
                writer.write_iter(it)?;
                Ok(2 + data_len)
            }
            4 => {
                let it = &mut Format::FixExt4
                    .into_iter()
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());
                writer.write_iter(it)?;
                Ok(2 + data_len)
            }
            8 => {
                let it = &mut Format::FixExt8
                    .into_iter()
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());
                writer.write_iter(it)?;
                Ok(2 + data_len)
            }
            16 => {
                let it = &mut Format::FixExt16
                    .into_iter()
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());
                writer.write_iter(it)?;
                Ok(2 + data_len)
            }
            0x00..=0xff => {
                let cast = data_len as u8;
                let it = &mut Format::Ext8
                    .into_iter()
                    .chain(cast.to_be_bytes())
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());
                writer.write_iter(it)?;
                Ok(3 + data_len)
            }
            0x100..=0xffff => {
                let cast = data_len as u16;
                let it = &mut Format::Ext16
                    .into_iter()
                    .chain(cast.to_be_bytes())
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());
                writer.write_iter(it)?;
                Ok(4 + data_len)
            }
            0x10000..0xffffffff => {
                let cast = data_len as u32;
                let it = &mut Format::Ext32
                    .into_iter()
                    .chain(cast.to_be_bytes())
                    .chain(iter::once(self.r#type))
                    .chain(self.data.iter().cloned());
                writer.write_iter(it)?;
                Ok(6 + data_len)
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
    #[case(0xd4,123,[0x12])]
    #[case(0xd5,123,[0x12,0x34])]
    #[case(0xd6,123,[0x12,0x34,0x56,0x78])]
    #[case(0xd7,123,[0x12;8])]
    #[case(0xd8,123,[0x12;16])]
    fn encode_ext_fixed<D: AsRef<[u8]>>(#[case] marker: u8, #[case] ty: u8, #[case] data: D) {
        let expected = marker
            .to_be_bytes()
            .iter()
            .chain(ty.to_be_bytes().iter())
            .chain(data.as_ref())
            .cloned()
            .collect::<Vec<_>>();

        let encoder = ExtensionEncoder::new(ty, data.as_ref());

        let mut buf = vec![];
        let n = encoder.encode(&mut buf).unwrap();

        assert_eq!(&buf, &expected);
        assert_eq!(n, expected.len());
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

        let mut buf = vec![];
        let n = encoder.encode(&mut buf).unwrap();

        assert_eq!(&buf, &expected);
        assert_eq!(n, expected.len());
    }
}
