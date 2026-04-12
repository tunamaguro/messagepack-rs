use messagepack_core::{Decode as _, Encode as _, io::SliceReader};
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[msgpack(map)]
struct Record {
    name: String,
    age: u8,
}

#[test]
fn map_mode() {
    let record = Record {
        name: "Alice".to_string(),
        age: 30,
    };

    let mut buf = Vec::new();
    let size = record.encode(&mut buf).unwrap();

    let expected = [
        0x82, // fixmap 2
        0xa4, b'n', b'a', b'm', b'e', // "name"
        0xa5, b'A', b'l', b'i', b'c', b'e', // "Alice"
        0xa3, b'a', b'g', b'e', // "age"
        0x1e, // 30
    ];
    assert_eq!(buf, expected);

    let mut reader = SliceReader::new(&buf[..size]);
    let decoded = Record::decode(&mut reader).unwrap();
    assert_eq!(decoded, record);
}
