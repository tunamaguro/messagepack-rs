use super::{ExtensionRef, FixedExtension};
use super::{U16_MAX, U32_MAX};
use crate::encode::{self, Encode};
use crate::formats::Format;
use crate::io::IoWrite;

impl<'a, W: IoWrite> Encode<W> for ExtensionRef<'a> {
    fn encode(&self, writer: &mut W) -> core::result::Result<usize, encode::Error<W::Error>> {
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
            0x1_0000..=U32_MAX => {
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
            _ => Err(encode::Error::InvalidFormat),
        }
    }
}

impl<const N: usize, W: IoWrite> Encode<W> for FixedExtension<N> {
    fn encode(&self, writer: &mut W) -> core::result::Result<usize, encode::Error<W::Error>> {
        self.as_ref().encode(writer)
    }
}

#[cfg(feature = "alloc")]
impl<W: IoWrite> Encode<W> for super::owned::ExtensionOwned {
    fn encode(&self, writer: &mut W) -> core::result::Result<usize, encode::Error<W::Error>> {
        self.as_ref().encode(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
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

        let encoder = ExtensionRef::new(ty, data.as_ref());

        let mut buf = vec![];
        let n = encoder.encode(&mut buf).unwrap();

        assert_eq!(&buf, &expected);
        assert_eq!(n, expected.len());
    }

    #[rstest::rstest]
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

        let encoder = ExtensionRef::new(ty, data.as_ref());

        let mut buf = vec![];
        let n = encoder.encode(&mut buf).unwrap();

        assert_eq!(&buf, &expected);
        assert_eq!(n, expected.len());
    }
}
