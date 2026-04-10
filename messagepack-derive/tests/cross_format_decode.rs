// // Test: decode accepts both map and array formats for named structs
// use messagepack_core::decode::Decode;
// use messagepack_core::encode::Encode;
// use messagepack_core::io::SliceReader;
// use messagepack_derive::{Decode, Encode};

// #[derive(Debug, PartialEq, Encode, Decode)]
// struct Point {
//     x: u8,
//     y: u8,
// }

// #[test]
// fn cross_format_decode() {
//     // Encode as map (default) and decode
//     let p = Point { x: 10, y: 20 };
//     let mut buf = Vec::new();
//     p.encode(&mut buf).unwrap();
//     let mut reader = SliceReader::new(&buf);
//     let decoded = <Point as Decode>::decode(&mut reader).unwrap();
//     assert_eq!(decoded, p);

//     // Manually construct array-encoded version: fixarray(2), 10, 20
//     let array_buf = [0x92u8, 0x0a, 0x14];
//     let mut reader = SliceReader::new(&array_buf);
//     let decoded = <Point as Decode>::decode(&mut reader).unwrap();
//     assert_eq!(decoded, Point { x: 10, y: 20 });
// }
