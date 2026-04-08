// Test: lifetime generics support
use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_core::io::SliceReader;
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Wrapper<'a> {
    value: &'a str,
}

fn main() {
    let w = Wrapper { value: "hello" };
    let mut buf = Vec::new();
    w.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <Wrapper<'_> as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, w);
}
