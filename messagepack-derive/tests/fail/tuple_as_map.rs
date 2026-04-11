use messagepack_derive::{Decode, Encode};

#[derive(Encode, Decode)]
#[msgpack(map)]
struct TupleAsMap(String, u8);

fn main() {}
