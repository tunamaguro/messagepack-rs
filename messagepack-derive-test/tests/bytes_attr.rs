use messagepack_core::{Decode as _, Encode as _, io::SliceReader};
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Record<'a> {
    /// Expect use `EncodeBytes` and `DecodeBytes` for this field
    #[msgpack(bytes)]
    bytes: &'a [u8],
}

#[test]
fn array_mode() {
    let record = Record {
        bytes: &[1, 2, 3, 4, 5],
    };

    let mut buf = Vec::new();
    let size = record.encode(&mut buf).unwrap();

    let expected = [
        0x81, // fixmap 1
        0xa5, b'b', b'y', b't', b'e', b's', // "bytes"
        0xc4, 0x05, // bin8 with length 5
        1, 2, 3, 4, 5,
    ];
    assert_eq!(buf, expected);

    let mut reader = SliceReader::new(&buf[..size]);
    let decoded = Record::decode(&mut reader).unwrap();
    assert_eq!(decoded, record);
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct OptionalBorrowedBytes<'a> {
    #[msgpack(bytes)]
    bytes: Option<&'a [u8]>,
}

#[test]
fn optional_borrowed_bytes_roundtrip() {
    let record = OptionalBorrowedBytes {
        bytes: Some(&[1, 2, 3, 4]),
    };

    let mut buf = Vec::new();
    record.encode(&mut buf).unwrap();

    let expected = [
        0x81, // fixmap 1
        0xa5, b'b', b'y', b't', b'e', b's', // "bytes"
        0xc4, 0x04, // bin8 with length 4
        1, 2, 3, 4,
    ];
    assert_eq!(buf, expected);

    let mut reader = SliceReader::new(&buf);
    let decoded = OptionalBorrowedBytes::decode(&mut reader).unwrap();
    assert_eq!(decoded, record);
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct OptionalOwnedBytes {
    #[msgpack(bytes)]
    bytes: Option<Vec<u8>>,
}

#[test]
fn optional_owned_bytes_handles_none_and_some() {
    let none = OptionalOwnedBytes { bytes: None };
    let mut none_buf = Vec::new();
    none.encode(&mut none_buf).unwrap();
    let expected_none = [
        0x81, // fixmap 1
        0xa5, b'b', b'y', b't', b'e', b's', // "bytes"
        0xc0, // nil
    ];
    assert_eq!(none_buf, expected_none);
    let mut none_reader = SliceReader::new(&none_buf);
    let decoded_none = OptionalOwnedBytes::decode(&mut none_reader).unwrap();
    assert_eq!(decoded_none, none);

    let some = OptionalOwnedBytes {
        bytes: Some(vec![9, 8, 7]),
    };
    let mut some_buf = Vec::new();
    some.encode(&mut some_buf).unwrap();
    let expected_some = [
        0x81, // fixmap 1
        0xa5, b'b', b'y', b't', b'e', b's', // "bytes"
        0xc4, 0x03, // bin8 with length 3
        9, 8, 7,
    ];
    assert_eq!(some_buf, expected_some);
    let mut some_reader = SliceReader::new(&some_buf);
    let decoded_some = OptionalOwnedBytes::decode(&mut some_reader).unwrap();
    assert_eq!(decoded_some, some);
}
