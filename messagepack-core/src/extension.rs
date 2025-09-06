use crate::decode::{self, NbyteReader};
use crate::encode;
use crate::{Decode, Encode, formats::Format, io::IoWrite};

const U8_MAX: usize = u8::MAX as usize;
const U16_MAX: usize = u16::MAX as usize;
const U32_MAX: usize = u32::MAX as usize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExtensionRef<'a> {
    pub r#type: i8,
    pub data: &'a [u8],
}

impl<'a> ExtensionRef<'a> {
    pub fn new(r#type: i8, data: &'a [u8]) -> Self {
        Self { r#type, data }
    }

    pub fn to_format<E>(&self) -> core::result::Result<Format, encode::Error<E>> {
        let format = match self.data.len() {
            1 => Format::FixExt1,
            2 => Format::FixExt2,
            4 => Format::FixExt4,
            8 => Format::FixExt8,
            16 => Format::FixExt16,
            0..U8_MAX => Format::Ext8,
            U8_MAX..U16_MAX => Format::Ext16,
            U16_MAX..U32_MAX => Format::Ext32,
            _ => return Err(encode::Error::InvalidFormat),
        };
        Ok(format)
    }
}

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
            _ => Err(encode::Error::InvalidFormat),
        }
    }
}

impl<'a> Decode<'a> for ExtensionRef<'a> {
    type Value = ExtensionRef<'a>;

    fn decode(buf: &'a [u8]) -> core::result::Result<(Self::Value, &'a [u8]), decode::Error> {
        let (format, buf) = Format::decode(buf)?;
        match format {
            Format::FixExt1
            | Format::FixExt2
            | Format::FixExt4
            | Format::FixExt8
            | Format::FixExt16
            | Format::Ext8
            | Format::Ext16
            | Format::Ext32 => Self::decode_with_format(format, buf),
            _ => Err(decode::Error::UnexpectedFormat),
        }
    }

    fn decode_with_format(
        format: Format,
        buf: &'a [u8],
    ) -> core::result::Result<(Self::Value, &'a [u8]), decode::Error> {
        let (ext_type, buf) = buf.split_first().ok_or(decode::Error::EofData)?;
        let (len, buf) = match format {
            Format::FixExt1 => (1, buf),
            Format::FixExt2 => (2, buf),
            Format::FixExt4 => (4, buf),
            Format::FixExt8 => (8, buf),
            Format::FixExt16 => (16, buf),
            Format::Ext8 => NbyteReader::<1>::read(buf)?,
            Format::Ext16 => NbyteReader::<2>::read(buf)?,
            Format::Ext32 => NbyteReader::<4>::read(buf)?,
            _ => return Err(decode::Error::UnexpectedFormat),
        };
        let (data, rest) = buf.split_at_checked(len).ok_or(decode::Error::EofData)?;
        let ext = ExtensionRef {
            r#type: (*ext_type) as i8,
            data,
        };
        Ok((ext, rest))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FixedExtension<const N: usize> {
    pub r#type: i8,
    len: usize,
    data: [u8; N],
}

impl<const N: usize> FixedExtension<N> {
    pub fn new(r#type: i8, data: &[u8]) -> Option<Self> {
        if data.len() > N {
            return None;
        }
        let mut buf = [0u8; N];
        buf[..data.len()].copy_from_slice(data);
        Some(Self {
            r#type,
            len: data.len(),
            data: buf,
        })
    }

    pub fn new_fixed(r#type: i8, data: [u8; N]) -> Self {
        Self {
            r#type,
            len: N,
            data,
        }
    }

    pub fn as_ref(&self) -> ExtensionRef<'_> {
        ExtensionRef {
            r#type: self.r#type,
            data: &self.data[..self.len],
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..self.len]
    }
}

impl<const N: usize, W: IoWrite> Encode<W> for FixedExtension<N> {
    fn encode(&self, writer: &mut W) -> core::result::Result<usize, encode::Error<W::Error>> {
        self.as_ref().encode(writer)
    }
}

impl<'a, const N: usize> Decode<'a> for FixedExtension<N> {
    type Value = FixedExtension<N>;

    fn decode(buf: &'a [u8]) -> core::result::Result<(Self::Value, &'a [u8]), decode::Error> {
        let (ext, rest) = ExtensionRef::decode(buf)?;
        if ext.data.len() > N {
            return Err(decode::Error::InvalidData);
        }
        let mut buf_arr = [0u8; N];
        buf_arr[..ext.data.len()].copy_from_slice(ext.data);
        Ok((
            FixedExtension {
                r#type: ext.r#type,
                len: ext.data.len(),
                data: buf_arr,
            },
            rest,
        ))
    }

    fn decode_with_format(
        format: Format,
        buf: &'a [u8],
    ) -> core::result::Result<(Self::Value, &'a [u8]), decode::Error> {
        let (ext, rest) = ExtensionRef::decode_with_format(format, buf)?;
        if ext.data.len() > N {
            return Err(decode::Error::InvalidData);
        }
        let mut buf_arr = [0u8; N];
        buf_arr[..ext.data.len()].copy_from_slice(ext.data);
        Ok((
            FixedExtension {
                r#type: ext.r#type,
                len: ext.data.len(),
                data: buf_arr,
            },
            rest,
        ))
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

        let encoder = ExtensionRef::new(ty, data.as_ref());

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

        let encoder = ExtensionRef::new(ty, data.as_ref());

        let mut buf = vec![];
        let n = encoder.encode(&mut buf).unwrap();

        assert_eq!(&buf, &expected);
        assert_eq!(n, expected.len());
    }

    const TIMESTAMP32: &[u8] = &[0xd6, 0xff, 0x62, 0x15, 0x62, 0x1e];

    #[test]
    fn decode_fix_ext4() {
        let (ext, rest) = ExtensionRef::decode(TIMESTAMP32).unwrap();
        let expect_type = -1;
        let expect_data = 1645568542;
        assert_eq!(ext.r#type, expect_type);
        let data_u32 = u32::from_be_bytes(ext.data.try_into().unwrap());
        assert_eq!(data_u32, expect_data);
        assert_eq!(rest.len(), 0);
    }

    #[test]
    fn fixed_extension_roundtrip() {
        let data = [1u8, 2, 3, 4];
        let ext = FixedExtension::<8>::new(5, &data).unwrap();
        let mut buf = vec![];
        ext.encode(&mut buf).unwrap();
        let (decoded, rest) = FixedExtension::<8>::decode(&buf).unwrap();
        assert_eq!(decoded.r#type, 5);
        assert_eq!(decoded.data(), &data);
        assert!(rest.is_empty());
    }
}
