#![forbid(unsafe_code)]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod de;
pub mod ser;
mod value;

pub use de::from_slice;
pub use ser::{to_slice, to_slice_with_config};

#[cfg(feature = "std")]
pub use de::from_reader;

#[cfg(feature = "alloc")]
pub use ser::to_vec;

#[cfg(feature = "std")]
pub use ser::{to_writer, to_writer_with_config};

pub use value::Number;
#[cfg(feature = "alloc")]
pub use value::ValueRef;
pub use value::{ext_fixed, ext_ref};
