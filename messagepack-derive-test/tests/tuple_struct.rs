use messagepack_core::{Decode as _, Encode as _, io::SliceReader};
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Pair(u8, u16);

fn main() {
    let pair = Pair(42, 65535);

    let mut buf = Vec::new();
    let size = pair.encode(&mut buf).unwrap();

    let expected = [
        0x92, // fixarray 2
        0x2a, // 42
        0xff, 0xff, // 65535
    ];
    assert_eq!(buf, expected);

    let mut reader = SliceReader::new(&buf[..size]);
    let decoded = Pair::decode(&mut reader).unwrap();
    assert_eq!(pair, decoded);
}
