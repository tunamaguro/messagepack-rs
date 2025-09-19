//! Timestamp decoding implementations and tests.

use super::{Timestamp32, Timestamp64, Timestamp96};
use crate::{
    Format,
    decode::{DecodeBorrowed, Error as DecodeError},
    extension::FixedExtension,
    io::IoRead,
};

impl<'de> DecodeBorrowed<'de> for Timestamp32 {
    type Value = Timestamp32;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, DecodeError<R::Error>>
    where
        R: IoRead<'de>,
    {
        match format {
            Format::FixExt4 => {}
            _ => return Err(DecodeError::UnexpectedFormat),
        }
        let ext = FixedExtension::<4>::decode_borrowed_with_format(format, reader)?;
        let timestamp = ext.try_into().map_err(|_| DecodeError::InvalidData)?;
        Ok(timestamp)
    }
}

impl<'de> DecodeBorrowed<'de> for Timestamp64 {
    type Value = Timestamp64;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, DecodeError<R::Error>>
    where
        R: IoRead<'de>,
    {
        match format {
            Format::FixExt8 => {}
            _ => return Err(DecodeError::UnexpectedFormat),
        }
        let ext = FixedExtension::<8>::decode_borrowed_with_format(format, reader)?;
        let timestamp = ext.try_into().map_err(|_| DecodeError::InvalidData)?;
        Ok(timestamp)
    }
}

