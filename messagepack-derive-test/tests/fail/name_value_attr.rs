use messagepack_derive::{Decode, Encode};

#[derive(Encode, Decode)]
struct S {
    #[msgpack = "bytes"]
    value: u8,
}

fn main() {}
