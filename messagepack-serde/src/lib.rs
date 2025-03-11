#![forbid(unsafe_code)]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod de;
pub mod ser;
mod value;

pub use de::{Deserializer, from_slice, from_slice_with_config};
pub use ser::{Serializer, to_slice, to_slice_with_config};

#[cfg(feature = "std")]
pub use de::{from_reader, from_reader_with_config};

#[cfg(feature = "std")]
pub use ser::{to_vec, to_vec_with_config, to_writer, to_writer_with_config};

#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub use value::ValueRef;
pub use value::{ExtensionRef, Number};
