# messagepack-rs

messagepack for `no_std` programs

[![workflow](https://github.com/tunamaguro/messagepack-rs/actions/workflows/pull_request.yaml/badge.svg)](https://github.com/tunamaguro/messagepack-rs/actions)
[![codecov](https://codecov.io/gh/tunamaguro/messagepack-rs/graph/badge.svg?token=1UJNSKR2C1)](https://codecov.io/gh/tunamaguro/messagepack-rs)
[![CodSpeed Badge](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/tunamaguro/messagepack-rs)

## Example

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

See [messagepack-serde](./crates/messagepack-serde/README.md) and [messagepack-core](./crates/messagepack-core/README.md)  for more information

## License

Licensed under either of

- Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.