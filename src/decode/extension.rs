use super::{Decode, Error, NbyteReader, Result};
use crate::formats::Format;

pub struct Extension<'a> {
    pub r#type: i8,
    pub data: &'a [u8],
}

pub struct ExtensionDecoder;

impl<'a> Decode<'a> for ExtensionDecoder {
    type Value = Extension<'a>;
    fn decode(buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
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
            _ => Err(Error::UnexpectedFormat),
        }
    }
    fn decode_with_format(format: Format, buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (ext_type, buf) = buf.split_first().ok_or(Error::EofData)?;
        let (len, buf) = match format {
            Format::FixExt1 => (1, buf),
            Format::FixExt2 => (2, buf),
            Format::FixExt4 => (4, buf),
            Format::FixExt8 => (8, buf),
            Format::FixExt16 => (16, buf),
            Format::Ext8 => NbyteReader::<1>::read(buf)?,
            Format::Ext16 => NbyteReader::<2>::read(buf)?,
            Format::Ext32 => NbyteReader::<4>::read(buf)?,
            _ => return Err(Error::UnexpectedFormat),
        };
        let (data, rest) = buf.split_at_checked(len).ok_or(Error::EofData)?;
        let ext = Extension {
            r#type: (*ext_type) as i8,
            data,
        };
        Ok((ext, rest))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TIMESTAMP32: &[u8] = &[
        0xd6, // FixExt4
        0xff, // Timestamp type = -1
        // Feb 22 2022 22:22:22 GMT+0000
        // Unix sec = 1645568542
        0x62, 0x15, 0x62, 0x1e,
    ];

    #[test]
    fn decode_fix_ext4() {
        let (ext, rest) = ExtensionDecoder::decode(TIMESTAMP32).unwrap();

        let expect_type = -1;
        let expect_data = 1645568542;
        assert_eq!(ext.r#type, expect_type);
        let data_u32 = u32::from_be_bytes(ext.data.try_into().unwrap());
        assert_eq!(data_u32, expect_data);
        assert_eq!(rest.len(), 0);
    }
}
