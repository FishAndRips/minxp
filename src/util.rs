use crate::ffi::OsStr;
use alloc::vec::Vec;
use core::iter::once;
use core::mem::zeroed;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};

/// Get the last error for the current thread.
pub fn get_last_windows_error() -> u32 {
    unsafe { GetLastError() }
}

/// Wraps the UTF16 pointer up to (but not including) a null terminator.
///
/// # Safety
///
/// Pointer `ptr` must be null-terminated and non-null.
pub unsafe fn utf16_ptr_to_slice(ptr: *const u16) -> &'static [u16] {
    debug_assert!(!ptr.is_null());
    unsafe { core::slice::from_raw_parts(ptr, strlen_w(ptr)) }
}

/// Wraps the UTF16 pointer up to (and including) a null terminator.
///
/// # Safety
///
/// Pointer `ptr` must be null-terminated and non-null.
pub unsafe fn utf16_ptr_to_slice_with_nul(ptr: *const u16) -> &'static [u16] {
    debug_assert!(!ptr.is_null());
    unsafe { core::slice::from_raw_parts(ptr, strlen_w(ptr) + 1) }
}

/// Count the number of characters at `ptr` until `0x0000` is reached.
///
/// # Safety
///
/// Pointer `ptr` must be null-terminated and non-null.
pub unsafe fn strlen_w(mut ptr: *const u16) -> usize {
    let mut size = 0usize;
    loop {
        // Safety: `ptr` is null-terminated
        if unsafe { *ptr == 0 } {
            return size;
        }
        size += 1;
        ptr = ptr.wrapping_add(1);
    }
}

fn encode_utf16_with_nul(string: &str) -> Vec<u16> {
    string.encode_utf16().chain(once(0)).collect()
}

pub trait AsUtf16Nul {
    /// Encode the string as UTF16, null terminated.
    fn encode_utf16_with_nul(&self) -> Vec<u16>;
}

impl<S: AsRef<OsStr>> AsUtf16Nul for S {
    fn encode_utf16_with_nul(&self) -> Vec<u16> {
        encode_utf16_with_nul(self.as_ref().as_str())
    }
}

macro_rules! get_proc_from_module {
    ($module:literal, $function:literal) => {{
        use windows_sys::Win32::System::LibraryLoader::{LoadLibraryW, GetProcAddress};
        use windows_sys::{s, w};

        // SAFETY: This is safe since we are just getting data. We aren't actually using the function
        //         we're trying to get.
        let module = unsafe {
            // should hopefully be called a minimal number of times
            LoadLibraryW(w!($module))
        };
        if !module.is_null() {
            let get_final_path_name_by_handle = unsafe { GetProcAddress(module, s!($function)) };
            if let Some(m) = get_final_path_name_by_handle {
                // SAFETY: Hopefully the caller knows what they're doing...
                unsafe { core::mem::transmute(m) }
            }
            else {
                None
            }
        }
        else {
            None
        }

    }};
}

pub(crate) use get_proc_from_module;

pub fn get_system_info() -> SYSTEM_INFO {
    let mut system_info = unsafe { zeroed::<SYSTEM_INFO>() };
    unsafe { GetSystemInfo(&mut system_info) };
    system_info
}
