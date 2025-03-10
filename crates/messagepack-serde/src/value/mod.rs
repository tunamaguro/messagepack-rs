#[cfg(feature = "alloc")]
pub(crate) mod _value;
#[cfg(feature = "alloc")]
pub use _value::ValueRef;

pub(crate) mod extension;
pub use extension::ExtensionRef;
pub(crate) mod number;
pub use number::Number;
