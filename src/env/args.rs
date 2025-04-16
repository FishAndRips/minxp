use alloc::string::String;
use windows_sys::core::{PCWSTR, PWSTR};
use windows_sys::Win32::Foundation::LocalFree;
use windows_sys::Win32::System::Environment::GetCommandLineW;
use windows_sys::Win32::UI::Shell::CommandLineToArgvW;
use crate::util::{get_last_windows_error, utf16_ptr_to_slice};

pub struct Args {
    argv: *mut PWSTR,
    argc: usize,
    current_index_front: usize,
    current_index_back: usize
}

impl Args {
    unsafe fn new(command_line: PCWSTR) -> Self {
        assert!(!command_line.is_null(), "command_line is NULL: {}", get_last_windows_error());

        let mut argc: i32 = 0;

        // Safety: This string should be from GetCommandLineW
        let argv = unsafe { CommandLineToArgvW(command_line, &mut argc) };
        let argc = argc as usize;

        assert!(!argv.is_null(), "CommandLineToArgvW was NULL: {}", get_last_windows_error());

        Self {
            current_index_front: 0,
            current_index_back: argc,
            argc,
            argv
        }
    }
    fn get(&self, index: usize) -> String {
        assert!(index < self.argc);
        unsafe {
            let argument: *const u16 = *self.argv.wrapping_add(index);

            // Safety: This is from CommandLineToArgvW, and we're within `argc`
            String::from_utf16(utf16_ptr_to_slice(argument)).expect("non-utf16 string gotten")
        }
    }
}

impl Iterator for Args {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index_front < self.argc {
            let arg = self.get(self.current_index_front);
            self.current_index_front += 1;
            Some(arg)
        }
        else {
            None
        }
    }
}

impl DoubleEndedIterator for Args {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.current_index_back > 0 {
            self.current_index_back -= 1;
            let arg = self.get(self.current_index_back);
            Some(arg)
        }
        else {
            None
        }
    }
}

impl Drop for Args {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.argv as _);
        }
    }
}

pub fn args() -> Args {
    unsafe {
        Args::new(GetCommandLineW())
    }
}
