// Test: basic named struct encode/decode as map (default)
use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_core::io::SliceReader;
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Point {
    x: u32,
    y: u32,
}

fn main() {
    let p = Point { x: 10, y: 20 };
    let mut buf = Vec::new();
    p.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <Point as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, p);
}
