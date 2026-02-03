use messagepack_core::{
    Decode, Encode,
    io::{SliceReader, VecRefWriter},
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_map(x in prop::collection::btree_map(any::<i64>(), any::<usize>(), 0..1024)) {
        let mut buf = vec![];
        let mut writer = VecRefWriter::new(&mut buf);
        let written_length = x.encode(&mut writer).unwrap();

        let mut reader = SliceReader::new(&buf[..written_length]);
        let y = <std::collections::BTreeMap<i64,usize>>::decode(&mut reader).unwrap();

        assert_eq!(x,y);
    }
}
