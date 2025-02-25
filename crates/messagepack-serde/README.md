## messagepack-serde

messagepack for `no_std` with `serde`

## Examples

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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

let data = messagepack_serde::from_slice::<Data<'_>>(buf).unwrap();
let expected = Data {
    compact: true,
    schema: 0,
    less: "than json",
};
assert_eq!(data, expected);

let mut deserialized = [0u8; 33];
let len = messagepack_serde::to_slice(&expected, &mut deserialized).unwrap();
assert_eq!(&deserialized[..len], buf);
```

## `no_std` support

If you want this crate for `no_std`, please opt out default features.

```toml
serde = { version = "1.0", default-features = false, features = ["derive"] }
messagepack-serde = { git = "https://github.com/tunamaguro/messagepack-rs.git", default-features = false }
```