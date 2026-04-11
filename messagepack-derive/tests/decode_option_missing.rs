use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode as _;
use messagepack_core::io::SliceReader;
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct S1 {
    foo: u8,
    bar: Option<u8>,
}

#[test]
fn allow_option_missing() {
    let data = [0x81, 0xa3, 0x66, 0x6f, 0x6f, 0x0c]; // {"foo": 12}
    let mut reader = SliceReader::new(&data);
    let decoded = <S1 as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, S1 { foo: 12, bar: None });
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct S2 {
    foo: u8,
    #[msgpack(default)]
    bar: u8,
}

#[test]
fn allow_option_default() {
    let data = [0x81, 0xa3, 0x66, 0x6f, 0x6f, 0x0c]; // {"foo": 12}
    let mut reader = SliceReader::new(&data);
    let decoded = <S2 as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, S2 { foo: 12, bar: 0 });
}

#[test]
fn default_named_field_roundtrip_preserves_encoded_value() {
    let value = S2 { foo: 12, bar: 42 };
    let mut buf = Vec::new();
    value.encode(&mut buf).unwrap();

    let expected = [
        0x82, // fixmap 2
        0xa3, b'f', b'o', b'o', 0x0c, // "foo": 12
        0xa3, b'b', b'a', b'r', 0x2a, // "bar": 42
    ];
    assert_eq!(buf, expected);

    let mut reader = SliceReader::new(&buf);
    let decoded = <S2 as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, value);
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct S3<T> {
    foo: u8,
    #[msgpack(default)]
    bar: T,
}

#[test]
fn allow_generic_option_default() {
    let data = [0x81, 0xa3, 0x66, 0x6f, 0x6f, 0x0c]; // {"foo": 12 }
    let mut reader = SliceReader::new(&data);
    let decoded = <S3<u8> as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, S3 { foo: 12, bar: 0 });
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct S4(u8, #[msgpack(default)] u8);

#[test]
fn allow_tuple_default_missing() {
    let data = [0x91, 0x0c]; // [12]
    let mut reader = SliceReader::new(&data);
    let decoded = <S4 as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, S4(12, 0));
}

#[test]
fn default_tuple_field_roundtrip_preserves_encoded_value() {
    let value = S4(12, 42);
    let mut buf = Vec::new();
    value.encode(&mut buf).unwrap();

    let expected = [0x92, 0x0c, 0x2a];
    assert_eq!(buf, expected);

    let mut reader = SliceReader::new(&buf);
    let decoded = <S4 as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, value);
}
