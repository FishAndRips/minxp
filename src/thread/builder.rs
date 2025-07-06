use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use core::cell::UnsafeCell;
use core::ffi::c_void;
use core::ptr::null;
use spin::mutex::Mutex;
use windows_sys::Win32::System::Threading::CreateThread;
use crate::io::Error;
use crate::thread::join_handle::JoinHandle;
use crate::thread::thread::Thread;
use crate::thread::THREAD_MAP;
use crate::util::get_last_windows_error;

pub struct Builder {
    name: Option<Arc<String>>,
    stack_size: usize,

    thread_id: UnsafeCell<u32>
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            name: None,
            stack_size: 0,
            thread_id: UnsafeCell::new(0)
        }
    }

    pub fn name(mut self, name: String) -> Builder {
        self.name = Some(Arc::new(name));
        self
    }

    pub fn stack_size(mut self, size: usize) -> Builder {
        self.stack_size = size;
        self
    }

    pub fn spawn<F: FnOnce() -> T + Send + 'static, T: Send + 'static>(self, f: F) -> crate::io::Result<JoinHandle<T>> {
        let mut threads = THREAD_MAP.lock();

        let spawner: Box<ThreadSpawner<F, T>> = Box::new(
            ThreadSpawner {
                function: f,
                return_value: Arc::new(Mutex::new(None)),
            }
        );

        let thread = Thread::new(
            self.name,

            // we will figure out the thread id soon
            0
        );

        let join_handle_returned = JoinHandle {
            return_value: spawner.return_value.clone(),
            thread: thread.clone()
        };

        let spawner_raw = Box::into_raw(spawner);

        let return_value = unsafe {
            CreateThread(
                null(), // default
                self.stack_size,
                Some(do_spawn_thread::<F, T>),
                spawner_raw as *const _,
                0, // default
                join_handle_returned.thread.thread_id_ptr()
            )
        };

        if return_value.is_null() {
            let reason = get_last_windows_error();
            let _ = unsafe { Box::from_raw(spawner_raw) };
            return Err(Error { reason: format!("Failed to spawn a thread: windows error 0x{reason:08X}") });
        }

        threads.insert(thread.id().0, thread);

        Ok(join_handle_returned)
    }
}

struct ThreadSpawner<F: FnOnce() -> T + Send + 'static, T: Send + 'static> {
    function: F,
    return_value: Arc<spin::Mutex<Option<T>>>
}

unsafe extern "system" fn do_spawn_thread<F: FnOnce() -> T + Send + 'static, T: Send + 'static>(t: *mut c_void) -> u32 {
    let spawner = unsafe { Box::from_raw(t as *mut ThreadSpawner<F, T>) };

    let mut t = spawner.return_value.lock();
    let return_value = (spawner.function)();
    *t = Some(return_value);

    0
}

pub fn spawn<F: FnOnce() -> T + Send + 'static, T: Send + 'static>(f: F) -> JoinHandle<T> {
    Builder::new().spawn(f).expect("error when trying to spawn a default thread")
}
