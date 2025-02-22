#![forbid(unsafe_code)]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

pub mod decode;
pub mod encode;
mod formats;

pub use decode::Decode;
pub use encode::Encode;
pub use formats::Format;
