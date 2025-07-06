use alloc::boxed::Box;
use core::ffi::c_void;
use core::sync::atomic::Ordering;
use core::time::Duration;
use spin::Lazy;
use windows_sys::Win32::Foundation::BOOL;
use windows_sys::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
use crate::thread::{current, Thread};
use crate::util::get_proc_from_module;

pub struct ParkImpl {
    pub park: Box<dyn Fn(Option<Duration>) + Send + Sync>,
    pub wake: Box<dyn Fn(&Thread) + Send + Sync>
}

pub static PARK_IMPL: Lazy<ParkImpl> = Lazy::new(|| {
    let wait_on_address: Option<
        unsafe extern "system" fn(address: *const c_void, compareaddress: *const c_void, addresssize: usize, dwmilliseconds: u32) -> BOOL
    > = get_proc_from_module!(
        "api-ms-win-core-synch-l1-2-0.dll",
        "WaitOnAddress"
    );

    let wake_by_address_all: Option<
        unsafe extern "system" fn(address: *const c_void)
    > = get_proc_from_module!(
        "api-ms-win-core-synch-l1-2-0.dll",
        "WakeByAddressAll"
    );

    let fns = wait_on_address.and_then(|w| Some((w, wake_by_address_all?)));

    if let Some((wait_on_address, wake_by_address_all)) = fns {
        get_wait_on_address_park_impl(wait_on_address, wake_by_address_all)
    }
    else {
        get_busy_wait_park_impl()
    }
});

fn get_wait_on_address_park_impl(
    wait_on_address: unsafe extern "system" fn(address: *const c_void, compareaddress: *const c_void, addresssize: usize, dwmilliseconds: u32) -> BOOL,
    wake_by_address_all: unsafe extern "system" fn(address: *const c_void),
) -> ParkImpl {
    ParkImpl {
        park: Box::new(move |duration_maybe: Option<Duration>| {
            let parked = current().parked.clone();
            let original_value = true;
            parked.store(original_value, Ordering::Relaxed);

            let ms = duration_maybe
                .map(|m| m.as_millis().min(u32::MAX.into()) as u32)
                .unwrap_or(0);

            unsafe {
                wait_on_address(
                    parked.as_ptr() as *const c_void,
                    &original_value as *const bool as *const c_void,
                    size_of_val(&original_value),
                    ms
                )
            };

            parked.store(false, Ordering::Relaxed);
        }),
        wake: Box::new(move |thread: &Thread| {
            thread.parked.store(false, Ordering::Relaxed);
            unsafe { wake_by_address_all(thread.parked.as_ptr() as *mut c_void) };
        })
    }
}

fn get_busy_wait_park_impl() -> ParkImpl {
    ParkImpl {
        park: Box::new(|duration_maybe: Option<Duration>| {
            let parked = current().parked.clone();
            parked.store(true, Ordering::Relaxed);

            match duration_maybe {
                Some(timeout) => {
                    let mut counter = 0i64;
                    let mut start = 0i64;
                    assert_ne!(unsafe { QueryPerformanceFrequency(&mut counter) }, 0, "QueryPerformanceCounter not supported"); // should never happen
                    assert_ne!(counter, 0, "QueryPerformanceCounter got zero counter");
                    unsafe { QueryPerformanceCounter(&mut start) };
                    let ms_counter = (counter / 1000).max(1).cast_unsigned();
                    while parked.load(Ordering::Relaxed) {
                        let mut now = 0i64;
                        unsafe { QueryPerformanceCounter(&mut now) };

                        let delta = now.wrapping_sub(start).cast_unsigned() / ms_counter;
                        if delta as u128 > timeout.as_millis() {
                            parked.store(false, Ordering::Relaxed);
                        }
                    }
                }
                None => while parked.load(Ordering::Relaxed) {}
            }
        }),
        wake: Box::new(|thread: &Thread| {
            thread.parked.store(false, Ordering::Relaxed);
        })
    }
}
