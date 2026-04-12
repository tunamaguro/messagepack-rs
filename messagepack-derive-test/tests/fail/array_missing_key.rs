// Test: array mode requires key on all fields
use messagepack_derive::{Encode, Decode};

#[derive(Encode, Decode)]
#[msgpack(array)]
struct Missing {
    name: String,
    age: u8,
}

fn main() {}
