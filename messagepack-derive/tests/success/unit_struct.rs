// Test: unit struct encode/decode
use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_core::io::SliceReader;
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Unit;

fn main() {
    let u = Unit;
    let mut buf = Vec::new();
    u.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <Unit as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, u);
}
