// Test: bytes attribute
use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_core::io::SliceReader;
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct BlobHolder<'a> {
    name: String,
    #[msgpack(bytes)]
    data_vec: Vec<u8>,
    #[msgpack(bytes)]
    data_array: [u8; 5],
    #[msgpack(bytes)]
    data_boxed: Box<[u8]>,
    #[msgpack(bytes)]
    data_ref: &'a [u8],
}

fn main() {
    let b = BlobHolder {
        name: "test".to_string(),
        data_vec: vec![1, 2, 3, 4, 5],
        data_array: [1, 2, 3, 4, 5],
        data_boxed: vec![1, 2, 3, 4, 5].into_boxed_slice(),
        data_ref: &[1, 2, 3, 4, 5],
    };
    let mut buf = Vec::new();
    b.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <BlobHolder as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, b);
}
