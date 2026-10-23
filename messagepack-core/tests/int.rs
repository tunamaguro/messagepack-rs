use messagepack_core::{
    Decode, Encode,
    io::{SliceReader, SliceWriter},
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_usize(x in any::<usize>()) {
        let mut buf = [0; 9];
        let mut writer = SliceWriter::new(&mut buf);
        let written_length = x.encode(&mut writer).unwrap();

        let mut reader = SliceReader::new(&buf[..written_length]);
        let y = usize::decode(&mut reader).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_isize(x in any::<isize>()) {
        let mut buf = [0; 9];
        let mut writer = SliceWriter::new(&mut buf);
        let written_length = x.encode(&mut writer).unwrap();

        let mut reader = SliceReader::new(&buf[..written_length]);
        let y = isize::decode(&mut reader).unwrap();

        assert_eq!(x, y);
    }
}
