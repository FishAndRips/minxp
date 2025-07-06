mod park_impl;
use park_impl::PARK_IMPL;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::Arc;
use core::cell::UnsafeCell;
use core::sync::atomic::AtomicBool;
use core::time::Duration;
use spin::Mutex;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Threading::{GetCurrentThread, GetCurrentThreadId, Sleep};

#[derive(Clone)]
pub struct Thread {
    name: Option<Arc<String>>,
    thread_id: Arc<UnsafeCell<u32>>,
    parked: Arc<AtomicBool>
}

impl Thread {
    pub(in crate::thread) fn new(
        name: Option<Arc<String>>,
        thread_id: u32
    ) -> Self {
        Self {
            name,
            thread_id: Arc::new(UnsafeCell::new(thread_id)),
            parked: Arc::new(AtomicBool::new(false))
        }
    }

    pub(in crate::thread) fn thread_id_ptr(&self) -> *mut u32 {
        self.thread_id.get()
    }

    pub(in crate::thread) fn set_thread_handle(&self, handle: HANDLE) -> *mut u32 {
        self.thread_id.get()
    }

    pub fn unpark(&self) {
        (PARK_IMPL.wake)(self)
    }

    pub fn id(&self) -> ThreadId {
        // SAFETY: By now, this shouldn't be written anymore.
        ThreadId(unsafe { *self.thread_id.get() })
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|n| n.as_str())
    }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct ThreadId(pub(in crate::thread) u32);

unsafe impl Send for Thread {}
unsafe impl Sync for Thread {}

pub fn sleep(duration: Duration) {
    unsafe {
        Sleep(duration.as_millis().min(u32::MAX.into()) as u32)
    }
}

pub fn yield_now() {
    unsafe {
        Sleep(0)
    }
}

pub fn park() {
    (PARK_IMPL.park)(None)
}

pub fn park_timeout(timeout: Duration) {
    (PARK_IMPL.park)(Some(timeout))
}

pub fn current() -> Thread {
    let mut threads = THREAD_MAP.lock();
    let current_thread = current_thread_id();

    match threads.get(&current_thread) {
        Some(n) => n.clone(),
        None => {
            let t = Thread::new(None, current_thread);
            threads.insert(current_thread, t.clone());
            t
        }
    }
}

pub(in crate::thread) fn current_thread_id() -> u32 {
    unsafe { GetCurrentThreadId() }
}

pub(in crate::thread) fn current_thread_handle() -> HANDLE {
    unsafe { GetCurrentThread() }
}

pub(in crate::thread) static THREAD_MAP: Mutex<BTreeMap<u32, Thread>> = Mutex::new(BTreeMap::new());
