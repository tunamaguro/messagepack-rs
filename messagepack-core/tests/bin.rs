use messagepack_core::{
    Decode, Encode,
    encode::BinaryEncoder,
    io::{SliceReader, SliceWriter},
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_bin(x in any::<Vec<u8>>()) {
        let mut buf = vec![0; x.len() + 5];
        let mut writer = SliceWriter::new(&mut buf);
        let written_length = BinaryEncoder(x.as_slice()).encode(&mut writer).unwrap();

        let mut reader = SliceReader::new(&buf[..written_length]);
        let y = <&[u8]>::decode(&mut reader).unwrap();

        assert_eq!(x,y);
    }
}
