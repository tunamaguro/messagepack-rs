//! Extension family helper

pub(crate) mod de;
pub(crate) mod ser;

use serde::{Serialize, Serializer, de::Visitor};
pub(crate) const EXTENSION_STRUCT_NAME: &str = "$__MSGPACK_EXTENSION_STRUCT";

#[cfg(feature = "alloc")]
mod owned;
#[cfg(feature = "alloc")]
pub use owned::ext_owned;

mod timestamp;
pub use timestamp::{timestamp32, timestamp64, timestamp96};

struct Bytes<'a>(pub &'a [u8]);
impl Serialize for Bytes<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.0)
    }
}

struct ExtInner<'a> {
    kind: i8,
    data: &'a [u8],
}

impl Serialize for ExtInner<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use messagepack_core::extension::ExtensionRef;
        use serde::ser::{self, SerializeSeq};
        let encoder = ExtensionRef::new(self.kind, self.data);
        let format = encoder
            .to_format::<core::convert::Infallible>()
            .map_err(|_| ser::Error::custom("Invalid data length"))?;

        let mut seq = serializer.serialize_seq(None)?;

        seq.serialize_element(&Bytes(&format.as_slice()))?;

        match format {
            messagepack_core::Format::FixExt1
            | messagepack_core::Format::FixExt2
            | messagepack_core::Format::FixExt4
            | messagepack_core::Format::FixExt8
            | messagepack_core::Format::FixExt16 => {}

            messagepack_core::Format::Ext8 => {
                let len = (self.data.len() as u8).to_be_bytes();
                seq.serialize_element(&Bytes(&len))?;
            }
            messagepack_core::Format::Ext16 => {
                let len = (self.data.len() as u16).to_be_bytes();
                seq.serialize_element(&Bytes(&len))?;
            }
            messagepack_core::Format::Ext32 => {
                let len = (self.data.len() as u32).to_be_bytes();
                seq.serialize_element(&Bytes(&len))?;
            }
            _ => return Err(ser::Error::custom("unexpected format")),
        };
        seq.serialize_element(&Bytes(&self.kind.to_be_bytes()))?;
        seq.serialize_element(&Bytes(self.data))?;

        seq.end()
    }
}

/// De/Serialize [messagepack_core::extension::ExtensionRef]
///
/// ## Example
///
/// ```rust
/// use serde::{Serialize,Deserialize};
/// use messagepack_core::extension::ExtensionRef;
///
/// #[derive(Debug, Serialize, Deserialize, PartialEq)]
/// #[serde(transparent)]
/// struct WrapRef<'a>(
///     #[serde(with = "messagepack_serde::extension::ext_ref", borrow)] ExtensionRef<'a>,
/// );
///
/// # fn main() {
///
/// let ext = WrapRef(
///     ExtensionRef::new(10,&[0,1,2,3,4,5])
/// );
/// let mut buf = [0u8; 9];
/// messagepack_serde::to_slice(&ext, &mut buf).unwrap();
///
/// let result = messagepack_serde::from_slice::<WrapRef<'_>>(&buf).unwrap();
/// assert_eq!(ext,result);
///
/// # }
/// ```
pub mod ext_ref {
    use super::*;
    use serde::de;

    /// Serialize [messagepack_core::extension::ExtensionRef]
    pub fn serialize<S>(
        ext: &messagepack_core::extension::ExtensionRef<'_>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct(
            EXTENSION_STRUCT_NAME,
            &ExtInner {
                kind: ext.r#type,
                data: ext.data,
            },
        )
    }

    /// Deserialize [messagepack_core::extension::ExtensionRef]
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<messagepack_core::extension::ExtensionRef<'de>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ExtensionVisitor;

        impl<'de> Visitor<'de> for ExtensionVisitor {
            type Value = messagepack_core::extension::ExtensionRef<'de>;
            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("expect extension")
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                deserializer.deserialize_seq(self)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let kind = seq
                    .next_element::<i8>()?
                    .ok_or(de::Error::missing_field("extension type missing"))?;

                let data = seq
                    .next_element::<&[u8]>()?
                    .ok_or(de::Error::missing_field("extension data missing"))?;

                Ok(messagepack_core::extension::ExtensionRef::new(kind, data))
            }
        }
        deserializer.deserialize_seq(ExtensionVisitor)
    }
}

/// De/Serialize [messagepack_core::extension::FixedExtension]
///
/// ## Example
///
/// ```rust
/// use serde::{Serialize,Deserialize};
/// use messagepack_core::extension::FixedExtension;
///
/// #[derive(Debug, Serialize, Deserialize, PartialEq)]
/// #[serde(transparent)]
/// struct WrapRef(
///     #[serde(with = "messagepack_serde::extension::ext_fixed")] FixedExtension<16>,
/// );
///
/// # fn main() {
///
/// let ext = WrapRef(
///     FixedExtension::new(10,&[0,1,2,3,4,5]).unwrap()
/// );
/// let mut buf = [0u8; 9];
/// messagepack_serde::to_slice(&ext, &mut buf).unwrap();
///
/// let result = messagepack_serde::from_slice::<WrapRef>(&buf).unwrap();
/// assert_eq!(ext,result);
///
/// # }
/// ```
pub mod ext_fixed {
    use super::*;
    use serde::{Deserialize, de};

