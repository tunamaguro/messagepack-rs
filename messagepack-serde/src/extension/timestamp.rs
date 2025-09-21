use crate::extension::ext_fixed;

/// De/Serialize messagepack timestamp 32 extension.
///
/// This module allows serializing and deserializing
/// `messagepack_core::timestamp::Timestamp32` as MessagePack
/// timestamp extension (type `-1`, 4‑byte payload).
///
/// ## Example
///
/// ```rust
/// use serde::{Serialize,Deserialize};
/// use messagepack_core::timestamp::Timestamp32;
///
/// #[derive(Debug, Serialize, Deserialize, PartialEq)]
/// struct Wrap(
///     #[serde(with = "messagepack_serde::extension::timestamp32")] Timestamp32,
/// );
///
/// # fn main() {
/// let v = Wrap(Timestamp32::new(123456));
/// let mut buf = [0u8; 16];
/// let n = messagepack_serde::to_slice(&v, &mut buf).unwrap();
/// let back = messagepack_serde::from_slice::<Wrap>(&buf[..n]).unwrap();
/// assert_eq!(v, back);
/// # }
/// ```
pub mod timestamp32 {
    /// Serialize `Timestamp32` as MessagePack extension.
    pub fn serialize<S>(
        ts: &messagepack_core::timestamp::Timestamp32,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let ext: messagepack_core::extension::FixedExtension<4> = (*ts).into();
        super::ext_fixed::serialize::<4, _>(&ext, serializer)
    }

    /// Deserialize `Timestamp32` from MessagePack extension.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<messagepack_core::timestamp::Timestamp32, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let ext = super::ext_fixed::deserialize::<4, _>(deserializer)?;
        ext.try_into()
            .map_err(|_| serde::de::Error::custom("invalid timestamp32"))
    }
}

/// De/Serialize messagepack timestamp 64 extension.
///
/// This module allows serializing and deserializing
/// `messagepack_core::timestamp::Timestamp64` as MessagePack
/// timestamp extension (type `-1`, 8‑byte payload).
///
/// ## Example
///
/// ```rust
/// use serde::{Serialize,Deserialize};
/// use messagepack_core::timestamp::Timestamp64;
///
/// #[derive(Debug, Serialize, Deserialize, PartialEq)]
/// struct Wrap(
///     #[serde(with = "messagepack_serde::extension::timestamp64")] Timestamp64,
/// );
///
/// # fn main() {
/// let v = Wrap(Timestamp64::new(123456, 789).unwrap());
/// let mut buf = [0u8; 32];
/// let n = messagepack_serde::to_slice(&v, &mut buf).unwrap();
/// let back = messagepack_serde::from_slice::<Wrap>(&buf[..n]).unwrap();
/// assert_eq!(v, back);
/// # }
/// ```
pub mod timestamp64 {
    /// Serialize `Timestamp64` as MessagePack extension.
    pub fn serialize<S>(
        ts: &messagepack_core::timestamp::Timestamp64,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let ext: messagepack_core::extension::FixedExtension<8> = (*ts).into();
        super::ext_fixed::serialize::<8, _>(&ext, serializer)
    }

    /// Deserialize `Timestamp64` from MessagePack extension.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<messagepack_core::timestamp::Timestamp64, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let ext = super::ext_fixed::deserialize::<8, _>(deserializer)?;
        ext.try_into()
            .map_err(|_| serde::de::Error::custom("invalid timestamp64"))
    }
}

