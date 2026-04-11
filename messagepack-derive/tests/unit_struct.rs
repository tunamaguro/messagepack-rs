use messagepack_core::{Decode as _, Encode as _, io::SliceReader};
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Unit;

#[test]
fn round_trip_unit() {
    let unit = Unit;
    let mut buf = Vec::new();
    let size = unit.encode(&mut buf).unwrap();
    let expected = [0xc0]; // nil

    assert_eq!(buf, expected);

    let mut reader = SliceReader::new(&buf[..size]);
    let decoded = Unit::decode(&mut reader).unwrap();
    assert_eq!(decoded, unit);
}
