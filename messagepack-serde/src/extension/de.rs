use messagepack_core::{Format, io::IoRead};
use serde::de::Visitor;
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
            Format::Ext8 => NbyteReader::<1>::read(reader)?,
            Format::Ext16 => NbyteReader::<2>::read(reader)?,
            Format::Ext32 => NbyteReader::<4>::read(reader)?,
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
