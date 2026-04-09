#![deny(missing_docs)]


//! Derive marco for [messagepack_core::encode::Encode] and [messagepack_core::decode::Decode]

mod attrs;
mod decode;
mod encode;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Derive the `Encode` trait for a struct.
///
/// # Supported types
/// - **Named-field structs** — encoded as a MessagePack map by default
/// - **Tuple structs** — encoded as a MessagePack array
/// - **Unit structs** — encoded as MessagePack `nil`
///
/// # Container attributes
/// - `#[msgpack(map)]` — encode as a MessagePack map (default for named-field structs)
/// - `#[msgpack(array)]` — encode as a MessagePack array
///
/// # Field attributes
/// - `#[msgpack(key = N)]` — required for all fields in `array` mode
/// - `#[msgpack(bytes)]` — encode the field as MessagePack binary
/// - `#[msgpack(encode_with = "path::to::fn")]` — custom encode function
#[proc_macro_derive(Encode, attributes(msgpack))]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    encode::derive_encode(&input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Derive the `DecodeBorrowed` trait for a struct.
///
/// Named-field structs accept both map and array MessagePack formats on
/// decode regardless of the `map`/`array` attribute.
///
/// # Supported types
/// - **Named-field structs** — decoded from a MessagePack map or array
/// - **Tuple structs** — decoded from a MessagePack array
/// - **Unit structs** — decoded from MessagePack `nil`
///
/// # Container attributes
/// - `#[msgpack(map)]` — default for named-field structs
/// - `#[msgpack(array)]` — encode mode; requires `key` on every field
///
/// # Field attributes
/// - `#[msgpack(key = N)]` — array index (required in `array` mode)
/// - `#[msgpack(bytes)]` — decode the field from MessagePack binary
/// - `#[msgpack(decode_with = "path::to::fn")]` — custom decode function
#[proc_macro_derive(Decode, attributes(msgpack))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    decode::derive_decode(&input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
