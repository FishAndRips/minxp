use windows_sys::Win32::System::Threading::{ExitProcess, GetCurrentProcess, TerminateProcess};

/// Aborts the process
///
/// Internally, this calls `TerminateProcess(GetCurrentProcess(), 197)`.
pub fn abort() -> ! {
    unsafe {
        // TerminateProcess SHOULD abort the process instantly without returning, since we are
        // terminating our own process, though TerminateProcess does not return `!`, so we need to
        // do something that DOES result in that
        TerminateProcess(GetCurrentProcess(), 197);
        
        // to do this, we also add a call to ExitProcess.
        ExitProcess(196); // should not reach this far
    }
}

pub fn exit(exit_code: i32) -> ! {
    unsafe {
        ExitProcess(exit_code.cast_unsigned())
    }
}
