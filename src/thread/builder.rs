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
use crate::thread::{ThreadId, THREAD_MAP};
use crate::util::get_last_windows_error;

pub struct Builder {
    name: Option<String>,
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
        self.name = Some(name);
        self
    }

    pub fn stack_size(mut self, size: usize) -> Builder {
        self.stack_size = size;
        self
    }

    pub fn spawn<F: FnOnce() -> T + Send + 'static, T: Send + 'static>(self, f: F) -> crate::io::Result<JoinHandle<T>> {
        let spawner: Box<ThreadSpawner<F, T>> = Box::new(
            ThreadSpawner {
                function: f,
                return_value: Arc::new(Mutex::new(None)),
                thread: Thread::new(
                    self.name,

                    // we will figure out the thread id soon
                    0
                )
            }
        );

        let join_handle_returned = JoinHandle {
            return_value: spawner.return_value.clone(),
            thread: spawner.thread.clone()
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

        Ok(join_handle_returned)
    }
}

struct ThreadSpawner<F: FnOnce() -> T + Send + 'static, T: Send + 'static> {
    function: F,
    return_value: Arc<spin::Mutex<Option<T>>>,
    thread: Thread
}

unsafe extern "system" fn do_spawn_thread<F: FnOnce() -> T + Send + 'static, T: Send + 'static>(t: *mut c_void) -> u32 {
    #[inline]
    fn add_thread_to_thread_map(thread_id: ThreadId, thread: Thread) {
        THREAD_MAP.lock().insert(thread_id.0, thread);
    }

    #[inline]
    fn remove_thread_from_thread_map(thread_id: ThreadId) {
        THREAD_MAP.lock().remove(&thread_id.0);
    }
    
    let spawner = unsafe { Box::from_raw(t as *mut ThreadSpawner<F, T>) };
    let thread_id = spawner.thread.id();
    
    add_thread_to_thread_map(thread_id, spawner.thread.clone());
    
    let mut t = spawner.return_value.lock();
    *t = Some((spawner.function)());
    
    // NOTE: We do not currently handle unwinds or thread termination. If the thread dies, this will
    //       stay in the map forever.
    remove_thread_from_thread_map(thread_id);

    0
}

pub fn spawn<F: FnOnce() -> T + Send + 'static, T: Send + 'static>(f: F) -> JoinHandle<T> {
    Builder::new().spawn(f).expect("error when trying to spawn a default thread")
}
