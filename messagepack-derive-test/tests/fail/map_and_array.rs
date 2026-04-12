// Test: map and array are mutually exclusive
use messagepack_derive::{Encode, Decode};

#[derive(Encode, Decode)]
#[msgpack(map, array)]
struct Both {
    x: u32,
}

fn main() {}
