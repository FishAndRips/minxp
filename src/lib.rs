#![no_std]
#![allow(dead_code)]
#![allow(unused_variables)]

extern crate alloc;

#[macro_use]
mod print;
mod util;
pub mod env;
pub mod fs;
pub mod io;
pub mod path;
pub mod ffi;
pub mod process;

