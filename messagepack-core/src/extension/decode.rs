// Extension decoding implementations and tests.

use super::{read_ext_header, ExtensionRef, FixedExtension};
use crate::decode::{DecodeBorrowed, Error as DecodeError};
use crate::io::IoRead;
use crate::Format;

impl<'de> DecodeBorrowed<'de> for ExtensionRef<'de> {
    type Value = ExtensionRef<'de>;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, DecodeError<R::Error>>
    where
        R: IoRead<'de>,
    {
        let (len, ext_type) = read_ext_header(format, reader)?;

        let data_ref = reader.read_slice(len).map_err(DecodeError::Io)?;
        let data = match data_ref {
            crate::io::Reference::Borrowed(b) => b,
            crate::io::Reference::Copied(_) => return Err(DecodeError::InvalidData),
        };
        Ok(ExtensionRef {
            r#type: ext_type,
            data,
        })
    }
}

impl<'de, const N: usize> DecodeBorrowed<'de> for FixedExtension<N> {
    type Value = FixedExtension<N>;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, DecodeError<R::Error>>
    where
        R: IoRead<'de>,
    {
        let (len, ext_type) = read_ext_header(format, reader)?;

        if len > N {
            return Err(DecodeError::InvalidData);
        }

        let payload = reader.read_slice(len).map_err(DecodeError::Io)?;
        let bytes = payload.as_bytes();
        if bytes.len() != len {
            return Err(DecodeError::UnexpectedEof);
        }

        let mut data = [0u8; N];
        data[..len].copy_from_slice(bytes);

        Ok(FixedExtension {
            r#type: ext_type,
            len,
            data,
        })
    }
}

#[cfg(feature = "alloc")]
impl<'de> DecodeBorrowed<'de> for super::owned::ExtensionOwned {
    type Value = super::owned::ExtensionOwned;

    fn decode_borrowed_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, DecodeError<R::Error>>
    where
        R: crate::io::IoRead<'de>,
    {
        let (len, ext_type) = read_ext_header(format, reader)?;

        let payload = reader.read_slice(len).map_err(DecodeError::Io)?;
        let data = payload.as_bytes().to_vec();

        Ok(super::owned::ExtensionOwned { r#type: ext_type, data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::Decode;
    use crate::encode::Encode;

    #[rstest::rstest]
    #[case(crate::Format::FixExt1.as_byte(),  5_i8, [0x12])]
    #[case(crate::Format::FixExt2.as_byte(), -1_i8, [0x34, 0x56])]
    #[case(crate::Format::FixExt4.as_byte(), 42_i8, [0xde, 0xad, 0xbe, 0xef])]
    #[case(crate::Format::FixExt8.as_byte(), -7_i8, [0xAA; 8])]
    #[case(crate::Format::FixExt16.as_byte(), 7_i8, [0x55; 16])]
    fn decode_ext_fixed<E: AsRef<[u8]>>(#[case] marker: u8, #[case] ty: i8, #[case] data: E) {
        // Buffer: [FixExtN marker][type][data..]
        let buf = core::iter::once(marker)
            .chain(core::iter::once(ty as u8))
            .chain(data.as_ref().iter().cloned())
            .collect::<Vec<u8>>();

        let mut r = crate::io::SliceReader::new(&buf);
        let ext = ExtensionRef::decode(&mut r).unwrap();
        assert_eq!(ext.r#type, ty);
        assert_eq!(ext.data, data.as_ref());
        assert!(r.rest().is_empty());
    }

    #[rstest::rstest]
    #[case(crate::Format::Ext8, 42_i8, 5u8.to_be_bytes(), [0x11;5])] // small: Ext8
    #[case(crate::Format::Ext16, -7_i8,   300u16.to_be_bytes(), [0xAA;300])] // medium: Ext16 (>255)
    #[case(crate::Format::Ext32, 7_i8, 70000u32.to_be_bytes(), [0x55;70000])] // large: Ext32 (>65535)
    fn decode_ext_sized<S: AsRef<[u8]>, D: AsRef<[u8]>>(
        #[case] format: crate::Format,
        #[case] ty: i8,
        #[case] size: S,
        #[case] data: D,
    ) {
        // MessagePack ext variable-length layout: [format][length][type][data]
        let buf = format
            .as_slice()
            .iter()
            .chain(size.as_ref())
            .chain(ty.to_be_bytes().iter())
            .chain(data.as_ref())
            .cloned()
            .collect::<Vec<_>>();

        let mut r = crate::io::SliceReader::new(&buf);
        let ext = ExtensionRef::decode(&mut r).unwrap();
        assert_eq!(ext.r#type, ty);
        assert_eq!(ext.data, data.as_ref());
        assert!(r.rest().is_empty());
    }

    #[rstest::rstest]
    fn fixed_extension_roundtrip() {
        let data = [1u8, 2, 3, 4];
        let ext = FixedExtension::<8>::new(5, &data).unwrap();
        let mut buf = vec![];
        ext.encode(&mut buf).unwrap();
        let mut r = crate::io::SliceReader::new(&buf);
        let decoded = FixedExtension::<8>::decode(&mut r).unwrap();
        assert_eq!(decoded.r#type, 5);
        assert_eq!(decoded.as_slice(), &data);
        assert!(r.rest().is_empty());
    }

    #[cfg(feature = "std")]
    #[rstest::rstest]
    fn fixed_extension_decode_with_std_reader_copied() {
        // Build Ext8 with len=5, type=7, payload=[9;5]
        let mut buf = vec![];
        buf.extend_from_slice(&[crate::Format::Ext8.as_byte(), 5u8, 7u8]);
        buf.extend_from_slice(&[9u8; 5]);

        // Use StdReader which yields Copied references internally
        let cursor = std::io::Cursor::new(buf);
        let mut r = crate::io::StdReader::new(cursor);

        let decoded = FixedExtension::<8>::decode(&mut r).unwrap();
        assert_eq!(decoded.r#type, 7);
        assert_eq!(decoded.as_slice(), &[9u8; 5]);
    }

    #[cfg(feature = "std")]
    #[rstest::rstest]
    fn extension_owned_decode_with_std_reader_copied() {
        // Build Ext8 with len=3, type=-5, payload=[0xAA, 0xBB, 0xCC]
        let mut buf = vec![];
        buf.extend_from_slice(&[crate::Format::Ext8.as_byte(), 3u8, (!5u8) + 1]); // 251 -> -5
        buf.extend_from_slice(&[0xAA, 0xBB, 0xCC]);

        let cursor = std::io::Cursor::new(buf);
        let mut r = crate::io::StdReader::new(cursor);

        let ext = super::super::ExtensionOwned::decode(&mut r).unwrap();
        assert_eq!(ext.r#type, -5);
        assert_eq!(ext.data, vec![0xAA, 0xBB, 0xCC]);
    }
}
