use super::Value;
use alloc::{vec, vec::Vec};
use serde::ser::{self};

type Error = crate::ser::Error<core::convert::Infallible>;

struct Serializer;

impl ser::Serializer for Serializer {
    type Ok = Value;
    type Error = Error;
    type SerializeSeq = SerializeSeq;
    type SerializeTuple = SerializeSeq;
    type SerializeTupleStruct = SerializeSeq;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Value::from(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::from(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::from(v))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::from(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        use crate::alloc::string::ToString;
        Ok(Value::String(v.to_string()))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::from(v))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(Value::from(v))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Nil)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        match name {
            crate::extension::EXTENSION_STRUCT_NAME => {
                let mut ser = SerializeExt::new();
                value.serialize(&mut ser)?;
                ser.into_value()
            }
            _ => value.serialize(self),
        }
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let val = value.serialize(self)?;
        let key = Value::from(variant);
        Ok(Value::Map(vec![(key, val)]))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeSeq::with_capacity(len))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeTupleVariant::with_capacity(variant, len))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap::with_capacity(len))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeStructVariant::with_capacity(variant, len))
    }
}

struct SerializeSeq {
    values: Vec<Value>,
}

impl SerializeSeq {
    fn with_capacity(len: Option<usize>) -> Self {
        Self {
            values: len.map(Vec::with_capacity).unwrap_or_default(),
        }
    }
}

impl ser::SerializeSeq for SerializeSeq {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let val = value.serialize(Serializer)?;
        self.values.push(val);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Array(self.values))
    }
}

impl ser::SerializeTuple for SerializeSeq {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for SerializeSeq {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

struct SerializeTupleVariant {
    variant_name: &'static str,
    seq: SerializeSeq,
}

impl SerializeTupleVariant {
    fn with_capacity(name: &'static str, len: usize) -> Self {
        Self {
            variant_name: name,
            seq: SerializeSeq::with_capacity(Some(len)),
        }
    }
}

impl ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(&mut self.seq, value)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let array = ser::SerializeSeq::end(self.seq)?;
        let key = Value::from(self.variant_name);
        Ok(Value::Map(vec![(key, array)]))
    }
}

struct SerializeMap {
    key: Option<Value>,
    items: Vec<(Value, Value)>,
}

impl SerializeMap {
    fn with_capacity(len: Option<usize>) -> Self {
        Self {
            items: len.map(Vec::with_capacity).unwrap_or_default(),
            key: None,
        }
    }
}

impl ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let key = key.serialize(Serializer)?;
        self.key = Some(key);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let key = self
            .key
            .take()
            .ok_or(<Error as ser::Error>::custom("missing map key"))?;
        let val = value.serialize(Serializer)?;
        self.items.push((key, val));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Map(self.items))
    }
}

impl ser::SerializeStruct for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeMap::serialize_key(self, key)?;
        ser::SerializeMap::serialize_value(self, value)?;
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeMap::end(self)
    }
}

struct SerializeStructVariant {
    variant_name: &'static str,
    map: SerializeMap,
}

impl SerializeStructVariant {
    fn with_capacity(name: &'static str, len: usize) -> Self {
        Self {
            variant_name: name,
            map: SerializeMap::with_capacity(Some(len)),
        }
    }
}

impl ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Value;
    type Error = Error;
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeStruct::serialize_field(&mut self.map, key, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        let map = ser::SerializeStruct::end(self.map)?;
        let key = Value::from(self.variant_name);
        Ok(Value::Map(vec![(key, map)]))
    }
}

struct SerializeExt {
    format_seen: bool,
    data_length: Option<u32>,
    kind: Option<i8>,
    data: Option<Vec<u8>>,
}

impl SerializeExt {
    fn new() -> Self {
        Self {
            format_seen: false,
            data_length: None,
            kind: None,
            data: None,
        }
    }

    fn into_value(self) -> Result<Value, Error> {
        let Self {
            format_seen: _,
            data_length: _,
            kind,
            data,
        } = self;

        let Some(kind) = kind else {
            return Err(ser::Error::custom("extension type not found"));
        };
        let Some(data) = data else {
            return Err(ser::Error::custom("extension data not found"));
        };
        let ext = messagepack_core::extension::ExtensionOwned::new(kind, data);
        Ok(Value::from(ext))
    }

    fn unsupported_type() -> Error {
        ser::Error::custom("support only `i8`, `u8`, `u16`, `u32`, `bytes` and `seq`")
    }
}

impl AsMut<Self> for SerializeExt {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<'a> ser::Serializer for &'a mut SerializeExt {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = SerializeExtSeq<'a>;
    type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.kind = Some(v);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.data_length = Some(u32::from(v));
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.data_length = Some(u32::from(v));
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.data_length = Some(v);
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        // first time
        if !self.format_seen {
            self.format_seen = true;
            return Ok(());
        };

        self.data = Some(v.to_vec());
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let seq = SerializeExtSeq { ser: self };
        Ok(seq)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(SerializeExt::unsupported_type())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(SerializeExt::unsupported_type())
    }
}

