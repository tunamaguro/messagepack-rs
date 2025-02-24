#![forbid(unsafe_code)]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

#[doc = include_str!("../README.md")]
pub mod decode;
pub mod encode;
mod formats;

pub use decode::Decode;
pub use encode::Encode;
pub use formats::Format;
