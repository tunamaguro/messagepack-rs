// Test: tuple struct encode/decode
use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_core::io::SliceReader;
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Pair(u8, u16);

fn main() {
    let p = Pair(42, 1000);
    let mut buf = Vec::new();
    p.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <Pair as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, p);
}
