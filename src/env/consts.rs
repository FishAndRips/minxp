//! Constants
//!
//! Most of these are derived from the Rust Standard Library values on Windows.
//!
//! [`ARCH`] is based on the `CARGO_CFG_TARGET_ARCH` environment value.

// the value of CARGO_CFG_TARGET_ARCH when run by the build script
pub const ARCH: &'static str = env!("MINXP_ENV_ARCH");

// Derived from the Rust Standard Library when built on x86_64-pc-windows-gnu on Windows 11; can be
// assumed to be true for all Windows versions.
pub const DLL_EXTENSION: &'static str = "dll";
pub const DLL_PREFIX: &'static str = "";
pub const DLL_SUFFIX: &'static str = ".dll";
pub const EXE_EXTENSION: &'static str = "exe";
pub const EXE_SUFFIX: &'static str = ".exe";
pub const FAMILY: &'static str = "windows";
pub const OS: &'static str = "windows";
