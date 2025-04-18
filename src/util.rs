use alloc::vec::Vec;
use core::iter::once;
use windows_sys::Win32::Foundation::GetLastError;
use crate::ffi::OsStr;

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