impl<'de> DecodeBorrowed<'de> for Timestamp96 {
    type Value = Timestamp96;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, DecodeError<R::Error>>
    where
        R: IoRead<'de>,
    {
        match format {
            Format::Ext8 => {}
            _ => return Err(DecodeError::UnexpectedFormat),
        }
        const TIMESTAMP96_DATA_LENGTH: usize = 12;
        let ext =
            FixedExtension::<TIMESTAMP96_DATA_LENGTH>::decode_borrowed_with_format(format, reader)?;
        let timestamp = ext.try_into().map_err(|_| DecodeError::InvalidData)?;
        Ok(timestamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::Decode;

    const TIMESTAMP_EXT_TYPE_U8: u8 = 255; // -1

    #[test]
    fn decode_success_timestamp32() {
        let secs: u32 = 1234567890;
        let mut buf = vec![0xd6, TIMESTAMP_EXT_TYPE_U8];
        buf.extend_from_slice(&secs.to_be_bytes());

        let mut r = crate::io::SliceReader::new(&buf);
        let ts = Timestamp32::decode(&mut r).unwrap();
        assert_eq!(ts.seconds(), secs);
        assert!(r.rest().is_empty());
    }

    #[test]
    fn decode_failed_timestamp32_invalid_ext_type() {
        let secs: u32 = 1;
        let mut buf = vec![0xd6, 0]; // ext type != -1
        buf.extend_from_slice(&secs.to_be_bytes());

        let mut r = crate::io::SliceReader::new(&buf);
        let err = Timestamp32::decode(&mut r).unwrap_err();
        assert_eq!(err, DecodeError::InvalidData);
    }

    #[test]
    fn decode_failed_timestamp32_eof_data() {
        let secs: u32 = 123;
        let mut buf = vec![0xd6, TIMESTAMP_EXT_TYPE_U8];
        buf.extend_from_slice(&secs.to_be_bytes()[..3]); // 1 byte short

        let mut r = crate::io::SliceReader::new(&buf);
        let err = Timestamp32::decode(&mut r).unwrap_err();
        assert!(matches!(err, DecodeError::Io(_)));
    }

    #[test]
    fn decode_success_timestamp64() {
        let secs: u64 = 1234567890;
        let nanos: u32 = 789;

        let data = ((nanos as u64) << 34) | secs;
        let mut buf = vec![0xd7, TIMESTAMP_EXT_TYPE_U8];
        buf.extend_from_slice(&data.to_be_bytes());

        let mut r = crate::io::SliceReader::new(&buf);
        let ts = Timestamp64::decode(&mut r).unwrap();
        assert_eq!(ts.seconds(), secs);
        assert_eq!(ts.nanos(), nanos);
        assert!(r.rest().is_empty());
    }

    #[test]
    fn decode_failed_timestamp64_unexpected_format() {
        let mut buf = vec![0xd6, TIMESTAMP_EXT_TYPE_U8]; // FixExt4, not FixExt8
        buf.extend_from_slice(&0u64.to_be_bytes());

        let mut r = crate::io::SliceReader::new(&buf);
        let err = Timestamp64::decode(&mut r).unwrap_err();
        assert_eq!(err, DecodeError::UnexpectedFormat);
    }

    #[test]
    fn decode_failed_timestamp64_invalid_ext_type() {
        let mut buf = vec![0xd7, 0]; // ext type != -1
        buf.extend_from_slice(&0u64.to_be_bytes());

        let mut r = crate::io::SliceReader::new(&buf);
        let err = Timestamp64::decode(&mut r).unwrap_err();
        assert_eq!(err, DecodeError::InvalidData);
    }

    #[test]
    fn decode_failed_timestamp64_eof_data() {
        let mut buf = vec![0xd7, TIMESTAMP_EXT_TYPE_U8];
        buf.extend_from_slice(&[0u8; 7]); // 1 byte short

        let mut r = crate::io::SliceReader::new(&buf);
        let err = Timestamp64::decode(&mut r).unwrap_err();
        assert!(matches!(err, DecodeError::Io(_)));
    }

    #[test]
    fn decode_failed_timestamp64_invalid_nanos() {
        // Construct data with nanos = 1_000_000_000 (out of spec)
        let secs: u64 = 0;
        let nanos: u64 = 1_000_000_000;
        let data = (nanos << 34) | secs;
        let mut buf = vec![0xd7, TIMESTAMP_EXT_TYPE_U8];
        buf.extend_from_slice(&data.to_be_bytes());

        let mut r = crate::io::SliceReader::new(&buf);
        let err = Timestamp64::decode(&mut r).unwrap_err();
        assert_eq!(err, DecodeError::InvalidData);
    }

    #[test]
    fn decode_success_timestamp96_positive() {
        let secs: i64 = 123456;
        let nanos: u32 = 789;

        let mut buf = vec![0xc7, 12, TIMESTAMP_EXT_TYPE_U8];
        buf.extend_from_slice(&nanos.to_be_bytes());
        buf.extend_from_slice(&secs.to_be_bytes());

        let mut r = crate::io::SliceReader::new(&buf);
        let ts = Timestamp96::decode(&mut r).unwrap();
        assert_eq!(ts.seconds(), secs);
        assert_eq!(ts.nanos(), nanos);
        assert!(r.rest().is_empty());
    }

    #[test]
    fn decode_success_timestamp96_negative() {
        let secs: i64 = -123;
        let nanos: u32 = 42;

        let mut buf = vec![0xc7, 12, TIMESTAMP_EXT_TYPE_U8];
        buf.extend_from_slice(&nanos.to_be_bytes());
        buf.extend_from_slice(&secs.to_be_bytes());

        let mut r = crate::io::SliceReader::new(&buf);
        let ts = Timestamp96::decode(&mut r).unwrap();
        assert_eq!(ts.seconds(), secs);
        assert_eq!(ts.nanos(), nanos);
        assert!(r.rest().is_empty());
    }

    #[test]
    fn decode_failed_timestamp96_unexpected_format() {
        // FixExt8 header instead of Ext8
        let mut buf = vec![0xd7, TIMESTAMP_EXT_TYPE_U8];
        buf.extend_from_slice(&[0u8; 8]);

        let mut r = crate::io::SliceReader::new(&buf);
        let err = Timestamp96::decode(&mut r).unwrap_err();
        assert_eq!(err, DecodeError::UnexpectedFormat);
    }

    #[test]
    fn decode_failed_timestamp96_invalid_length() {
        // Ext8 length != 12
        let mut buf = vec![0xc7, 11, TIMESTAMP_EXT_TYPE_U8];
        buf.extend_from_slice(&[0u8; 11]);

        let mut r = crate::io::SliceReader::new(&buf);
        let err = Timestamp96::decode(&mut r).unwrap_err();
        assert_eq!(err, DecodeError::InvalidData);
    }

    #[test]
    fn decode_failed_timestamp96_invalid_ext_type() {
        let secs: i64 = 1;
        let nanos: u32 = 2;

        let mut buf = vec![0xc7, 12, 0]; // ext type != -1
        buf.extend_from_slice(&nanos.to_be_bytes());
        buf.extend_from_slice(&secs.to_be_bytes());

        let mut r = crate::io::SliceReader::new(&buf);
        let err = Timestamp96::decode(&mut r).unwrap_err();
        assert_eq!(err, DecodeError::InvalidData);
    }

    #[test]
    fn decode_failed_timestamp96_eof_data() {
        // length says 12 but provide 11
        let mut buf = vec![0xc7, 12, TIMESTAMP_EXT_TYPE_U8];
        buf.extend_from_slice(&[0u8; 11]);

        let mut r = crate::io::SliceReader::new(&buf);
        let err = Timestamp96::decode(&mut r).unwrap_err();
        assert!(matches!(err, DecodeError::Io(_)));
    }

    #[test]
    fn decode_failed_timestamp96_invalid_nanos() {
        // nanos = 1_000_000_000 should be rejected
        let nanos: u32 = 1_000_000_000;
        let secs: i64 = 0;
        let mut buf = vec![0xc7, 12, TIMESTAMP_EXT_TYPE_U8];
        buf.extend_from_slice(&nanos.to_be_bytes());
        buf.extend_from_slice(&secs.to_be_bytes());

        let mut r = crate::io::SliceReader::new(&buf);
        let err = Timestamp96::decode(&mut r).unwrap_err();
        assert_eq!(err, DecodeError::InvalidData);
    }
}
