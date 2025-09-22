//! Untyped value helpers for MessagePack.
//!
//! This module provides a dynamic representation of MessagePack data and
//! utility adapters for extension types.
//!
//! # Examples
//!
//! Serialize a struct into [`Value`] and then deserialize it back. Both
//! [`Value`] and [`ValueRef`] implement `serde::Deserializer`, so any
//! `T: serde::Deserialize` can be decoded from them.
//!
//! ```rust
//! # #[cfg(feature = "alloc")]
//! # fn main() {
//! use serde::{Deserialize, Serialize};
//! use messagepack_serde::value::{to_value, Value};
//!
//! #[derive(Debug, Serialize, Deserialize, PartialEq)]
//! struct User<'a> {
//!     id: u64,
//!     name: &'a str,
//!     active: bool,
//! }
//!
//! let original = User { id: 42, name: "alice", active: true };
//!
//! // Serialize Rust value to an owned MessagePack "Value" tree
//! let v: Value = to_value(&original).unwrap();
//!
//! // Deserialize back from &Value
//! let decoded = User::deserialize(&v).unwrap();
//! assert_eq!(decoded, original);
//! # }
//! # #[cfg(not(feature = "alloc"))]
//! # fn main() {}
//! ```
//!
//! Borrowed decoding from [`ValueRef`]. This avoids copying strings and
//! byte slices when possible.
//!
//! ```rust
//! # #[cfg(feature = "alloc")]
//! # fn main() {
//! use serde::Deserialize;
//! use messagepack_serde::value::ValueRef;
//!
//! // Borrowed primitives without allocation
//! let s = <&str>::deserialize(&ValueRef::String("hello")).unwrap();
//! assert_eq!(s, "hello");
//!
//! // Decode a tuple from a borrowed array
//! let v = ValueRef::Array(vec![
//!     ValueRef::from(1u64),
//!     ValueRef::from("hello"),
//!     ValueRef::from(false),
//! ]);
//! let tup = <(u64, &str, bool)>::deserialize(&v).unwrap();
//! assert_eq!(tup, (1, "hello", false));
//! # }
//! # #[cfg(not(feature = "alloc"))]
//! # fn main() {}
//! ```

#[cfg(feature = "alloc")]
mod value_ref;
#[cfg(feature = "alloc")]
pub use value_ref::ValueRef;

#[cfg(feature = "alloc")]
mod value_owned;
#[cfg(feature = "alloc")]
pub use value_owned::Value;

mod number;
pub use number::Number;

#[cfg(feature = "alloc")]
mod de;

#[cfg(feature = "alloc")]
mod ser;
#[cfg(feature = "alloc")]
pub use ser::to_value;

#[cfg(feature = "alloc")]
fn cautiously_size_hint<T>(hint: Option<usize>) -> usize {
    const MAX_ALLOC_BYTES: usize = 1024 * 1024;
    let element_byte: usize = core::mem::size_of::<T>();
    if element_byte == 0 {
        0
    } else {
        let max_count = MAX_ALLOC_BYTES / element_byte;
        hint.unwrap_or(0).min(max_count)
    }
}
