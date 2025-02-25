#![forbid(unsafe_code)]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

#[doc = include_str!("../README.md")]
pub mod decode;
pub mod encode;
mod formats;
pub mod io;

pub use decode::Decode;
pub use encode::Encode;
pub use formats::Format;
pub use io::{SliceReader, SliceWriter};
