use messagepack_core::decode::Decode;
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
