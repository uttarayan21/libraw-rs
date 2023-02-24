#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(deref_nullptr)]
#![allow(clippy::useless_transmute)]
#![allow(clippy::approx_constant)]

pub use self::bindings::*;

#[cfg(all(windows, not(feature = "bindgen")))]
#[path = "windows.rs"]
mod bindings;

#[cfg(all(target_os = "macos", not(feature = "bindgen")))]
#[path = "macos.rs"]
mod bindings;

#[cfg(all(target_os = "linux", not(feature = "bindgen")))]
#[path = "linux.rs"]
mod bindings;

#[cfg(feature = "bindgen")]
mod bindings;
