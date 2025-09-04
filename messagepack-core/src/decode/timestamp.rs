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

#[cfg(test)]
mod tests {
    use super::*;
    const TIMESTAMP_EXT_TYPE: u8 = 255; // -1

    #[test]
    fn decode_success_timestamp32() {
        let secs: u32 = 1234567890;
        let mut buf = vec![0xd6, TIMESTAMP_EXT_TYPE];
        buf.extend_from_slice(&secs.to_be_bytes());

        let (ts, rest) = Timestamp32::decode(&buf).unwrap();
        assert_eq!(ts.seconds(), secs);
        assert!(rest.is_empty());
    }

    #[test]
    fn decode_failed_timestamp32_invalid_ext_type() {
        let secs: u32 = 1;
        let mut buf = vec![0xd6, 0]; // ext type != -1
        buf.extend_from_slice(&secs.to_be_bytes());

        let err = Timestamp32::decode(&buf).unwrap_err();
        assert_eq!(err, Error::InvalidData);
    }

    #[test]
    fn decode_failed_timestamp32_eof_data() {
        let secs: u32 = 123;
        let mut buf = vec![0xd6, TIMESTAMP_EXT_TYPE];
        buf.extend_from_slice(&secs.to_be_bytes()[..3]); // 1 byte short

        let err = Timestamp32::decode(&buf).unwrap_err();
        assert_eq!(err, Error::EofData);
    }

    #[test]
    fn decode_success_timestamp64() {
        let secs: u64 = 1234567890;
        let nanos: u32 = 789;

        let data = ((nanos as u64) << 34) | secs;
        let mut buf = vec![0xd7, TIMESTAMP_EXT_TYPE];
        buf.extend_from_slice(&data.to_be_bytes());

        let (ts, rest) = Timestamp64::decode(&buf).unwrap();
        assert_eq!(ts.seconds(), secs);
        assert_eq!(ts.nanos(), nanos);
        assert!(rest.is_empty());
    }

    #[test]
    fn decode_failed_timestamp64_unexpected_format() {
        let mut buf = vec![0xd6, TIMESTAMP_EXT_TYPE]; // FixExt4, not FixExt8
        buf.extend_from_slice(&0u64.to_be_bytes());

        let err = Timestamp64::decode(&buf).unwrap_err();
        assert_eq!(err, Error::UnexpectedFormat);
    }

    #[test]
    fn decode_failed_timestamp64_invalid_ext_type() {
        let mut buf = vec![0xd7, 0]; // ext type != -1
        buf.extend_from_slice(&0u64.to_be_bytes());

        let err = Timestamp64::decode(&buf).unwrap_err();
        assert_eq!(err, Error::InvalidData);
    }

    #[test]
    fn decode_failed_timestamp64_eof_data() {
        let mut buf = vec![0xd7, TIMESTAMP_EXT_TYPE];
        buf.extend_from_slice(&[0u8; 7]); // 1 byte short

        let err = Timestamp64::decode(&buf).unwrap_err();
        assert_eq!(err, Error::EofData);
    }

    #[test]
    fn decode_success_timestamp96_positive() {
        let secs: i64 = 123456;
        let nanos: u32 = 789;

        let mut buf = vec![0xc7, 12, TIMESTAMP_EXT_TYPE];
        buf.extend_from_slice(&nanos.to_be_bytes());
        buf.extend_from_slice(&secs.to_be_bytes());

        let (ts, rest) = Timestamp96::decode(&buf).unwrap();
        assert_eq!(ts.seconds(), secs);
        assert_eq!(ts.nanos(), nanos);
        assert!(rest.is_empty());
    }

    #[test]
    fn decode_success_timestamp96_negative() {
        let secs: i64 = -123;
        let nanos: u32 = 42;

        let mut buf = vec![0xc7, 12, TIMESTAMP_EXT_TYPE];
        buf.extend_from_slice(&nanos.to_be_bytes());
        buf.extend_from_slice(&secs.to_be_bytes());

        let (ts, rest) = Timestamp96::decode(&buf).unwrap();
        assert_eq!(ts.seconds(), secs);
        assert_eq!(ts.nanos(), nanos);
        assert!(rest.is_empty());
    }

    #[test]
    fn decode_failed_timestamp96_unexpected_format() {
        // FixExt8 header instead of Ext8
        let mut buf = vec![0xd7, TIMESTAMP_EXT_TYPE];
        buf.extend_from_slice(&[0u8; 8]);

        let err = Timestamp96::decode(&buf).unwrap_err();
        assert_eq!(err, Error::UnexpectedFormat);
    }

    #[test]
    fn decode_failed_timestamp96_invalid_length() {
        // Ext8 length != 12
        let mut buf = vec![0xc7, 11, TIMESTAMP_EXT_TYPE];
        buf.extend_from_slice(&[0u8; 11]);

        let err = Timestamp96::decode(&buf).unwrap_err();
        assert_eq!(err, Error::InvalidData);
    }

    #[test]
    fn decode_failed_timestamp96_invalid_ext_type() {
        let secs: i64 = 1;
        let nanos: u32 = 2;

        let mut buf = vec![0xc7, 12, 0]; // ext type != -1
        buf.extend_from_slice(&nanos.to_be_bytes());
        buf.extend_from_slice(&secs.to_be_bytes());

        let err = Timestamp96::decode(&buf).unwrap_err();
        assert_eq!(err, Error::InvalidData);
    }

    #[test]
    fn decode_failed_timestamp96_eof_data() {
        // length says 12 but provide 11
        let mut buf = vec![0xc7, 12, TIMESTAMP_EXT_TYPE];
        buf.extend_from_slice(&[0u8; 11]);

        let err = Timestamp96::decode(&buf).unwrap_err();
        assert_eq!(err, Error::EofData);
    }
}
