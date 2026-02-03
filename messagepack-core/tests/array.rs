use messagepack_core::{
    Decode, Encode,
    io::{SliceReader, VecRefWriter},
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_array(x in prop::collection::vec(any::<i64>(),0..1024)) {
        let mut buf = vec![];
        let mut writer = VecRefWriter::new(&mut buf);
        let written_length = x.encode(&mut writer).unwrap();

        let mut reader = SliceReader::new(&buf[..written_length]);
        let y = <Vec<i64>>::decode(&mut reader).unwrap();

        assert_eq!(x,y);
    }
}
