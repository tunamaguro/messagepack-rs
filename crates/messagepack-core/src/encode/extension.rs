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
                writer.write_bytes(&[Format::FixExt1.as_byte(), self.r#type])?;
                writer.write_bytes(self.data)?;

                Ok(2 + data_len)
            }
            2 => {
                writer.write_bytes(&[Format::FixExt2.as_byte(), self.r#type])?;
                writer.write_bytes(self.data)?;

                Ok(2 + data_len)
            }
            4 => {
                writer.write_bytes(&[Format::FixExt4.as_byte(), self.r#type])?;
                writer.write_bytes(self.data)?;
                Ok(2 + data_len)
            }
            8 => {
                writer.write_bytes(&[Format::FixExt8.as_byte(), self.r#type])?;
                writer.write_bytes(self.data)?;

                Ok(2 + data_len)
            }
            16 => {
                writer.write_bytes(&[Format::FixExt16.as_byte(), self.r#type])?;
                writer.write_bytes(self.data)?;

                Ok(2 + data_len)
            }
            0x00..=0xff => {
                let cast = data_len as u8;
                writer.write_bytes(&[Format::Ext8.as_byte(), cast, self.r#type])?;
                writer.write_bytes(self.data)?;

                Ok(3 + data_len)
            }
            0x100..=0xffff => {
                let cast = (data_len as u16).to_be_bytes();
                writer.write_bytes(&[Format::Ext16.as_byte(), cast[0], cast[1], self.r#type])?;
                writer.write_bytes(self.data)?;

                Ok(4 + data_len)
            }
            0x10000..0xffffffff => {
                let cast = (data_len as u32).to_be_bytes();
                writer.write_bytes(&[
                    Format::Ext32.as_byte(),
                    cast[0],
                    cast[1],
                    cast[2],
                    cast[3],
                    self.r#type,
                ])?;
                writer.write_bytes(self.data)?;

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
