mod args;
mod vars;
mod path;
pub mod consts;

use crate::io::Error;
use crate::path::{PathBuf, MAX_PATH_EXTENDED};
use crate::util::{get_last_windows_error, get_proc_from_module};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::{format, vec};
use core::iter::once;
use core::ptr::null_mut;
use spin::Lazy;
use windows_sys::Win32::Foundation::{CloseHandle, FALSE};
use windows_sys::Win32::Security::TOKEN_QUERY;
use windows_sys::Win32::Storage::FileSystem::GetTempPathW;
use windows_sys::Win32::System::Environment::{GetCurrentDirectoryW, SetCurrentDirectoryW};
use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows_sys::Win32::UI::Shell::GetUserProfileDirectoryW;
pub use args::*;
pub use vars::*;
pub use path::*;

pub fn current_exe() -> crate::io::Result<PathBuf> {
    let mut path = vec![0u16; MAX_PATH_EXTENDED + 1];
    let path_len = unsafe { GetModuleFileNameW(null_mut(), path.as_mut_ptr(), path.len() as u32) } as usize;
    let err = get_last_windows_error();
    assert_ne!(path_len, 0, "current_exe() failed: {err}");
    let data = String::from_utf16(&path[..path_len]).expect("GetModuleFileNameW returned non UTF-16");

    Ok(data.into())
}

pub fn current_dir() -> crate::io::Result<PathBuf> {
    let mut path = vec![0u16; MAX_PATH_EXTENDED + 1];
    let path_len = unsafe { GetCurrentDirectoryW(path.len() as u32, path.as_mut_ptr()) } as usize;
    let err = get_last_windows_error();
    assert_ne!(path_len, 0, "current_exe() failed: {err}");
    let data = String::from_utf16(&path[..path_len]).expect("GetCurrentDirectoryW returned non UTF-16");

    Ok(data.into())
}

pub fn home_dir() -> Option<PathBuf> {
    if let Ok(v) = var("USERPROFILE") {
        return Some(v.into())
    };

    let mut data = vec![0u16; MAX_PATH_EXTENDED + 1];
    let handle = unsafe { GetCurrentProcess() };
    let mut token_handle = null_mut();
    let token = unsafe { OpenProcessToken(handle, TOKEN_QUERY, &mut token_handle) };

    if token == FALSE {
        return None
    }

    unsafe {
        let mut len = data.len() as u32;
        let success = GetUserProfileDirectoryW(token_handle, data.as_mut_ptr(), &mut len);
        let err = get_last_windows_error();
        CloseHandle(token_handle);
        if success == FALSE {
            return None
        }
    }

    let null = data.iter().position(|s| *s == 0).unwrap_or(data.len());
    String::from_utf16(&data[..null]).ok().map(Into::into)
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

type GetTempPathWFn = unsafe extern "system" fn (len: u32, *mut u16) -> u32;

static GET_TEMP_PATH: Lazy<GetTempPathWFn> = Lazy::new(|| {
    let get_temp_path: Option<GetTempPathWFn> = get_proc_from_module!(
        "kernel32.dll",
        "GetTempPath2W"
    );

    match get_temp_path {
        Some(t) => t,
        None => GetTempPathW
    }
});

/// Get a temp directory.
///
/// # Compatibility notes
///
/// - On Windows 11 or newer, this uses [`GetTempPath2W`]
/// - On any edition of Windows 10 that received the March 2025 monthly rollup, this also uses [`GetTempPath2W`]
/// - On other versions of Windows, this uses [`GetTempPathW`]
///
/// The difference between `GetTempPath2` and `GetTempPath` is that `GetTempPath2` supports storing
/// temporary files in a `SystemTemp` directory when running as a SYSTEM process. Otherwise, it will
/// return the same value as `GetTempPath`.
///
/// [`GetTempPathW`]: https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-gettemppathw
/// [`GetTempPath2W`]: https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-gettemppath2w
pub fn temp_dir() -> PathBuf {
    let mut buffer = vec![0u16; MAX_PATH_EXTENDED + 1];
    let length = unsafe { GET_TEMP_PATH(buffer.len() as u32, buffer.as_mut_ptr()) };
    buffer.truncate(length as usize);

    let data = String::from_utf16(&buffer[..length as usize]).expect("GetTempPath(2)W returned non UTF-16");
    data.into()
}
