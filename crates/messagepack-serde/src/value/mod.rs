#[cfg(feature = "alloc")]
pub(crate) mod value_;
#[cfg(feature = "alloc")]
pub use value_::ValueRef;

pub(crate) mod extension;
pub use extension::ExtensionRef;
pub(crate) mod number;
pub use number::Number;
