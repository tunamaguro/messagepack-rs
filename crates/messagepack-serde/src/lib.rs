#![forbid(unsafe_code)]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

#[doc = include_str!("../README.md")]
pub mod de;
pub mod ser;

pub use de::{Deserializer, Error as DeserializationError, from_slice, from_slice_with_config};
pub use ser::{Error as SerializationError, Serializer, to_slice};

#[cfg(feature = "std")]
pub use de::{from_reader, from_reader_with_config};

#[cfg(feature = "std")]
pub use ser::{to_vec, to_writer};
