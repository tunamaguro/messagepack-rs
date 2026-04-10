// Test: lifetime generics support
use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Wrapper<'a> {
    value: &'a str,
}

// expect expands to below
//
// impl<'a> Encode for Wrapper<'a> {
//     fn encode<__W: io::IoWrite>(
//         &self,
//         writer: &mut __W,
//     ) -> Result<usize, encode::Error<__W::Error>> {
//         todo!("some implementation")
//     }
// }
// impl<'__msgpack_de, 'a> Decode<'__msgpack_de> for Wrapper<'a>
// where
//     '__msgpack_de: 'a,
// {
//     type Value<'__reader>
//         = Wrapper<'a>
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
//         'a: '__reader,
//     {
//         todo!("some implementation")
//     }
// }

fn assert_derive<'de, T: Encode + Decode<'de>>() {}

fn main() {
    assert_derive::<Wrapper<'_>>();
}
