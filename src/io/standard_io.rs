use crate::io::Write;
use alloc::sync::Arc;
use core::ptr::null_mut;
use spin::{Lazy, Mutex, MutexGuard};
use windows_sys::Win32::Foundation::{FALSE, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Globalization::CP_UTF8;
use windows_sys::Win32::Storage::FileSystem::{FlushFileBuffers, WriteFile};
use windows_sys::Win32::System::Console::{GetStdHandle, SetConsoleOutputCP, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE};

static STDOUT_HANDLE: Lazy<Stdout> = Lazy::new(|| {
    let handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
    let handle = (!handle.is_null() && handle != INVALID_HANDLE_VALUE).then(|| {
        unsafe { SetConsoleOutputCP(CP_UTF8) };
        handle
    });

    Stdout::new(Arc::new(Mutex::new(ConsoleOutSinkInner { handle })))
});

static STDERR_HANDLE: Lazy<Stderr> = Lazy::new(|| {
    let handle = unsafe { GetStdHandle(STD_ERROR_HANDLE) };
    let handle = (!handle.is_null() && handle != INVALID_HANDLE_VALUE).then(|| {
        unsafe { SetConsoleOutputCP(CP_UTF8) };
        handle
    });

    Stderr::new(Arc::new(Mutex::new(ConsoleOutSinkInner { handle })))
});

macro_rules! impl_stdout {
    ($type:tt, $type_lock:tt) => {
        pub struct $type {
            handle: Arc<Mutex<ConsoleOutSinkInner>>
        }

        impl $type {
            fn new(handle: Arc<Mutex<ConsoleOutSinkInner>>) -> Self {
                Self {
                    handle
                }
            }

            pub fn lock(&self) -> $type_lock<'static> {
                // SAFETY: stdout/stderr are singletons and will always exist
                unsafe { core::mem::transmute::<$type_lock, $type_lock>($type_lock { guard: self.handle.lock() }) }
            }
        }

        impl Write for $type {
            fn write(&mut self, buf: &[u8]) -> crate::io::Result<usize> {
                self.lock().write(buf)
            }
            fn write_all(&mut self, buf: &[u8]) -> crate::io::Result<()> {
                self.lock().write_all(buf)
            }
            fn flush(&mut self) -> crate::io::Result<()> {
                self.lock().flush()
            }
        }

        unsafe impl Sync for $type {}
        unsafe impl Send for $type {}

        pub struct $type_lock<'a> {
            guard: MutexGuard<'a, ConsoleOutSinkInner>
        }

        impl Write for $type_lock<'_> {
            fn write(&mut self, buf: &[u8]) -> crate::io::Result<usize> {
                self.guard.write(buf)
            }
            fn write_all(&mut self, buf: &[u8]) -> crate::io::Result<()> {
                self.guard.write_all(buf)
            }
            fn flush(&mut self) -> crate::io::Result<()> {
                self.guard.flush()
            }
        }
    };
}

impl_stdout!(Stdout, Stdoutlock);
impl_stdout!(Stderr, Stderrlock);

pub fn stdout() -> Stdout {
    let h = &*STDERR_HANDLE;
    Stdout { handle: h.handle.clone() }
}
pub fn stderr() -> Stderr {
    let h = &*STDERR_HANDLE;
    Stderr { handle: h.handle.clone() }
}

struct ConsoleOutSinkInner {
    handle: Option<HANDLE>
}

impl Write for ConsoleOutSinkInner {
    fn write(&mut self, buf: &[u8]) -> crate::io::Result<usize> {
        if let Some(n) = self.handle {
            let mut written_total = 0usize;
            for i in buf.chunks(u32::MAX as usize) {
                let mut written = 0u32;
                let success = unsafe {
                    WriteFile(
                        n,
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
        self.write(buf)?;
        Ok(())
    }
    fn flush(&mut self) -> crate::io::Result<()> {
        if let Some(n) = self.handle {
            unsafe { FlushFileBuffers(n) };
        }
        Ok(())
    }
}

