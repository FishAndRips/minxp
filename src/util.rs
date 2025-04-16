#![allow(dead_code)]

use windows_sys::Win32::Foundation::GetLastError;

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
