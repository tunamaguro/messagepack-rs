// Test: basic named struct encode/decode as map (default)
use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_core::io::SliceReader;
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Point {
    x: u32,
    y: u32,
}

// expect expands to below
// 
// impl Encode for Point {
//     fn encode<__W: io::IoWrite>(
//         &self,
//         writer: &mut __W,
//     ) -> Result<usize, encode::Error<__W::Error>> {
//         todo!("some implementation")
//     }
// }
// impl<'__msgpack_de> Decode<'__msgpack_de> for Point {
//     type Value<'__reader>
//         = Point
//     where
//         Self: '__reader,
//         '__msgpack_de: '__reader;
//     fn decode_with_format<'__reader, __R>(
//         format: Format,
//         reader: &'__reader mut __R,
//     ) -> Result<Self::Value<'__reader>, decode::Error<__R::Error>>
//     where
//         __R: io::IoRead<'__msgpack_de>,
//         '__msgpack_de: '__reader,
//     {
//         todo!("some implementation")
//     }
// }

fn main() {
    let p = Point { x: 10, y: 20 };
    let mut buf = Vec::new();
    p.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <Point as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, p);
}
