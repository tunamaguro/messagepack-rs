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

pub(crate) mod extension;
pub use extension::{ext_fixed, ext_owned, ext_ref, timestamp32, timestamp64, timestamp96};
mod number;
pub use number::Number;
