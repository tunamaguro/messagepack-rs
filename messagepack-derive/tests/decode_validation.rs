use messagepack_core::{
    decode::{Decode, Error},
    io::SliceReader,
};
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Point {
    x: u8,
    y: u8,
}

#[test]
fn named_struct_decode_ignores_unknown_keys() {
    let data = [
        0x83, // fixmap 3
        0xa1, b'x', 0x0a, // "x" => 10
        0xa1, b'z', 0x63, // "z" => 99
        0xa1, b'y', 0x14, // "y" => 20
    ];
    let mut reader = SliceReader::new(&data);
    let decoded = <Point as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, Point { x: 10, y: 20 });
}

#[test]
fn named_struct_decode_rejects_duplicate_keys() {
    let data = [
        0x83, // fixmap 3
        0xa1, b'x', 0x0a, // "x" => 10
        0xa1, b'x', 0x14, // duplicate "x" => 20
        0xa1, b'y', 0x1e, // "y" => 30
    ];
    let mut reader = SliceReader::new(&data);
    let err = <Point as Decode>::decode(&mut reader).unwrap_err();
    assert!(matches!(err, Error::InvalidData));
}

#[test]
fn named_struct_decode_rejects_missing_required_field() {
    let data = [
        0x81, // fixmap 1
        0xa1, b'x', 0x0a, // "x" => 10
    ];
    let mut reader = SliceReader::new(&data);
    let err = <Point as Decode>::decode(&mut reader).unwrap_err();
    assert!(matches!(err, Error::InvalidData));
}

#[test]
fn named_struct_decode_rejects_array_length_mismatch() {
    let data = [
        0x91, // fixarray 1
        0x0a, // 10
    ];
    let mut reader = SliceReader::new(&data);
    let err = <Point as Decode>::decode(&mut reader).unwrap_err();
    assert!(matches!(err, Error::InvalidData));
}

#[test]
fn named_struct_decode_rejects_non_map_or_array_format() {
    let data = [0xc0]; // nil
    let mut reader = SliceReader::new(&data);
    let err = <Point as Decode>::decode(&mut reader).unwrap_err();
    assert!(matches!(err, Error::UnexpectedFormat));
}
