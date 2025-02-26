#![forbid(unsafe_code)]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

#[doc = include_str!("../README.md")]
pub mod de;
pub mod ser;

pub use de::{Deserializer, Error as DeserializationError, from_slice};
pub use ser::{Error as SerializationError, Serializer, to_slice};

#[cfg(feature = "std")]
pub use de::from_reader;

#[cfg(feature = "std")]
pub use ser::to_writer;
