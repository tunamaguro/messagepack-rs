use messagepack_core::{
    Decode, Encode,
    io::{SliceReader, SliceWriter},
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_str(x in any::<String>()) {
        let mut buf = vec![0; x.len() + 5];
        let mut writer = SliceWriter::new(&mut buf);
        let written_length = x.encode(&mut writer).unwrap();

        let mut reader = SliceReader::new(&buf[..written_length]);
        let y = <&str>::decode(&mut reader).unwrap();

        assert_eq!(x,y);
    }
}
