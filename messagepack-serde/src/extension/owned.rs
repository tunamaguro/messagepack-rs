/// De/Serialize [messagepack_core::extension::ExtensionOwned]
///
/// ## Example
///
/// ```rust
/// use serde::{Serialize,Deserialize};
/// use messagepack_core::extension::ExtensionOwned;
///
/// #[derive(Debug, Serialize, Deserialize, PartialEq)]
/// #[serde(transparent)]
/// struct WrapOwned(
///     #[serde(with = "messagepack_serde::extension::ext_owned")] ExtensionOwned,
/// );
///
/// # fn main() {
///
/// let ext = WrapOwned(
///     ExtensionOwned::new(10, vec![0,1,2,3,4,5])
/// );
/// let mut buf = [0u8; 9];
/// messagepack_serde::to_slice(&ext, &mut buf).unwrap();
///
/// let result = messagepack_serde::from_slice::<WrapOwned>(&buf).unwrap();
/// assert_eq!(ext,result);
///
/// # }
/// ```
pub mod ext_owned {
    use crate::extension::ext_ref;
    use serde::{
        Deserialize,
        de::{self, Visitor},
    };

    /// Serialize [messagepack_core::extension::ExtensionOwned]
    pub fn serialize<S>(
        ext: &messagepack_core::extension::ExtensionOwned,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ext_ref::serialize(&ext.as_ref(), serializer)
    }

    /// Deserialize [messagepack_core::extension::ExtensionOwned]
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<messagepack_core::extension::ExtensionOwned, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct VecStruct(alloc::vec::Vec<u8>);
        impl<'de> Deserialize<'de> for VecStruct {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct VecVisitor;
                impl<'de> Visitor<'de> for VecVisitor {
                    type Value = VecStruct;
                    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                        formatter.write_str("expect extension bytes")
                    }

                    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        let v = alloc::vec::Vec::from(v);
                        Ok(VecStruct(v))
                    }
                }
                deserializer.deserialize_bytes(VecVisitor)
            }
        }

        struct ExtensionVisitor;

        impl<'de> Visitor<'de> for ExtensionVisitor {
            type Value = messagepack_core::extension::ExtensionOwned;
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
                    .next_element::<VecStruct>()?
                    .ok_or(de::Error::missing_field("extension data missing"))?;

                Ok(messagepack_core::extension::ExtensionOwned::new(
                    kind, data.0,
                ))
            }
        }
        deserializer.deserialize_seq(ExtensionVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use messagepack_core::extension::ExtensionOwned;
    use rstest::rstest;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct WrapOwned(#[serde(with = "ext_owned")] ExtensionOwned);

    #[rstest]
    fn encode_ext_owned_fix() {
        let mut buf = [0u8; 3];
        let kind: i8 = 123;
        let ext = WrapOwned(ExtensionOwned::new(kind, vec![0x12]));
        let length = crate::to_slice(&ext, &mut buf).unwrap();

        assert_eq!(length, 3);
        assert_eq!(buf, [0xd4, kind.to_be_bytes()[0], 0x12]);
    }

    #[rstest]
    fn encode_ext_owned_ext8() {
        let kind: i8 = 42;
        let data = vec![0xAB; 17];
        let ext = WrapOwned(ExtensionOwned::new(kind, data.clone()));
        let mut buf = [0u8; 1 + 1 + 1 + 17]; // marker + len + type + payload
        let n = crate::to_slice(&ext, &mut buf).unwrap();

        assert_eq!(n, buf.len());
        assert_eq!(buf[0], 0xc7); // Ext8
        assert_eq!(buf[1], 17);
        assert_eq!(buf[2], kind.to_be_bytes()[0]);
        assert_eq!(&buf[3..], &data[..]);
    }

    const SAMPLE_FIXEXT4: &[u8] = &[0xd6, 0x07, 0xDE, 0xAD, 0xBE, 0xEF];

    #[rstest]
    fn decode_ext_owned() {
        let v = crate::from_slice::<WrapOwned>(SAMPLE_FIXEXT4).unwrap();
        assert_eq!(v.0.r#type, 7);
        assert_eq!(v.0.data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }
}
