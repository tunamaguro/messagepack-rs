## messagepack-serde

[![workflow](https://github.com/tunamaguro/messagepack-rs/actions/workflows/pull_request.yaml/badge.svg)](https://github.com/tunamaguro/messagepack-rs/actions)
[![Crates.io Version](https://img.shields.io/crates/v/messagepack_serde)](https://crates.io/crates/messagepack-serde)
[![codecov](https://codecov.io/gh/tunamaguro/messagepack-rs/graph/badge.svg?token=1UJNSKR2C1)](https://codecov.io/gh/tunamaguro/messagepack-rs)
[![CodSpeed Badge](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json)](https://codspeed.io/tunamaguro/messagepack-rs)

MessagePack for `no_std` with `serde`.

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

Add this crate to `Cargo.toml`. `no_std` is supported by default.

```toml
messagepack-serde = { version = "0.1" }
```

## Features

- `no_std` support  
  If you want to use `std::io::Read` or `std::io::Write`, enable the `std` feature and use `messagepack_serde::from_reader` or `messagepack_serde::to_writer`.

- Flexible numeric serialization
  - Provides multiple encoding strategies:
    - `Exact`: Encodes numeric types exactly as provided.
    - `LosslessMinimize`: Minimizes size without losing information (default).
    - `AggressiveMinimize`: Aggressively minimizes values, including encoding floats with integral values as integers.
  - To deserialize arbitrary numeric values, use `messagepack_serde::value::Number`.

- `ext` format support

## Design Decisions

### Struct encoding format

This crate serializes Rust structs as MessagePack maps by default to preserve field names and allow flexible field ordering. Some other implementations (e.g., [rmp-serde](https://github.com/3Hren/msgpack-rust) and [MessagePack for C#](https://github.com/MessagePack-CSharp/MessagePack-CSharp)) serialize structs as arrays by default.

To maximize interoperability, the deserializer accepts both map- and array-encoded structs. When an array is encountered, fields are read in the declaration order of the Rust struct.

Example: decoding a struct from an array and a map

```rust
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
struct S {
    compact: bool,
    schema: u8,
}

// [true, 0] encoded as a MessagePack array of length 2
let buf: &[u8] = &[0x92, 0xc3, 0x00];
let s = messagepack_serde::from_slice::<S>(buf).unwrap();
assert_eq!(s, S { compact: true, schema: 0 });

// {"compact": true, "schema": 0} encoded as a MessagePack map
let buf: &[u8] = &[
    0x82, 0xa7, 0x63, 0x6f, 0x6d, 0x70, 0x61, 0x63, 0x74, 0xc3, 0xa6, 0x73, 0x63, 0x68,
    0x65, 0x6d, 0x61, 0x00
];
let s = messagepack_serde::from_slice::<S>(buf).unwrap();
assert_eq!(s, S { compact: true, schema: 0 });
```

A major advantage of serializing structs as arrays is reduced output size. Smaller output sizes positively impact processing speed. Additionally, since you no longer need to search for properties within strings during deserialization, you can expect faster deserialization times as well.
On the downside, this eliminates the self-describing nature of maps, making binary compatibility more fragile.

```rust
use serde::Deserialize;

// A future version of the struct
#[derive(Deserialize, Debug, PartialEq)]
struct FutureS {
    compact: bool,
    // Strongly depends on field order; adding this may break old data
    awesome: Option<bool>,
    schema: u8,
}

// Older payload: [true, 0]
let buf: &[u8] = &[0x92, 0xc3, 0x00];
let s = messagepack_serde::from_slice::<FutureS>(buf);
assert!(s.is_err()); // cannot decode the array

// Older payload: {"compact": true, "schema": 0}
let buf: &[u8] = &[
    0x82, 0xa7, 0x63, 0x6f, 0x6d, 0x70, 0x61, 0x63, 0x74, 0xc3, 0xa6, 0x73, 0x63, 0x68,
    0x65, 0x6d, 0x61, 0x00
];
let s = messagepack_serde::from_slice::<FutureS>(buf).unwrap();
assert_eq!(s, FutureS { compact: true, awesome: None, schema: 0 });
```

This crate prioritizes robustness and binary compatibility; therefore, it encodes structs as maps. If you need array encoding, consider representing your data as tuples or implementing a `Serialize` manually for your types.

```rust
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
struct S {
    compact: bool,
    schema: u8,
}

let val = S { compact: true, schema: 0 };
let mut buf = [0u8; 3];
let len = messagepack_serde::to_slice(&(&val.compact, &val.schema), &mut buf).unwrap();

// [true, 0] encoded as a MessagePack array of length 2
let expected: &[u8] = &[0x92, 0xc3, 0x00];
assert_eq!(buf, expected);
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/tunamaguro/messagepack-rs/blob/main/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://github.com/tunamaguro/messagepack-rs/blob/main/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.