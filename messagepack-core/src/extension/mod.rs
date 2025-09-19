//! MessagePack extension helpers.

use crate::decode::Error as DecodeError;
use crate::{formats::Format, io::IoRead};

const U8_MAX: usize = u8::MAX as usize;
const U16_MAX: usize = u16::MAX as usize;
const U32_MAX: usize = u32::MAX as usize;
const U8_MAX_PLUS_ONE: usize = U8_MAX + 1;
const U16_MAX_PLUS_ONE: usize = U16_MAX + 1;

// Read extension header and return (length, type)
pub(crate) fn read_ext_header<'de, R>(
    format: Format,
    reader: &mut R,
) -> Result<(usize, i8), DecodeError<R::Error>>
where
    R: IoRead<'de>,
{
    use crate::decode::NbyteReader;
    let len = match format {
        Format::FixExt1 => 1,
        Format::FixExt2 => 2,
        Format::FixExt4 => 4,
        Format::FixExt8 => 8,
        Format::FixExt16 => 16,
        Format::Ext8 => NbyteReader::<1>::read(reader)?,
        Format::Ext16 => NbyteReader::<2>::read(reader)?,
        Format::Ext32 => NbyteReader::<4>::read(reader)?,
        _ => return Err(DecodeError::UnexpectedFormat),
    };

    let ext_type: [u8; 1] = reader
        .read_slice(1)
        .map_err(DecodeError::Io)?
        .as_bytes()
        .try_into()
        .map_err(|_| DecodeError::UnexpectedEof)?;
    let ty = ext_type[0] as i8;

    Ok((len, ty))
}

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
    pub fn to_format<E>(&self) -> core::result::Result<Format, crate::encode::Error<E>> {
        let format = match self.data.len() {
            1 => Format::FixExt1,
            2 => Format::FixExt2,
            4 => Format::FixExt4,
            8 => Format::FixExt8,
            16 => Format::FixExt16,
            0..=U8_MAX => Format::Ext8,
            U8_MAX_PLUS_ONE..=U16_MAX => Format::Ext16,
            U16_MAX_PLUS_ONE..=U32_MAX => Format::Ext32,
            _ => return Err(crate::encode::Error::InvalidFormat),
        };
        Ok(format)
    }
}

mod encode;
mod decode;

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

/// Error indicating that extension payload exceeds the fixed capacity `N`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExtensionCapacityError(());

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
    /// Construct with an exact `N`-byte payload
    pub fn new_fixed(r#type: i8, data: [u8; N]) -> Self {
        Self {
            r#type,
            len: N,
            data,
        }
    }

    /// Construct with a logical prefix
    pub fn new_fixed_with_prefix(
        r#type: i8,
        len: usize,
        data: [u8; N],
    ) -> core::result::Result<Self, ExtensionCapacityError> {
        if len <= N {
            Ok(Self { r#type, len, data })
        } else {
            Err(ExtensionCapacityError(()))
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

impl core::fmt::Display for ExtensionCapacityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "extension data exceeds capacity")
    }
}

impl core::error::Error for ExtensionCapacityError {}

impl<const N: usize> TryFrom<ExtensionRef<'_>> for FixedExtension<N> {
    type Error = ExtensionCapacityError;

    fn try_from(value: ExtensionRef<'_>) -> Result<Self, Self::Error> {
        if value.data.len() > N {
            return Err(ExtensionCapacityError(()));
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
}

#[cfg(feature = "alloc")]
pub use owned::ExtensionOwned;
