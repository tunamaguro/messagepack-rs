## messagepack-core

[![Crates.io Version](https://img.shields.io/crates/v/messagepack_core)](https://crates.io/crates/messagepack-core)

messagepack for `no_std`

### Example

```rust
use messagepack_core::{Decode, Encode, io::{SliceWriter, SliceReader}};

let mut buf = [0u8; 12];
let mut writer = SliceWriter::new(&mut buf);
let written = "MessagePack".encode(&mut writer).unwrap();

assert_eq!(
    buf,
    [
        0xab, 0x4d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x50, 0x61, 0x63, 0x6b
    ]
);
assert_eq!(written, 12);

let mut reader = SliceReader::new(&buf);
let decoded = <&str as Decode>::decode(&mut reader).unwrap();
assert_eq!(decoded, "MessagePack");
assert_eq!(reader.rest().len(), 0);
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
