#![forbid(unsafe_code)]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[doc = include_str!("../README.md")]
pub mod de;
pub mod ser;
pub mod value;

pub use de::{Deserializer, Error as DeserializationError, from_slice, from_slice_with_config};
pub use ser::{Error as SerializationError, Serializer, to_slice, to_slice_with_config};

#[cfg(feature = "std")]
pub use de::{from_reader, from_reader_with_config};

#[cfg(feature = "std")]
pub use ser::{to_vec, to_vec_with_config, to_writer, to_writer_with_config};