/// De/Serialize messagepack timestamp 96 extension.
///
/// This module allows serializing and deserializing
/// `messagepack_core::timestamp::Timestamp96` as MessagePack
/// timestamp extension (type `-1`, 12‑byte payload).
///
/// ## Example
///
/// ```rust
/// use serde::{Serialize,Deserialize};
/// use messagepack_core::timestamp::Timestamp96;
///
/// #[derive(Debug, Serialize, Deserialize, PartialEq)]
/// struct Wrap(
///     #[serde(with = "messagepack_serde::extension::timestamp96")] Timestamp96,
/// );
///
/// # fn main() {
/// let v = Wrap(Timestamp96::new(123456, 789).unwrap());
/// let mut buf = [0u8; 32];
/// let n = messagepack_serde::to_slice(&v, &mut buf).unwrap();
/// let back = messagepack_serde::from_slice::<Wrap>(&buf[..n]).unwrap();
/// assert_eq!(v, back);
/// # }
/// ```
pub mod timestamp96 {
    /// Serialize `Timestamp96` as MessagePack extension.
    pub fn serialize<S>(
        ts: &messagepack_core::timestamp::Timestamp96,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let ext: messagepack_core::extension::FixedExtension<12> = (*ts).into();
        super::ext_fixed::serialize::<12, _>(&ext, serializer)
    }

    /// Deserialize `Timestamp96` from MessagePack extension.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<messagepack_core::timestamp::Timestamp96, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let ext = super::ext_fixed::deserialize::<12, _>(deserializer)?;
        ext.try_into()
            .map_err(|_| serde::de::Error::custom("invalid timestamp96"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use messagepack_core::timestamp::{Timestamp32, Timestamp64, Timestamp96};

    use rstest::rstest;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct WrapTs32(#[serde(with = "timestamp32")] Timestamp32);

    #[rstest]
    fn encode_timestamp32() {
        let ts = WrapTs32(Timestamp32::new(123456));
        let mut buf = [0u8; 16];
        let n = crate::to_slice(&ts, &mut buf).unwrap();

        let mut expected = vec![0xd6, (-1i8 as u8)];
        expected.extend_from_slice(&123456u32.to_be_bytes());
        assert_eq!(&buf[..n], expected.as_slice());
    }

    #[rstest]
    fn decode_timestamp32() {
        let mut buf = vec![0xd6, (-1i8 as u8)];
        buf.extend_from_slice(&0u32.to_be_bytes());
        let v = crate::from_slice::<WrapTs32>(&buf).unwrap();
        assert_eq!(v.0.seconds(), 0);
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct WrapTs64(#[serde(with = "timestamp64")] Timestamp64);

    #[rstest]
    fn encode_timestamp64() {
        let ts = WrapTs64(Timestamp64::new(123456, 789).unwrap());
        let mut buf = [0u8; 32];
        let n = crate::to_slice(&ts, &mut buf).unwrap();

        let mut expected = vec![0xd7, (-1i8 as u8)];
        let data = ((789u64 << 34) | 123456).to_be_bytes();
        expected.extend_from_slice(&data);
        assert_eq!(&buf[..n], expected.as_slice());
    }

    #[rstest]
    fn decode_timestamp64() {
        let mut buf = vec![0xd7, (-1i8 as u8)];
        let data = ((789u64 << 34) | 123456).to_be_bytes();
        buf.extend_from_slice(&data);
        let v = crate::from_slice::<WrapTs64>(&buf).unwrap();
        assert_eq!(v.0.seconds(), 123456);
        assert_eq!(v.0.nanos(), 789);
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct WrapTs96(#[serde(with = "timestamp96")] Timestamp96);

    #[rstest]
    fn encode_timestamp96() {
        let ts = WrapTs96(Timestamp96::new(123456, 789).unwrap());
        let mut buf = [0u8; 32];
        let n = crate::to_slice(&ts, &mut buf).unwrap();

        let mut expected = vec![0xc7, 12, (-1i8 as u8)];
        expected.extend_from_slice(&789u32.to_be_bytes());
        expected.extend_from_slice(&123456u64.to_be_bytes());
        assert_eq!(&buf[..n], expected.as_slice());
    }

    #[rstest]
    fn decode_timestamp96() {
        let mut buf = vec![0xc7, 12, (-1i8 as u8)];
        buf.extend_from_slice(&789u32.to_be_bytes());
        buf.extend_from_slice(&123456u64.to_be_bytes());
        let v = crate::from_slice::<WrapTs96>(&buf).unwrap();
        assert_eq!(v.0.seconds(), 123456);
        assert_eq!(v.0.nanos(), 789);
    }
}