struct SerializeExtSeq<'a> {
    ser: &'a mut SerializeExt,
}

impl<'a> ser::SerializeSeq for SerializeExtSeq<'a> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self.ser.as_mut())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde::Serialize;
    use serde_bytes::ByteBuf;

    #[derive(Serialize)]
    enum Kind<'a> {
        Unit,
        New(u8),
        Tup(u8, u16),
        Str { a: bool, b: &'a str },
    }

    #[rstest]
    #[case(Kind::Unit, Value::from("Unit"))]
    #[case(Kind::New(5), Value::Map(
        vec![(Value::from("New"), Value::from(5))]
    ))]
    #[case(Kind::Tup(1,2), Value::Map(
        vec![(
                Value::from("Tup"),
                Value::Array(vec![Value::from(1), Value::from(2)])
            )]
    ))]
    #[case(
        Kind::Str { a: false, b: "hi" },
        Value::Map(vec![(
                Value::from("Str"),
                Value::Map(vec![
                    (Value::from("a"), Value::from(false)),
                    (Value::from("b"), Value::from("hi")),
                ]),
        ),])
    )]
    fn serialize_enum(#[case] val: Kind, #[case] expected: Value) {
        let serialized = val.serialize(Serializer).unwrap();
        assert_eq!(serialized, expected);
    }
    #[derive(Debug, Serialize)]
    struct WrappedRef(
        #[serde(with = "crate::extension::ext_ref")]
        messagepack_core::extension::ExtensionRef<'static>,
    );

    impl WrappedRef {
        fn new(kind: i8, data: &'static [u8]) -> Self {
            Self(messagepack_core::extension::ExtensionRef::new(kind, data))
        }
    }

    #[rstest]
    fn serialize_extension() {
        let val = WrappedRef::new(8, &[1, 2, 3, 4]);
        let serialized = val.serialize(Serializer).unwrap();

        let expected = Value::Extension(messagepack_core::extension::ExtensionOwned::new(
            8,
            vec![1, 2, 3, 4],
        ));

        assert_eq!(serialized, expected);
    }

    // primitives and simple composites
    #[rstest]
    #[case(true)]
    #[case(false)]
    fn serialize_bool_primitives(#[case] v: bool) {
        let serialized = v.serialize(Serializer).unwrap();
        assert_eq!(serialized, Value::from(v));
    }

    #[rstest]
    #[case(0i8)]
    #[case(-1)]
    #[case(127)]
    fn serialize_i8_numbers(#[case] v: i8) {
        let serialized = v.serialize(Serializer).unwrap();
        assert_eq!(serialized, Value::from(v));
    }

    #[rstest]
    #[case(0i16)]
    #[case(-128)]
    #[case(1024)]
    fn serialize_i16_numbers(#[case] v: i16) {
        let serialized = v.serialize(Serializer).unwrap();
        assert_eq!(serialized, Value::from(v));
    }

    #[rstest]
    #[case(0i32)]
    #[case(-32768)]
    #[case(1_000_000)]
    fn serialize_i32_numbers(#[case] v: i32) {
        let serialized = v.serialize(Serializer).unwrap();
        assert_eq!(serialized, Value::from(v));
    }

    #[rstest]
    #[case(0i64)]
    #[case(-2147483648)]
    #[case(9_223_372_036_854_775_807i64)]
    fn serialize_i64_numbers(#[case] v: i64) {
        let serialized = v.serialize(Serializer).unwrap();
        assert_eq!(serialized, Value::from(v));
    }

    #[rstest]
    #[case(0u8)]
    #[case(255)]
    fn serialize_u8_numbers(#[case] v: u8) {
        let serialized = v.serialize(Serializer).unwrap();
        assert_eq!(serialized, Value::from(v));
    }

    #[rstest]
    #[case(0u16)]
    #[case(65_535)]
    fn serialize_u16_numbers(#[case] v: u16) {
        let serialized = v.serialize(Serializer).unwrap();
        assert_eq!(serialized, Value::from(v));
    }

    #[rstest]
    #[case(0u32)]
    #[case(4_294_967_295)]
    fn serialize_u32_numbers(#[case] v: u32) {
        let serialized = v.serialize(Serializer).unwrap();
        assert_eq!(serialized, Value::from(v));
    }

    #[rstest]
    #[case(0u64)]
    #[case(18_446_744_073_709_551_615u64)]
    fn serialize_u64_numbers(#[case] v: u64) {
        let serialized = v.serialize(Serializer).unwrap();
        assert_eq!(serialized, Value::from(v));
    }

    #[rstest]
    #[case(0.0f32)]
    #[case(-0.0)]
    #[case(1.5)]
    fn serialize_f32_numbers(#[case] v: f32) {
        let serialized = v.serialize(Serializer).unwrap();
        assert_eq!(serialized, Value::from(v));
    }

    #[rstest]
    #[case(0.0f64)]
    #[case(-0.0)]
    #[case(1.5)]
    fn serialize_f64_numbers(#[case] v: f64) {
        let serialized = v.serialize(Serializer).unwrap();
        assert_eq!(serialized, Value::from(v));
    }

    #[rstest]
    #[case('a', Value::String("a".to_string()))]
    #[case('😀', Value::String("😀".to_string()))]
    fn serialize_char_as_string(#[case] ch: char, #[case] expected: Value) {
        let serialized = ch.serialize(Serializer).unwrap();
        assert_eq!(serialized, expected);
    }

    #[rstest]
    #[case("")]
    #[case("hello")]
    fn serialize_strs(#[case] s: &str) {
        let serialized = s.serialize(Serializer).unwrap();
        assert_eq!(serialized, Value::from(s));
    }

    #[rstest]
    #[case(vec![])]
    #[case(vec![9u8, 8, 7, 6])]
    fn serialize_bytes_via_bytebuf(#[case] data: Vec<u8>) {
        let bb = ByteBuf::from(data.clone());
        let serialized = bb.serialize(Serializer).unwrap();
        assert_eq!(serialized, Value::Bin(data));
    }

    #[rstest]
    #[case(vec![])]
    #[case(vec![1u8, 2, 3])]
    fn serialize_slice_u8_as_array(#[case] data: Vec<u8>) {
        let s: &[u8] = &data;
        let serialized = s.serialize(Serializer).unwrap();
        assert_eq!(
            serialized,
            Value::Array(data.into_iter().map(Value::from).collect())
        );
    }

    #[rstest]
    #[case(vec![])]
    #[case(vec![1u8, 2, 3])]
    fn serialize_vec_u8_as_array(#[case] data: Vec<u8>) {
        let serialized = data.serialize(Serializer).unwrap();
        assert_eq!(
            serialized,
            Value::Array(data.into_iter().map(Value::from).collect())
        );
    }

    #[rstest]
    #[case(vec![])]
    #[case(vec![1u8, 2, 3])]
    fn serialize_bytes_via_bytes_wrapper(#[case] data: Vec<u8>) {
        let bytes = serde_bytes::Bytes::new(&data);
        let serialized = bytes.serialize(Serializer).unwrap();
        assert_eq!(serialized, Value::Bin(data));
    }

    #[derive(Serialize)]
    struct U;

    #[rstest]
    #[case(U)]
    #[case(())]
    #[case(Option::<u8>::None)]
    fn serialize_option_unit_and_unit_struct<V>(#[case] val: V)
    where
        V: Serialize,
    {
        assert_eq!(val.serialize(Serializer).unwrap(), Value::Nil)
    }

    #[rstest]
    fn serialize_newtype_struct_plain() {
        #[derive(Serialize)]
        struct Wrapper(u16);
        let v = Wrapper(7);
        // Should delegate to inner
        assert_eq!(v.serialize(Serializer).unwrap(), Value::from(7));
    }

    #[derive(Serialize)]
    struct Tup(u8, i16);

    #[rstest]
    #[case(
        (1u8, 2u16, 3i32),
        Value::Array(vec![Value::from(1), Value::from(2), Value::from(3)])
    )]
    #[case(
        Tup(1, -2),
        Value::Array(vec![Value::from(1), Value::from(-2)])
    )]
    fn serialize_tuple_and_tuple_struct<V>(#[case] val: V, #[case] expected: Value)
    where
        V: Serialize,
    {
        assert_eq!(val.serialize(Serializer).unwrap(), expected)
    }

    #[rstest]
    fn serialize_struct_to_map() {
        #[derive(Serialize)]
        struct S<'a> {
            a: u8,
            #[serde(rename = "msg")]
            b: &'a str,
        }
        let v = S { a: 7, b: "hi" };
        let s = v.serialize(Serializer).unwrap();
        assert_eq!(
            s,
            Value::Map(vec![
                (Value::from("a"), Value::from(7u8)),
                (Value::from("msg"), Value::from("hi")),
            ])
        );
    }

    #[test]
    fn serialize_seq_and_map_with_unknown_len() {
        // Serialize a seq with None length
        struct DynSeq;
        impl Serialize for DynSeq {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                use serde::ser::SerializeSeq;
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element(&1u8)?;
                seq.serialize_element(&2u16)?;
                seq.serialize_element(&3i32)?;
                seq.end()
            }
        }
        let seq_val = DynSeq.serialize(Serializer).unwrap();
        assert_eq!(
            seq_val,
            Value::Array(vec![Value::from(1u8), Value::from(2u16), Value::from(3i32)])
        );

        // Serialize a map with None length
        struct DynMap;
        impl Serialize for DynMap {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry(&"k1", &10u8)?;
                map.serialize_entry(&2u8, &"v2")?;
                map.end()
            }
        }
        let map_val = DynMap.serialize(Serializer).unwrap();
        assert_eq!(
            map_val,
            Value::Map(vec![
                (Value::from("k1"), Value::from(10u8)),
                (Value::from(2u8), Value::from("v2")),
            ])
        );
    }
}