    /// Serialize [messagepack_core::extension::FixedExtension]
    pub fn serialize<const N: usize, S>(
        ext: &messagepack_core::extension::FixedExtension<N>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        super::ext_ref::serialize(&ext.as_ref(), serializer)
    }

    /// Deserialize [messagepack_core::extension::FixedExtension]
    pub fn deserialize<'de, const N: usize, D>(
        deserializer: D,
    ) -> Result<messagepack_core::extension::FixedExtension<N>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Data<const N: usize> {
            len: usize,
            buf: [u8; N],
        }
        impl<'de, const N: usize> Deserialize<'de> for Data<N> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct DataVisitor<const N: usize>;
                impl<'de, const N: usize> Visitor<'de> for DataVisitor<N> {
                    type Value = Data<N>;
                    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                        formatter.write_str("expect extension")
                    }

                    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        let len = v.len();

                        if len > N {
                            return Err(de::Error::invalid_length(len, &self));
                        }

                        let mut buf = [0; N];
                        buf[..len].copy_from_slice(v);
                        Ok(Data { len, buf })
                    }
                }
                deserializer.deserialize_bytes(DataVisitor)
            }
        }

        struct ExtensionVisitor<const N: usize>;
        impl<'de, const N: usize> Visitor<'de> for ExtensionVisitor<N> {
            type Value = messagepack_core::extension::FixedExtension<N>;
            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("expect extension")
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                deserializer.deserialize_seq(self)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let kind = seq
                    .next_element::<i8>()?
                    .ok_or(serde::de::Error::missing_field("extension type missing"))?;
                let data = seq
                    .next_element::<Data<N>>()?
                    .ok_or(de::Error::missing_field("extension data missing"))?;

                let ext = messagepack_core::extension::FixedExtension::new_fixed_with_prefix(
                    kind, data.len, data.buf,
                )
                .map_err(|_| de::Error::invalid_length(data.len, &"length is too long"))?;
                Ok(ext)
            }
        }

        deserializer.deserialize_seq(ExtensionVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use messagepack_core::extension::{ExtensionRef, FixedExtension};
    use rstest::rstest;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct WrapRef<'a>(
        #[serde(with = "ext_ref", borrow)] messagepack_core::extension::ExtensionRef<'a>,
    );

    #[rstest]
    fn encode_ext_ref() {
        let mut buf = [0_u8; 3];

        let kind: i8 = 123;

        let ext = WrapRef(ExtensionRef::new(kind, &[0x12]));
        let length = crate::to_slice(&ext, &mut buf).unwrap();

        assert_eq!(length, 3);
        assert_eq!(buf, [0xd4, kind.to_be_bytes()[0], 0x12]);
    }

    #[rstest]
    fn decode_ext_ref() {
        let buf = [0xd6, 0xff, 0x00, 0x00, 0x00, 0x00]; // timestamp ext type

        let ext = crate::from_slice::<WrapRef<'_>>(&buf).unwrap().0;
        assert_eq!(ext.r#type, -1);
        let seconds = u32::from_be_bytes(ext.data.try_into().unwrap());
        assert_eq!(seconds, 0);
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct WrapFixed<const N: usize>(
        #[serde(with = "ext_fixed")] messagepack_core::extension::FixedExtension<N>,
    );

    #[rstest]
    fn encode_ext_fixed() {
        let mut buf = [0u8; 3];
        let kind: i8 = 123;

        let ext = WrapFixed(FixedExtension::new_fixed(kind, [0x12]));
        let length = crate::to_slice(&ext, &mut buf).unwrap();

        assert_eq!(length, 3);
        assert_eq!(buf, [0xd4, kind.to_be_bytes()[0], 0x12]);
    }

    const TIMESTAMP32: &[u8] = &[0xd6, 0xff, 0x00, 0x00, 0x00, 0x00];

    #[rstest]
    fn decode_ext_fixed_bigger_will_success() {
        let ext = crate::from_slice::<WrapFixed<6>>(TIMESTAMP32).unwrap().0;
        assert_eq!(ext.r#type, -1);
        assert_eq!(ext.as_slice(), &TIMESTAMP32[2..])
    }

    #[rstest]
    #[should_panic]
    fn decode_ext_fixed_smaller_will_failed() {
        let _ = crate::from_slice::<WrapFixed<3>>(TIMESTAMP32).unwrap();
    }
}
