use super::{DecodeBorrowed, Error};
use crate::{formats::Format, io::IoRead};

impl<'de> DecodeBorrowed<'de> for u8 {
    type Value = Self;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        match format {
            Format::PositiveFixInt(v) => Ok(v),
            Format::Uint8 => {
                let b = reader.read_slice(1).map_err(Error::Io)?;
                let v: [u8; 1] = b.as_bytes().try_into().map_err(|_| Error::UnexpectedEof)?;
                Ok(v[0])
            }
            _ => Err(Error::UnexpectedFormat),
        }
    }
}

impl<'de> DecodeBorrowed<'de> for i8 {
    type Value = Self;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        match format {
            Format::Int8 => {
                let b = reader.read_slice(1).map_err(Error::Io)?;
                let v: [u8; 1] = b.as_bytes().try_into().map_err(|_| Error::UnexpectedEof)?;
                Ok(v[0] as i8)
            }
            Format::NegativeFixInt(v) => Ok(v),
            _ => Err(Error::UnexpectedFormat),
        }
    }
}

macro_rules! impl_decode_int {
    ($ty:ty,$format:path) => {
        impl<'de> DecodeBorrowed<'de> for $ty {
            type Value = Self;

            fn decode_borrowed_with_format<R>(
                format: Format,
                reader: &mut R,
            ) -> core::result::Result<Self::Value, Error<R::Error>>
            where
                R: IoRead<'de>,
            {
                match format {
                    $format => {
                        const SIZE: usize = core::mem::size_of::<$ty>();
                        let bytes = reader.read_slice(SIZE).map_err(Error::Io)?;
                        let slice = bytes.as_bytes();
                        let data: [u8; SIZE] =
                            slice.try_into().map_err(|_| Error::UnexpectedEof)?;
                        let val = <$ty>::from_be_bytes(data);
                        Ok(val)
                    }
                    _ => Err(Error::UnexpectedFormat),
                }
            }
        }
    };
}

impl_decode_int!(u16, Format::Uint16);
impl_decode_int!(u32, Format::Uint32);
impl_decode_int!(u64, Format::Uint64);
impl_decode_int!(i16, Format::Int16);
impl_decode_int!(i32, Format::Int32);
impl_decode_int!(i64, Format::Int64);

impl<'de> DecodeBorrowed<'de> for u128 {
    type Value = Self;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        let val = u64::decode_borrowed_with_format(format, reader)?;
        Ok(Self::from(val))
    }
}

impl<'de> DecodeBorrowed<'de> for i128 {
    type Value = Self;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        let val = i64::decode_borrowed_with_format(format, reader)?;
        Ok(Self::from(val))
    }
}

impl<'de> DecodeBorrowed<'de> for usize {
    type Value = Self;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        let val = u64::decode_borrowed_with_format(format, reader)?;
        usize::try_from(val).map_err(|_| Error::InvalidData)
    }
}

impl<'de> DecodeBorrowed<'de> for isize {
    type Value = Self;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        let val = i64::decode_borrowed_with_format(format, reader)?;
        isize::try_from(val).map_err(|_| Error::InvalidData)
    }
}

macro_rules! impl_nonzero_int {
    ($ty:ty) => {
        impl<'de> DecodeBorrowed<'de> for core::num::NonZero<$ty> {
            type Value = Self;

            fn decode_borrowed_with_format<R>(
                format: Format,
                reader: &mut R,
            ) -> core::result::Result<Self::Value, Error<R::Error>>
            where
                R: IoRead<'de>,
            {
                let val = <$ty>::decode_borrowed_with_format(format, reader)?;
                Self::new(val).ok_or(Error::InvalidData)
            }
        }
    };
}
impl_nonzero_int!(u8);
impl_nonzero_int!(u16);
impl_nonzero_int!(u32);
impl_nonzero_int!(u64);
impl_nonzero_int!(usize);
impl_nonzero_int!(i8);
impl_nonzero_int!(i16);
impl_nonzero_int!(i32);
impl_nonzero_int!(i64);
impl_nonzero_int!(isize);

macro_rules! impl_atomic_int {
    ($ty:ty, $base:ty) => {
        #[cfg(target_has_atomic = $bits)]
        impl<'de> DecodeBorrowed<'de> for $ty {
            type Value = Self;

            fn decode_borrowed_with_format<R>(
                format: Format,
                reader: &mut R,
            ) -> Result<Self::Value, Error<R::Error>>
            where
                R: IoRead<'de>,
            {
                let val = <$base>::decode_borrowed_with_format(format, reader)?;
                Ok(Self::new(val))
            }
        }
    };
}
impl_atomic_int!(core::sync::atomic::AtomicU8, u8, "8");
impl_atomic_int!(core::sync::atomic::AtomicU16, u16, "16");
impl_atomic_int!(core::sync::atomic::AtomicU32, u32, "32");
impl_atomic_int!(core::sync::atomic::AtomicU64, u64, "64");
impl_atomic_int!(core::sync::atomic::AtomicUsize, usize, "ptr");
impl_atomic_int!(core::sync::atomic::AtomicI8, i8, "8");
impl_atomic_int!(core::sync::atomic::AtomicI16, i16, "16");
impl_atomic_int!(core::sync::atomic::AtomicI32, i32, "32");
impl_atomic_int!(core::sync::atomic::AtomicI64, i64, "64");
impl_atomic_int!(core::sync::atomic::AtomicIsize, isize, "ptr");

