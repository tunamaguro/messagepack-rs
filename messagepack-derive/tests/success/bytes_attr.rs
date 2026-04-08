// Test: bytes attribute
use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_core::io::SliceReader;
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct BlobHolder {
    name: String,
    #[msgpack(bytes)]
    data: Vec<u8>,
}

fn main() {
    let b = BlobHolder {
        name: "test".to_string(),
        data: vec![1, 2, 3, 4, 5],
    };
    let mut buf = Vec::new();
    b.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <BlobHolder as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, b);
}
