use super::{Decode, Error, NbyteReader, Result};
use crate::{
    Format,
    timestamp::{TIMESTAMP_EXTENSION_TYPE, Timestamp32, Timestamp64, Timestamp96},
};

impl<'a> Decode<'a> for Timestamp32 {
    type Value = Timestamp32;
    fn decode(buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (format, buf) = Format::decode(buf)?;
        match format {
            Format::FixExt4 => Self::decode_with_format(format, buf),
            _ => Err(Error::UnexpectedFormat),
        }
    }
    fn decode_with_format(format: crate::Format, buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (len, buf) = match format {
            Format::FixExt4 => (4, buf),
            _ => return Err(Error::UnexpectedFormat),
        };
        let (ext_type, buf) = buf.split_first().ok_or(Error::EofData)?;
        let ext_type = (*ext_type) as i8;
        if ext_type != TIMESTAMP_EXTENSION_TYPE {
            return Err(Error::InvalidData);
        }

        let (data, rest) = buf.split_at_checked(len).ok_or(Error::EofData)?;
        let timestamp = Self::from_buf(data.try_into().expect("expect 4 len"));
        Ok((timestamp, rest))
    }
}

impl<'a> Decode<'a> for Timestamp64 {
    type Value = Timestamp64;
    fn decode(buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (format, buf) = Format::decode(buf)?;
        match format {
            Format::FixExt8 => Self::decode_with_format(format, buf),
            _ => Err(Error::UnexpectedFormat),
        }
    }
    fn decode_with_format(format: crate::Format, buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (len, buf) = match format {
            Format::FixExt8 => (8, buf),
            _ => return Err(Error::UnexpectedFormat),
        };
        let (ext_type, buf) = buf.split_first().ok_or(Error::EofData)?;
        let ext_type = (*ext_type) as i8;
        if ext_type != TIMESTAMP_EXTENSION_TYPE {
            return Err(Error::InvalidData);
        }

        let (data, rest) = buf.split_at_checked(len).ok_or(Error::EofData)?;
        let timestamp = Self::from_buf(data.try_into().expect("expect 8 len"));
        Ok((timestamp, rest))
    }
}

impl<'a> Decode<'a> for Timestamp96 {
    type Value = Timestamp96;
    fn decode(buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (format, buf) = Format::decode(buf)?;
        match format {
            Format::Ext8 => Self::decode_with_format(format, buf),
            _ => Err(Error::UnexpectedFormat),
        }
    }
    fn decode_with_format(format: crate::Format, buf: &'a [u8]) -> Result<(Self::Value, &'a [u8])> {
        let (len, buf) = match format {
            Format::Ext8 => NbyteReader::<1>::read(buf)?,
            _ => return Err(Error::UnexpectedFormat),
        };
        const TIMESTAMP64_DATA_LENGTH: usize = 12;
        if len != TIMESTAMP64_DATA_LENGTH {
            return Err(Error::InvalidData);
        }

        let (ext_type, buf) = buf.split_first().ok_or(Error::EofData)?;
        let ext_type = (*ext_type) as i8;
        if ext_type != TIMESTAMP_EXTENSION_TYPE {
            return Err(Error::InvalidData);
        }

        let (data, rest) = buf.split_at_checked(len).ok_or(Error::EofData)?;
        let timestamp = Self::from_buf(data.try_into().expect("expect 12 len"));
        Ok((timestamp, rest))
    }
}
