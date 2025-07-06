use alloc::boxed::Box;
use alloc::sync::Arc;
use core::any::Any;
use spin::Mutex;
use crate::thread::thread::Thread;

pub struct JoinHandle<T: Send + 'static> {
    pub(in crate::thread) return_value: Arc<Mutex<Option<T>>>,
    pub(in crate::thread) thread: Thread
}

impl<T: Send + 'static> JoinHandle<T> {
    pub fn thread(&self) -> &Thread {
        &self.thread
    }

    /// Joins the thread, returning the result.
    ///
    /// As minxp does not support unwinding threads, this will always return `Ok()`.
    ///
    /// # Remarks
    ///
    /// If the thread was terminated (e.g. via `TerminateThread`), this will infinitely loop.
    pub fn join(self) -> Result<T> {
        loop {
            let Some(p) = self.return_value.lock().take() else {
                continue
            };
            return Ok(p)
        }
    }

    pub fn is_finished(&self) -> bool {
        self.return_value.try_lock().is_some_and(|t| t.is_some())
    }
}

pub type Result<T> = core::result::Result<T, Box<dyn Any + Send + 'static>>;
