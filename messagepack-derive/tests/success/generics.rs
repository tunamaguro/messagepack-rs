// Test: generics support
use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_core::io::SliceReader;
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Wrapper<T> {
    value: T,
}

fn main() {
    let w = Wrapper { value: 42u32 };
    let mut buf = Vec::new();
    w.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <Wrapper<u32> as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, w);
}
