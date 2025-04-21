use crate::io::Write;
use alloc::sync::Arc;
use core::ptr::null_mut;
use spin::{Lazy, Mutex};
use windows_sys::Win32::Foundation::{FALSE, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Globalization::CP_UTF8;
use windows_sys::Win32::Storage::FileSystem::{FlushFileBuffers, WriteFile};
use windows_sys::Win32::System::Console::{GetConsoleMode, GetStdHandle, SetConsoleCP, SetConsoleOutputCP, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE};

fn enable_utf8() {
    // for input
    unsafe { SetConsoleCP(CP_UTF8) };

    // for output
    unsafe { SetConsoleOutputCP(CP_UTF8) };
}

static STDOUT_HANDLE: Lazy<Stdout> = Lazy::new(|| {
    let handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
    let mut is_console = false;
    let handle = Arc::new(Mutex::new((!handle.is_null() && handle != INVALID_HANDLE_VALUE).then(|| {
        enable_utf8();
        is_console = unsafe { GetConsoleMode(handle, &mut 0) != FALSE };
        handle
    })));

    Stdout::new(handle, is_console)
});

static STDERR_HANDLE: Lazy<Stderr> = Lazy::new(|| {
    let handle = unsafe { GetStdHandle(STD_ERROR_HANDLE) };
    let mut is_console = false;
    let handle = Arc::new(Mutex::new((!handle.is_null() && handle != INVALID_HANDLE_VALUE).then(|| {
        enable_utf8();
        is_console = unsafe { GetConsoleMode(handle, &mut 0) != FALSE };
        handle
    })));

    Stderr::new(handle, is_console)
});

macro_rules! impl_stdout {
    ($type:tt) => {
        pub struct $type {
            handle: Arc<Mutex<Option<HANDLE>>>,
            is_console: bool
        }

        impl $type {
            fn new(handle: Arc<Mutex<Option<HANDLE>>>, is_console: bool) -> Self {
                Self {
                    handle, is_console
                }
            }
        }

        impl Write for $type {
            fn write(&mut self, buf: &[u8]) -> crate::io::Result<usize> {
                let handle = self.handle.lock();
                if let Some(n) = Option::as_ref(&handle) {
                    let mut written_total = 0usize;
                    for i in buf.chunks(u32::MAX as usize) {
                        let mut written = 0u32;
                        let success = unsafe {
                            WriteFile(
                                *n,
                                i.as_ptr(),
                                i.len() as u32,
                                &mut written,
                                null_mut()
                            )
                        };
                        if success == FALSE {
                            break;
                        }
                        else {
                            written_total += written as usize;
                        }
                    }
                    Ok(written_total)
                }
                else {
                    Ok(0)
                }
            }
            fn write_all(&mut self, buf: &[u8]) -> crate::io::Result<()> {
                let _ = self.write(buf);
                Ok(())
            }
            fn flush(&mut self) -> crate::io::Result<()> {
                let lock = self.handle.lock();
                if let Some(n) = Option::as_ref(&lock.as_ref()) {
                    unsafe { FlushFileBuffers(**n) };
                }
                Ok(())
            }
        }

        impl core::fmt::Write for $type {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                let _ = crate::io::Write::write_all(self, s.as_bytes());
                Ok(())
            }
        }

        unsafe impl Sync for $type {}
        unsafe impl Send for $type {}
    };
}

fn asdf() {
    let stdout = stdout();
    Option::as_ref(&stdout.handle.lock());
}

impl_stdout!(Stdout);
impl_stdout!(Stderr);

pub fn stdout() -> Stdout {
    let h = &*STDERR_HANDLE;
    Stdout { handle: h.handle.clone(), is_console: h.is_console }
}
pub fn stderr() -> Stderr {
    let h = &*STDERR_HANDLE;
    Stderr { handle: h.handle.clone(), is_console: h.is_console }
}

