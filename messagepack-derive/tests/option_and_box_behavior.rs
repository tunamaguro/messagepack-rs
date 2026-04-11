use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_core::io::SliceReader;
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct NamedWithTrailingOption {
    foo: u8,
    bar: Option<u8>,
}

#[test]
fn named_struct_array_decode_missing_trailing_option_is_none() {
    let data = [0x91, 0x0c]; // [12]
    let mut reader = SliceReader::new(&data);
    let decoded = <NamedWithTrailingOption as Decode>::decode(&mut reader).unwrap();

    assert_eq!(decoded, NamedWithTrailingOption { foo: 12, bar: None });
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct WithBoxField {
    value: Box<u32>,
}

#[test]
fn named_struct_box_field_roundtrips() {
    let value = WithBoxField {
        value: Box::new(42),
    };
    let mut buf = Vec::new();
    value.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <WithBoxField as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, value);
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct WithOptionalBoxField {
    value: Option<Box<u32>>,
}

#[test]
fn named_struct_option_box_some_roundtrips() {
    let value = WithOptionalBoxField {
        value: Some(Box::new(42)),
    };
    let mut buf = Vec::new();
    value.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <WithOptionalBoxField as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn named_struct_option_box_none_roundtrips() {
    let value = WithOptionalBoxField { value: None };
    let mut buf = Vec::new();
    value.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <WithOptionalBoxField as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, value);
}
