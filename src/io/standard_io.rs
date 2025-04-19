use alloc::sync::Arc;
use core::ptr::null;
use spin::{Lazy, Mutex};
use windows_sys::Win32::Foundation::{FALSE, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Globalization::CP_UTF8;
use windows_sys::Win32::Storage::FileSystem::FlushFileBuffers;
use windows_sys::Win32::System::Console::{GetStdHandle, SetConsoleCP, SetConsoleOutputCP, WriteConsoleA, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE};
use crate::io::Write;

fn enable_utf8() {
    // for input
    unsafe { SetConsoleCP(CP_UTF8) };

    // for output
    unsafe { SetConsoleOutputCP(CP_UTF8) };
}

static STDOUT_HANDLE: Lazy<Stdout> = Lazy::new(|| {
    let handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
    Stdout::new(Arc::new(Mutex::new((!handle.is_null() && handle != INVALID_HANDLE_VALUE).then_some(handle))))
});

static STDERR_HANDLE: Lazy<Stderr> = Lazy::new(|| {
    let handle = unsafe { GetStdHandle(STD_ERROR_HANDLE) };
    Stderr::new(Arc::new(Mutex::new((!handle.is_null() && handle != INVALID_HANDLE_VALUE).then_some(handle))))
});

macro_rules! impl_stdout {
    ($type:tt) => {
        pub struct $type {
            handle: Arc<Mutex<Option<HANDLE>>>
        }

        impl $type {
            fn new(handle: Arc<Mutex<Option<HANDLE>>>) -> Self {
                Self {
                    handle
                }
            }
        }

        impl Write for $type {
            fn write(&mut self, buf: &[u8]) -> crate::io::Result<usize> {
                let handle = self.handle.lock();
                if let Some(n) = Option::as_ref(&handle) {
                    enable_utf8();

                    let mut written_total = 0usize;
                    for i in buf.chunks(u32::MAX as usize) {
                        let mut written = 0u32;
                        let success = unsafe {
                            WriteConsoleA(
                                *n,
                                i.as_ptr(),
                                i.len() as u32,
                                &mut written,
                                null()
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
                    Ok(buf.len())
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
    Stdout { handle: STDOUT_HANDLE.handle.clone() }
}
pub fn stderr() -> Stderr {
    Stderr { handle: STDERR_HANDLE.handle.clone() }
}

