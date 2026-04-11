## messagepack-core

[![Crates.io Version](https://img.shields.io/crates/v/messagepack_core)](https://crates.io/crates/messagepack-core)

messagepack for `no_std`

### Example

```rust
use messagepack_core::{Decode, Encode, io::{SliceWriter, SliceReader}};

#[derive(Debug,PartialEq,Encode,Decode)]
struct Data<'a> {
    compact: bool,
    schema: u8,
    less: &'a str,
}

let buf: &[u8] = &[
    0x83, 0xa7, 0x63, 0x6f, 0x6d, 0x70, 0x61, 0x63, 0x74, 0xc3, 0xa6, 0x73, 0x63, 0x68,
    0x65, 0x6d, 0x61, 0x00, 0xa4, 0x6c, 0x65, 0x73, 0x73, 0xa9, 0x74, 0x68, 0x61, 0x6e,
    0x20, 0x6a, 0x73, 0x6f, 0x6e,
];

let mut reader = SliceReader::new(&buf);
let data = Data::decode(&mut reader).unwrap();
let expected = Data {
    compact: true,
    schema: 0,
    less: "than json",
};
assert_eq!(data, expected);

let mut serialized = [0u8; 33];
let mut writer = SliceWriter::new(&mut serialized);
let len = expected.encode(&mut writer).unwrap();
assert_eq!(&serialized[..len], buf);
```

## Installation

Add this crate to `Cargo.toml`. If you want use this crate in `no_std`, disable default feature.

```toml
messagepack-core = { version = "0.2", default-features = false }
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/tunamaguro/messagepack-rs/blob/main/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://github.com/tunamaguro/messagepack-rs/blob/main/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
