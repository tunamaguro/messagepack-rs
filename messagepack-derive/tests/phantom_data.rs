use std::marker::PhantomData;

use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_core::io::SliceReader;
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Foo {
    data: PhantomData<u8>,
}

#[test]
fn array_types() {
    let foo = Foo { data: PhantomData };

    let mut buf = Vec::new();
    foo.encode(&mut buf).unwrap();
    let expected = [0x81]; // fixmap 1
    assert_eq!(buf, expected);

    let mut reader = SliceReader::new(&buf);
    let decoded = <Foo as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, foo);
}
