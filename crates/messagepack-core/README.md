## messagepack-core

messagepack for `no_std`


### Example

 ```rust
 use messagepack_core::{Encode, Decode,decode::StrDecoder};

 let mut buf = [0u8;12];
 let written = "MessagePack".encode_to_iter_mut(&mut buf.iter_mut()).unwrap();

 assert_eq!(buf, [0xab,0x4d,0x65,0x73,0x73,0x61,0x67,0x65,0x50,0x61,0x63,0x6b]);
 assert_eq!(written, 12);

 let (decoded, rest) = StrDecoder::decode(&buf).unwrap();
 assert_eq!(decoded,"MessagePack");
 assert_eq!(rest.len(), 0);
  ```