#[cfg(test)]
mod tests {
    use crate::decode::Decode;

    #[test]
    fn decode_u8_fix_pos_int() {
        // FixPosInt
        let buf: &[u8] = &[0x00];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = u8::decode(&mut r).unwrap();
        let expect = u8::MIN;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);

        let buf: &[u8] = &[0x7f];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = u8::decode(&mut r).unwrap();
        let expect = 0x7f;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);
    }

    #[test]
    fn decode_u8() {
        // Uint8
        let buf: &[u8] = &[0xcc, 0x00];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = u8::decode(&mut r).unwrap();
        let expect = u8::MIN;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);

        let buf: &[u8] = &[0xcc, 0xff];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = u8::decode(&mut r).unwrap();
        let expect = u8::MAX;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);
    }

    #[test]
    fn decode_u16() {
        // Uint16
        let buf: &[u8] = &[0xcd, 0x00, 0x00];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = u16::decode(&mut r).unwrap();
        let expect = u16::MIN;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);

        let buf: &[u8] = &[0xcd, 0x12, 0x34];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = u16::decode(&mut r).unwrap();
        let expect = 0x1234;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);

        let buf: &[u8] = &[0xcd, 0xff, 0xff];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = u16::decode(&mut r).unwrap();
        let expect = u16::MAX;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);
    }

    #[test]
    fn decode_u32() {
        // Uint32
        let buf: &[u8] = &[0xce, 0x00, 0x00, 0x00, 0x00];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = u32::decode(&mut r).unwrap();
        let expect = u32::MIN;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);

        let buf: &[u8] = &[0xce, 0x12, 0x34, 0x56, 0x78];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = u32::decode(&mut r).unwrap();
        let expect = 0x12345678;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);

        let buf: &[u8] = &[0xce, 0xff, 0xff, 0xff, 0xff];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = u32::decode(&mut r).unwrap();
        let expect = u32::MAX;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);
    }

    #[test]
    fn decode_u64() {
        // Uint64
        let buf: &[u8] = &[0xcf, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = u64::decode(&mut r).unwrap();
        let expect = u64::MIN;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);

        let buf: &[u8] = &[0xcf, 0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56, 0x78];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = u64::decode(&mut r).unwrap();
        let expect = 0x1234567812345678;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);

        let buf: &[u8] = &[0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = u64::decode(&mut r).unwrap();
        let expect = u64::MAX;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);
    }

    #[test]
    fn decode_i8_fix_neg_int() {
        // FixNegInt
        let buf: &[u8] = &[0xff];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = i8::decode(&mut r).unwrap();
        let expect = -1;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);

        let buf: &[u8] = &[0xe0];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = i8::decode(&mut r).unwrap();
        let expect = -32;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);
    }

    #[test]
    fn decode_i8() {
        // Int8
        let buf: &[u8] = &[0xd0, 0x80];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = i8::decode(&mut r).unwrap();
        let expect = i8::MIN;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);

        let buf: &[u8] = &[0xd0, 0x7f];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = i8::decode(&mut r).unwrap();
        let expect = i8::MAX;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);
    }

    #[test]
    fn decode_i16() {
        // Int16
        let buf: &[u8] = &[0xd1, 0x80, 0x00];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = i16::decode(&mut r).unwrap();
        let expect = i16::MIN;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);

        let buf: &[u8] = &[0xd1, 0x7f, 0xff];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = i16::decode(&mut r).unwrap();
        let expect = i16::MAX;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);
    }

    #[test]
    fn decode_i32() {
        // Int16
        let buf: &[u8] = &[0xd2, 0x80, 0x00, 0x00, 0x00];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = i32::decode(&mut r).unwrap();
        let expect = i32::MIN;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);

        let buf: &[u8] = &[0xd2, 0x7f, 0xff, 0xff, 0xff];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = i32::decode(&mut r).unwrap();
        let expect = i32::MAX;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);
    }

    #[test]
    fn decode_i64() {
        // Int16
        let buf: &[u8] = &[0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = i64::decode(&mut r).unwrap();
        let expect = i64::MIN;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);

        let buf: &[u8] = &[0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        let mut r = crate::io::SliceReader::new(buf);
        let decoded = i64::decode(&mut r).unwrap();
        let expect = i64::MAX;
        assert_eq!(decoded, expect);
        assert_eq!(r.rest().len(), 0);
    }
}
