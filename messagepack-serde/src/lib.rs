#![forbid(unsafe_code)]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

pub use messagepack_core;

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod de;
pub use de::from_slice;
#[cfg(feature = "std")]
pub use de::from_reader;

pub mod ser;
pub use ser::{to_slice, to_slice_with_config};
#[cfg(feature = "alloc")]
pub use ser::to_vec;
#[cfg(feature = "std")]
pub use ser::{to_writer, to_writer_with_config};

pub mod value;
#[cfg(feature = "alloc")]
pub use value::{Value, ValueRef, to_value};

pub mod extension;
