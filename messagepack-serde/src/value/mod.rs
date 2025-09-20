//! Dynamic value helpers for MessagePack.
//!
//! This module provides a dynamic representation of MessagePack data and
//! utility adapters for extension types.

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
