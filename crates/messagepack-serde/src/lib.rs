#![forbid(unsafe_code)]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

pub mod de;
pub mod ser;

pub use de::{Deserializer, from_slice};
pub use ser::{Serializer, to_slice};
