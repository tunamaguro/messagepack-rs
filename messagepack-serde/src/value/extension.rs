use messagepack_core::{
    Format,
    extension::ExtensionRef as CoreExtensionRef,
    io::{IoRead, IoWrite},
};
use serde::{
    Serialize, Serializer,
    de::Visitor,
    ser::{self, SerializeSeq},
};

use crate::ser::Error;

pub(crate) const EXTENSION_STRUCT_NAME: &str = "$__MSGPACK_EXTENSION_STRUCT";

pub(crate) struct SerializeExt<'a, W> {
    writer: &'a mut W,
    length: usize,
}

impl<W> AsMut<Self> for SerializeExt<'_, W> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<'a, W> SerializeExt<'a, W> {
    pub fn new(writer: &'a mut W) -> Self {
        Self { writer, length: 0 }
    }

    pub(crate) fn length(&self) -> usize {
        self.length
    }
}

impl<W: IoWrite> SerializeExt<'_, W> {
    fn unexpected(&self) -> Error<W::Error> {
        ser::Error::custom("unexpected value")
    }
}

impl<'a, 'b, W> ser::Serializer for &'a mut SerializeExt<'b, W>
where
    'b: 'a,
    W: IoWrite,
{
    type Ok = ();

    type Error = Error<W::Error>;

    type SerializeSeq = SerializeExtSeq<'a, 'b, W>;

    type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, _: bool) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(&v.to_be_bytes())
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(&v.to_be_bytes())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(&v.to_be_bytes())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(&v.to_be_bytes())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_bytes(&v.to_be_bytes())
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write(v)
            .map_err(messagepack_core::encode::Error::Io)?;
        self.length += v.len();
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(self.unexpected())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(self.unexpected())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeExtSeq::new(self))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(self.unexpected())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(self.unexpected())
    }

    fn collect_str<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + core::fmt::Display,
    {
        Err(self.unexpected())
    }
}

pub struct SerializeExtSeq<'a, 'b, W> {
    ser: &'a mut SerializeExt<'b, W>,
}

impl<'a, 'b, W> SerializeExtSeq<'a, 'b, W> {
    pub(crate) fn new(ser: &'a mut SerializeExt<'b, W>) -> Self {
        Self { ser }
    }
}

impl<'a, 'b, W> ser::SerializeSeq for SerializeExtSeq<'a, 'b, W>
where
    'b: 'a,
    W: IoWrite,
{
    type Ok = ();
    type Error = Error<W::Error>;
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self.ser.as_mut())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

struct Bytes<'a>(pub &'a [u8]);
impl ser::Serialize for Bytes<'_> {
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

impl ser::Serialize for ExtInner<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoder = CoreExtensionRef::new(self.kind, self.data);
        let format = encoder
            .to_format::<core::convert::Infallible>()
            .map_err(|_| ser::Error::custom("Invalid data length"))?;

        let mut seq = serializer.serialize_seq(Some(4))?;

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

pub(crate) struct DeserializeExt<'a, R> {
    data_len: usize,
    reader: &'a mut R,
}

impl<'a, R> AsMut<Self> for DeserializeExt<'a, R> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<'de, 'a, R> DeserializeExt<'a, R>
where
    R: IoRead<'de>,
{
    pub(crate) fn new(
        format: Format,
        reader: &'a mut R,
    ) -> Result<Self, crate::de::Error<R::Error>> {
        use messagepack_core::decode::NbyteReader;
        let data_len = match format {
            Format::FixExt1 => 1,
            Format::FixExt2 => 2,
            Format::FixExt4 => 4,
            Format::FixExt8 => 8,
            Format::FixExt16 => 16,
            Format::Ext8 => {
                
                NbyteReader::<1>::read(reader)?
            }
            Format::Ext16 => {
                
                NbyteReader::<2>::read(reader)?
            }
            Format::Ext32 => {
                
                NbyteReader::<4>::read(reader)?
            }
            _ => return Err(messagepack_core::decode::Error::UnexpectedFormat.into()),
        };
        Ok(DeserializeExt { data_len, reader })
    }
}

impl<'de, 'a, R> serde::Deserializer<'de> for &mut DeserializeExt<'a, R>
where
    R: IoRead<'de>,
{
    type Error = crate::de::Error<R::Error>;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(serde::de::Error::custom(
            "any when deserialize extension is not supported",
        ))
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let slice = self
            .reader
            .read_slice(1)
            .map_err(messagepack_core::decode::Error::Io)?;
        let buf: [u8; 1] = slice
            .as_bytes()
            .try_into()
            .map_err(|_| messagepack_core::decode::Error::UnexpectedEof)?;

        let val = i8::from_be_bytes(buf);
        visitor.visit_i8(val)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let slice = self
            .reader
            .read_slice(self.data_len)
            .map_err(messagepack_core::decode::Error::Io)?;
        match slice {
            messagepack_core::io::Reference::Borrowed(items) => visitor.visit_borrowed_bytes(items),
            messagepack_core::io::Reference::Copied(items) => visitor.visit_bytes(items),
        }
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(&mut self)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    serde::forward_to_deserialize_any! {
        bool i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        byte_buf option unit unit_struct tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, 'a, R> serde::de::SeqAccess<'de> for &mut DeserializeExt<'a, R>
where
    R: IoRead<'de>,
{
    type Error = crate::de::Error<R::Error>;
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.as_mut()).map(Some)
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
///     #[serde(with = "messagepack_serde::value::ext_ref", borrow)] ExtensionRef<'a>,
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
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_seq(self)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let kind = seq
                    .next_element::<i8>()?
                    .ok_or(serde::de::Error::custom("expect i8"))?;

                let data = seq
                    .next_element::<&[u8]>()?
                    .ok_or(serde::de::Error::custom("expect [u8]"))?;

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
///     #[serde(with = "messagepack_serde::value::ext_fixed")] FixedExtension<16>,
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
    use serde::de;

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
        let r = super::ext_ref::deserialize(deserializer)?;

        let ext = messagepack_core::extension::FixedExtension::new(r.r#type, r.data)
            .ok_or_else(|| de::Error::custom("extension length is too long"))?;
        Ok(ext)
    }
}

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
///     #[serde(with = "messagepack_serde::value::timestamp32")] Timestamp32,
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
///     #[serde(with = "messagepack_serde::value::timestamp64")] Timestamp64,
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
///     #[serde(with = "messagepack_serde::value::timestamp96")] Timestamp96,
/// );
///
/// # fn main() {
/// let v = Wrap(Timestamp96::new(123456, 789));
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
    use messagepack_core::extension::{ExtensionRef, FixedExtension};
    use messagepack_core::timestamp::{Timestamp32, Timestamp64, Timestamp96};
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
        let ts = WrapTs96(Timestamp96::new(123456, 789));
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
