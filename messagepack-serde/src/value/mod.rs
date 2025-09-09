#[cfg(feature = "alloc")]
pub(crate) mod value_;
#[cfg(feature = "alloc")]
pub use value_::ValueRef;

#[cfg(feature = "alloc")]
pub(crate) mod value_owned;
#[cfg(feature = "alloc")]
pub use value_owned::Value;

pub(crate) mod extension;
pub use extension::{ext_fixed, ext_ref};
pub(crate) mod number;
pub use number::Number;
