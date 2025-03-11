## messagepack-serde

[![workflow](https://github.com/tunamaguro/messagepack-rs/actions/workflows/pull_request.yaml/badge.svg)](https://github.com/tunamaguro/messagepack-rs/actions)
[![Crates.io Version](https://img.shields.io/crates/v/messagepack_serde)](https://crates.io/crates/messagepack-serde)
[![codecov](https://codecov.io/gh/tunamaguro/messagepack-rs/graph/badge.svg?token=1UJNSKR2C1)](https://codecov.io/gh/tunamaguro/messagepack-rs)
[![CodSpeed Badge](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/tunamaguro/messagepack-rs)

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

let mut serialized = [0u8; 33];
let len = messagepack_serde::to_slice(&expected, &mut serialized).unwrap();
assert_eq!(&serialized[..len], buf);
```

## Installation

Add this crate for `Cargo.toml`. Default support `no_std`.

```toml
messagepack-serde = { version = "0.1" }
```

## Features

- `no_std` support

    If you want this crate with `std::io::Read` or `std::io::Write`, please add feature `std` and use `messagepack_serde::from_reader` or `messagepack_serde::to_writer`.

- Flexible Numeric Serialization
    - Provides multiple numeric encoding strategies:
        - `Exact`: Encodes numeric types exactly as provided without minimization. This is default.
        - `Lossless Minimization`: Minimizes numeric type size during serialization without any loss of information (e.g., encoding 1_u16 as positive fixint).
        - `Aggressive Minimization`: Aggressively minimizes numeric values, including converting floats with integer values into integers for the most compact representation.
    - If you want deserialize any numeric value, please use `messagepack_serde::value::Number`.

- `ext` format support

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/tunamaguro/messagepack-rs/blob/main/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://github.com/tunamaguro/messagepack-rs/blob/main/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.