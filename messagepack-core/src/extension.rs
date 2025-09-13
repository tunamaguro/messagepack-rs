//! MessagePack extension helpers.

use crate::decode::{self, NbyteReader};
use crate::encode;
use crate::{
    Decode, Encode,
    formats::Format,
    io::{IoRead, IoWrite},
};

const U8_MAX: usize = u8::MAX as usize;
const U16_MAX: usize = u16::MAX as usize;
const U32_MAX: usize = u32::MAX as usize;
const U8_MAX_PLUS_ONE: usize = U8_MAX + 1;
const U16_MAX_PLUS_ONE: usize = U16_MAX + 1;

/// A borrowed view of a MessagePack extension value.
///
/// Note that the MessagePack header (FixExt vs Ext8/16/32) is determined by the
/// payload length when encoding. See [`ExtensionRef::to_format`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExtensionRef<'a> {
    /// Application‑defined extension type code.
    pub r#type: i8,
    /// Borrowed payload bytes.
    pub data: &'a [u8],
}

impl<'a> ExtensionRef<'a> {
    /// Create a borrowed reference to extension data with the given type code.
    pub fn new(r#type: i8, data: &'a [u8]) -> Self {
        Self { r#type, data }
    }

    /// Decide the MessagePack format to use given the payload length.
    ///
    /// - If `data.len()` is exactly 1, 2, 4, 8 or 16, `FixExtN` is selected.
    /// - Otherwise, `Ext8`/`Ext16`/`Ext32` is selected based on the byte length.
    pub fn to_format<E>(&self) -> core::result::Result<Format, encode::Error<E>> {
        let format = match self.data.len() {
            1 => Format::FixExt1,
            2 => Format::FixExt2,
            4 => Format::FixExt4,
            8 => Format::FixExt8,
            16 => Format::FixExt16,
            0..=U8_MAX => Format::Ext8,
            U8_MAX_PLUS_ONE..=U16_MAX => Format::Ext16,
            U16_MAX_PLUS_ONE..=U32_MAX => Format::Ext32,
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

impl<'de> Decode<'de> for ExtensionRef<'de> {
    type Value = ExtensionRef<'de>;

    fn decode_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, decode::Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        let len = match format {
            Format::FixExt1 => 1,
            Format::FixExt2 => 2,
            Format::FixExt4 => 4,
            Format::FixExt8 => 8,
            Format::FixExt16 => 16,
            Format::Ext8 => NbyteReader::<1>::read(reader)?,
            Format::Ext16 => NbyteReader::<2>::read(reader)?,
            Format::Ext32 => NbyteReader::<4>::read(reader)?,
            _ => return Err(decode::Error::UnexpectedFormat),
        };
        let ext_type: [u8; 1] = reader
            .read_slice(1)
            .map_err(decode::Error::Io)?
            .as_bytes()
            .try_into()
            .map_err(|_| decode::Error::UnexpectedEof)?;
        let ext_type = ext_type[0] as i8;

        let data_ref = reader.read_slice(len).map_err(decode::Error::Io)?;
        let data = match data_ref {
            crate::io::Reference::Borrowed(b) => b,
            crate::io::Reference::Copied(_) => return Err(decode::Error::InvalidData),
        };
        Ok(ExtensionRef {
            r#type: ext_type,
            data,
        })
    }
}

/// A fixed-capacity container for extension payloads of up to `N` bytes.
///
/// This type name refers to the fixed-size backing buffer, not the MessagePack
/// header kind. The actual header used at encode-time depends on the current
/// payload length:
/// - `len == 1, 2, 4, 8, 16` → `FixExtN`
/// - otherwise (0..=255, 256..=65535, 65536..=u32::MAX) → `Ext8/16/32`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FixedExtension<const N: usize> {
    /// Application‑defined extension type code.
    pub r#type: i8,
    len: usize,
    data: [u8; N],
}

impl<const N: usize> FixedExtension<N> {
    /// Construct from a slice whose length must be `<= N`.
    ///
    /// The chosen MessagePack format when encoding still follows the rules
    /// described in the type-level documentation above.
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

    /// Construct with an exact `N`-byte payload.
    ///
    /// Note: Even when constructed with a fixed-size buffer, the encoder will
    /// emit `FixExtN` only if `N` is one of {1, 2, 4, 8, 16}. For any other
    /// `N`, the encoder uses `Ext8/16/32` as appropriate.
    pub fn new_fixed(r#type: i8, len: usize, data: [u8; N]) -> Self {
        Self {
            r#type,
            len,
            data,
        }
    }

    /// Borrow as [`ExtensionRef`] for encoding.
    pub fn as_ref(&self) -> ExtensionRef<'_> {
        ExtensionRef {
            r#type: self.r#type,
            data: &self.data[..self.len],
        }
    }

    /// Current payload length in bytes.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the payload is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Extract a slice
    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }

    /// Extract a mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data[..self.len]
    }
}

