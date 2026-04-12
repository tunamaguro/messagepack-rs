// Test: bytes attribute
use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
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

fn assert_derive<'de, T: Encode + Decode<'de>>() {}

fn main() {
    assert_derive::<BlobHolder<'_>>();
}
