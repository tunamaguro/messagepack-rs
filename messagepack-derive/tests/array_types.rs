// use messagepack_core::decode::Decode;
// use messagepack_core::encode::Encode;
// use messagepack_core::io::SliceReader;
// use messagepack_derive::{Decode, Encode};

// #[derive(Debug, PartialEq, Encode, Decode)]
// struct Array {
//     data: Vec<u8>,
// }

// #[test]
// fn array_types() {
//     let a = Array {
//         data: vec![1, 2, 3],
//     };
//     let mut buf = Vec::new();
//     a.encode(&mut buf).unwrap();

//     let expected = [
//         0x81, // fixmap 1
//         0xa4, 0x64, 0x61, 0x74, 0x61, // "data"
//         0x93, 0x01, 0x02, 0x03, // fixarray 3
//     ];
//     assert_eq!(buf, expected);

//     let mut reader = SliceReader::new(&buf);
//     let decoded = <Array as Decode>::decode(&mut reader).unwrap();
//     assert_eq!(decoded, a);
// }
