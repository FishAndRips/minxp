mod args;
mod vars;
mod paths;

use alloc::{format, vec};
use alloc::string::String;
use alloc::vec::Vec;
use core::iter::once;
use core::ptr::null_mut;
use windows_sys::Win32::System::Environment::{GetCurrentDirectoryW, SetCurrentDirectoryW};
use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW;
pub use args::*;
pub use vars::*;
pub use paths::*;
use crate::path::PathBuf;
use crate::io::Error;
use crate::util::get_last_windows_error;

pub fn current_exe() -> crate::io::Result<PathBuf> {
    let mut path = vec![0u16; 32768];
    let path_len = unsafe { GetModuleFileNameW(null_mut(), path.as_mut_ptr(), path.len() as u32) } as usize;

    assert_ne!(path_len, 0, "current_exe() failed: {}", get_last_windows_error());
    let data = String::from_utf16(&path[..path_len]).expect("GetModuleFileNameW returned non UTF-16");

    Ok(data.into())
}

pub fn current_dir() -> crate::io::Result<PathBuf> {
    let mut path = vec![0u16; 32768];
    let path_len = unsafe { GetCurrentDirectoryW(path.len() as u32, path.as_mut_ptr()) } as usize;

    assert_ne!(path_len, 0, "current_exe() failed: {}", get_last_windows_error());
    let data = String::from_utf16(&path[..path_len]).expect("GetCurrentDirectoryW returned non UTF-16");

    Ok(data.into())
}

pub fn set_current_dir<P: AsRef<str>>(path: P) -> crate::io::Result<()> {
    let path: Vec<u16> = path.as_ref().encode_utf16().chain(once(0)).collect();
    let success = unsafe { SetCurrentDirectoryW(path.as_ptr()) };
    let last_err = get_last_windows_error();
    match success {
        0 => Err(Error { reason: format!("Cannot set path: {last_err}") }),
        _ => Ok(())
    }
}

