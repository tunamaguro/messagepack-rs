// Test: array mode with key indices
use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_core::io::SliceReader;
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[msgpack(array)]
struct Record {
    #[msgpack(key = 0)]
    name: String,
    #[msgpack(key = 1)]
    age: u8,
}

fn main() {
    let r = Record {
        name: "Alice".to_string(),
        age: 30,
    };
    let mut buf = Vec::new();
    r.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <Record as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, r);
}
