use messagepack_core::{Decode as _, Encode as _, io::SliceReader};
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[msgpack(array)]
struct Record {
    #[msgpack(key = 0)]
    name: String,
    #[msgpack(key = 1)]
    age: u8,
}

#[test]
fn array_mode() {
    let record = Record {
        name: "Alice".to_string(),
        age: 30,
    };

    let mut buf = Vec::new();
    let size = record.encode(&mut buf).unwrap();

    let expected = [
        0x92, // fixarray 2
        0xa5, b'A', b'l', b'i', b'c', b'e', // "Alice"
        0x1e, // 30
    ];
    assert_eq!(buf, expected);

    let mut reader = SliceReader::new(&buf[..size]);
    let decoded = Record::decode(&mut reader).unwrap();
    assert_eq!(decoded, record);
}
