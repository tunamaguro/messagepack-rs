use std::marker::PhantomData;

use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_core::io::SliceReader;
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Foo {
    data: PhantomData<u8>,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Bar<T> {
    value: u8,
    marker: PhantomData<T>,
}

fn phantom_only_field_is_omitted() {
    let foo = Foo { data: PhantomData };

    let mut buf = Vec::new();
    foo.encode(&mut buf).unwrap();
    let expected = [0x80]; // fixmap 0
    assert_eq!(buf, expected);

    let mut reader = SliceReader::new(&buf);
    let decoded = <Foo as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, foo);
}

#[test]
fn phantom_field_does_not_contribute_to_map_shape() {
    let bar = Bar {
        value: 7,
        marker: PhantomData,
    };

    let mut buf = Vec::new();
    bar.encode(&mut buf).unwrap();
    let expected = [0x81, 0xa5, b'v', b'a', b'l', b'u', b'e', 0x07]; // fixmap 1 , "value" => 7
    assert_eq!(buf, expected);

    let mut reader = SliceReader::new(&buf);
    let decoded = <Bar<u8> as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, bar);
}

#[derive(Debug, PartialEq, Encode, Decode)]
#[msgpack(array)]
struct Baz<T> {
    #[msgpack(key = 0)]
    value: u8,
    #[msgpack(key = 1)]
    marker: PhantomData<T>,
}

#[test]
fn phantom_field_does_not_contribute_to_array_shape() {
    let baz = Baz {
        value: 7,
        marker: PhantomData,
    };

    let mut buf = Vec::new();
    baz.encode(&mut buf).unwrap();
    let expected = [0x91, 0x07];
    assert_eq!(buf, expected);

    let mut reader = SliceReader::new(&buf);
    let decoded = <Baz<u8> as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, baz);
}
