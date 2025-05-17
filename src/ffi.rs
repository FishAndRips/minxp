//! Implements OsString and OsStr.
//!
//! Does not implement [`core::ffi`] or [`alloc::ffi`] which has the rest of the FFI functions.

mod os_string;
pub use os_string::*;