/// The error type returned when a checked conversion from [`ExtensionRef`] fails
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TryFromExtensionRefError(());

impl core::fmt::Display for TryFromExtensionRefError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "extension data exceeds capacity")
    }
}

impl core::error::Error for TryFromExtensionRefError {}

impl<const N: usize> TryFrom<ExtensionRef<'_>> for FixedExtension<N> {
    type Error = TryFromExtensionRefError;

    fn try_from(value: ExtensionRef<'_>) -> Result<Self, Self::Error> {
        if value.data.len() > N {
            return Err(TryFromExtensionRefError(()));
        }
        let mut buf = [0u8; N];
        buf[..value.data.len()].copy_from_slice(value.data);
        Ok(Self {
            r#type: value.r#type,
            len: value.data.len(),
            data: buf,
        })
    }
}

impl<const N: usize, W: IoWrite> Encode<W> for FixedExtension<N> {
    fn encode(&self, writer: &mut W) -> core::result::Result<usize, encode::Error<W::Error>> {
        self.as_ref().encode(writer)
    }
}

impl<'de, const N: usize> Decode<'de> for FixedExtension<N> {
    type Value = FixedExtension<N>;

    fn decode_with_format<R>(
        format: Format,
        reader: &mut R,
    ) -> core::result::Result<Self::Value, decode::Error<R::Error>>
    where
        R: IoRead<'de>,
    {
        let ext = ExtensionRef::decode_with_format(format, reader)?;
        if ext.data.len() > N {
            return Err(decode::Error::InvalidData);
        }
        let mut buf_arr = [0u8; N];
        buf_arr[..ext.data.len()].copy_from_slice(ext.data);
        Ok(FixedExtension {
            r#type: ext.r#type,
            len: ext.data.len(),
            data: buf_arr,
        })
    }
}

#[cfg(feature = "alloc")]
mod owned {
    use super::*;

    /// An owned container for extension payloads.
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ExtensionOwned {
        /// Application‑defined extension type code.
        pub r#type: i8,
        /// payload bytes.
        pub data: alloc::vec::Vec<u8>,
    }

    impl ExtensionOwned {
        /// Create an owned extension value with the given type code and payload.
        pub fn new(r#type: i8, data: alloc::vec::Vec<u8>) -> Self {
            Self { r#type, data }
        }

        /// Borrow as [`ExtensionRef`] for encoding.
        pub fn as_ref(&self) -> ExtensionRef<'_> {
            ExtensionRef {
                r#type: self.r#type,
                data: &self.data,
            }
        }
    }

    impl<'a> From<ExtensionRef<'a>> for ExtensionOwned {
        fn from(value: ExtensionRef<'a>) -> Self {
            Self {
                r#type: value.r#type,
                data: value.data.to_vec(),
            }
        }
    }

    impl<const N: usize> From<FixedExtension<N>> for ExtensionOwned {
        fn from(value: FixedExtension<N>) -> Self {
            Self {
                r#type: value.r#type,
                data: value.as_slice().to_vec(),
            }
        }
    }

    impl<W: IoWrite> Encode<W> for ExtensionOwned {
        fn encode(&self, writer: &mut W) -> core::result::Result<usize, encode::Error<W::Error>> {
            self.as_ref().encode(writer)
        }
    }

    impl<'de> Decode<'de> for ExtensionOwned {
        type Value = ExtensionOwned;

        fn decode_with_format<R>(
            format: Format,
            reader: &mut R,
        ) -> core::result::Result<Self::Value, decode::Error<R::Error>>
        where
            R: crate::io::IoRead<'de>,
        {
            let ext = ExtensionRef::decode_with_format(format, reader)?;
            Ok(ExtensionOwned {
                r#type: ext.r#type,
                data: ext.data.to_vec(),
            })
        }
    }
}

#[cfg(feature = "alloc")]
pub use owned::ExtensionOwned;

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

    #[rstest]
    #[case(Format::FixExt1.as_byte(),  5_i8, [0x12])]
    #[case(Format::FixExt2.as_byte(), -1_i8, [0x34, 0x56])]
    #[case(Format::FixExt4.as_byte(), 42_i8, [0xde, 0xad, 0xbe, 0xef])]
    #[case(Format::FixExt8.as_byte(), -7_i8, [0xAA; 8])]
    #[case(Format::FixExt16.as_byte(), 7_i8, [0x55; 16])]
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

    #[rstest]
    #[case(Format::Ext8, 42_i8, 5u8.to_be_bytes(), [0x11;5])] // small: Ext8
    #[case(Format::Ext16, -7_i8,   300u16.to_be_bytes(), [0xAA;300])] // medium: Ext16 (>255)
    #[case(Format::Ext32, 7_i8, 70000u32.to_be_bytes(), [0x55;70000])] // large: Ext32 (>65535)
    fn decode_ext_sized<S: AsRef<[u8]>, D: AsRef<[u8]>>(
        #[case] format: Format,
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

    #[rstest]
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
}
