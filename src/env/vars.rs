use crate::util::{get_last_windows_error, strlen_w, utf16_ptr_to_slice};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};
use core::iter::once;
use core::ptr::null;
use windows_sys::core::PWSTR;
use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::System::Environment::{FreeEnvironmentStringsW, GetEnvironmentStringsW, GetEnvironmentVariableW, SetEnvironmentVariableW};
use crate::path::MAX_PATH_EXTENDED;

pub struct Vars {
    first_var: PWSTR,

    front_var: PWSTR,
    back_var: PWSTR
}

impl Vars {
    unsafe fn new(vars: PWSTR) -> Self {
        let error = get_last_windows_error();
        assert!(!vars.is_null(), "vars is NULL: {error}");

        let mut back_var = vars;
        loop {
            // SAFETY: vars is from GetEnvironmentStringsW
            let len = unsafe { strlen_w(back_var) };
            if len == 0 {
                break;
            }
            back_var = back_var.wrapping_add(len + 1);
        };

        Self {
            first_var: vars,
            front_var: vars,
            back_var
        }
    }
}

fn keyval(what: &[u16]) -> (String, String) {
    let mut string = String::from_utf16(what).expect("key=value not valid UTF-16");
    let (index, _) = string.char_indices().find(|c| c.1 == '=').expect("key=value no equals");

    let value = string[index + 1..].to_string();
    string.truncate(index);

    (string, value)
}

impl Iterator for Vars {
    type Item = (String, String);
    fn next(&mut self) -> Option<Self::Item> {
        // Check if we hit the last variable
        if unsafe { *self.front_var == 0 } {
            return None
        }

        let slice = unsafe {
            utf16_ptr_to_slice(self.front_var)
        };

        self.front_var = self.front_var.wrapping_add(slice.len() + 1);
        Some(keyval(slice))
    }
}

impl DoubleEndedIterator for Vars {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back_var == self.front_var {
            return None
        }

        self.back_var = self.back_var.wrapping_sub(1);
        while self.back_var > self.front_var {
            let previous_character = self.back_var.wrapping_sub(1);
            if unsafe { *previous_character == 0 } {
                break
            }
            self.back_var = previous_character;
        }

        let slice = unsafe {
            utf16_ptr_to_slice(self.back_var)
        };

        Some(keyval(slice))
    }
}

impl Drop for Vars {
    fn drop(&mut self) {
        unsafe {
            FreeEnvironmentStringsW(self.first_var);
        }
    }
}

pub fn vars() -> Vars {
    unsafe {
        Vars::new(GetEnvironmentStringsW())
    }
}

pub fn var<K: AsRef<str>>(key: K) -> Result<String, VarError> {
    let key = key.as_ref();

    let key_utf16: Vec<u16> = key.encode_utf16().chain(once(0)).collect();
    let mut value_utf16: Vec<u16> = Vec::with_capacity(MAX_PATH_EXTENDED + 1);

    unsafe {
        let len = GetEnvironmentVariableW(key_utf16.as_ptr(), value_utf16.as_mut_ptr(), value_utf16.capacity() as u32);
        if len == 0 {
            return Err(VarError::NotPresent)
        }

        // Safety: Capacity is MAX_ENV_VAR_VALUE_LEN+1, and GetEnvironmentVariableW() < this...
        value_utf16.set_len(len as usize);
    }

    Ok(String::from_utf16(&value_utf16).expect("environment variable not utf-16"))
}

pub fn set_var<K: AsRef<str>>(key: K, value: K) {
    let key = key.as_ref();
    assert!(!key.contains('='), "key cannot contain an equals sign");
    assert!(!key.contains('\x00'), "key cannot contain a NUL char");
    let value = value.as_ref();
    assert!(!value.contains('\x00'), "key cannot contain a NUL char");

    let value_utf16: Vec<u16> = value.encode_utf16().chain(once(0)).collect();
    assert!(value_utf16.len() < MAX_PATH_EXTENDED, "value exceeds MAX_ENV_VAR_VALUE_LEN codepoints");
    let key_utf16: Vec<u16> = key.encode_utf16().chain(once(0)).collect();

    let return_value = unsafe { SetEnvironmentVariableW(key_utf16.as_ptr(), value_utf16.as_ptr()) };
    let error = get_last_windows_error();
    assert_ne!(return_value, FALSE, "set_var failed: {error}");
}

pub fn remove_var<K: AsRef<str>>(key: K) {
    let key = key.as_ref();
    assert!(!key.contains('='), "key cannot contain an equals sign");
    assert!(!key.contains('\x00'), "key cannot contain a NUL char");
    let key_utf16: Vec<u16> = key.encode_utf16().chain(once(0)).collect();

    let return_value = unsafe { SetEnvironmentVariableW(key_utf16.as_ptr(), null()) };
    let error = get_last_windows_error();
    assert_ne!(return_value, FALSE, "remove_var failed: {error}");
}

#[derive(Clone, Debug, PartialEq)]
pub enum VarError {
    NotPresent
}

impl Display for VarError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}
