use super::{Encode, Error, Result};
use crate::{formats::Format, io::IoWrite};

pub const U8_MAX: usize = u8::MAX as usize;
pub const U16_MAX: usize = u16::MAX as usize;
pub const U32_MAX: usize = u32::MAX as usize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExtensionEncoder<'data> {
    r#type: i8,
    data: &'data [u8],
}

impl<'data> ExtensionEncoder<'data> {
    pub fn new(r#type: i8, data: &'data [u8]) -> Self {
        Self { r#type, data }
    }

    pub fn to_format<E>(&self) -> Result<Format, E> {
        let format = match self.data.len() {
            1 => Format::FixExt1,
            2 => Format::FixExt2,
            4 => Format::FixExt4,
            8 => Format::FixExt8,
            16 => Format::FixExt16,
            0..U8_MAX => Format::Ext8,
            U8_MAX..U16_MAX => Format::Ext16,
            U16_MAX..U32_MAX => Format::Ext32,
            _ => return Err(Error::InvalidFormat),
        };
        Ok(format)
    }
}

impl<W: IoWrite> Encode<W> for ExtensionEncoder<'_> {
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
        let data_len = self.data.len();
        let type_byte = self.r#type.to_be_bytes()[0];

        match data_len {
            1 => {
                writer.write(&[Format::FixExt1.as_byte(), type_byte])?;
                writer.write(self.data)?;

                Ok(2 + data_len)
            }
            2 => {
                writer.write(&[Format::FixExt2.as_byte(), type_byte])?;
                writer.write(self.data)?;

                Ok(2 + data_len)
            }
            4 => {
                writer.write(&[Format::FixExt4.as_byte(), type_byte])?;
                writer.write(self.data)?;
                Ok(2 + data_len)
            }
            8 => {
                writer.write(&[Format::FixExt8.as_byte(), type_byte])?;
                writer.write(self.data)?;

                Ok(2 + data_len)
            }
            16 => {
                writer.write(&[Format::FixExt16.as_byte(), type_byte])?;
                writer.write(self.data)?;

                Ok(2 + data_len)
            }
            0..=0xff => {
                let cast = data_len as u8;
                writer.write(&[Format::Ext8.as_byte(), cast, type_byte])?;
                writer.write(self.data)?;

                Ok(3 + data_len)
            }
            0x100..=U16_MAX => {
                let cast = (data_len as u16).to_be_bytes();
                writer.write(&[Format::Ext16.as_byte(), cast[0], cast[1], type_byte])?;
                writer.write(self.data)?;

                Ok(4 + data_len)
            }
            0x10000..=U32_MAX => {
                let cast = (data_len as u32).to_be_bytes();
                writer.write(&[
                    Format::Ext32.as_byte(),
                    cast[0],
                    cast[1],
                    cast[2],
                    cast[3],
                    type_byte,
                ])?;
                writer.write(self.data)?;

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
    fn encode_ext_fixed<D: AsRef<[u8]>>(#[case] marker: u8, #[case] ty: i8, #[case] data: D) {
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
        #[case] ty: i8,
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
