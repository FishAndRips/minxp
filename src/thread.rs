mod builder;
mod join_handle;
mod thread;

use alloc::borrow::ToOwned;
pub use builder::*;
pub use join_handle::*;
pub use thread::*;

use crate::util::get_system_info;
use core::num::NonZeroUsize;
use crate::io::Error;

/// This returns the number of logical processors.
///
/// On all Windows versions, this just calls `GetSystemInfo()` and returns `dwNumberOfProcessors`.
///
/// Not all logical processors are created equally, as some CPU cores may run at different speeds
/// from others or have different capabilities.
///
/// In the case of SMT-enabled CPUs, some logical processors (i.e. threads) might share the same CPU
/// cores as others. Additionally, not all physical cores are guaranteed to support SMT, as some
/// CPUs may have a mix of multi and single thread cores.
///
/// # Remarks
///
/// Returns `Err` if `dwNumberOfProcessors` was somehow 0. This should generally never happen.
pub fn available_parallelism() -> crate::io::Result<NonZeroUsize> {
    NonZeroUsize::new(get_system_info().dwNumberOfProcessors as usize)
        .ok_or_else(|| Error { reason: "Unknown number of processor threads".to_owned() })
}
