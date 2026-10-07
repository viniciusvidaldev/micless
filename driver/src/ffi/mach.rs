// mach_timebase_info_data_t: host ticks to nanoseconds is ticks * numer / denom.
#[repr(C)]
pub struct TimebaseInfo {
    pub numer: u32,
    pub denom: u32,
}

unsafe extern "C" {
    pub fn mach_absolute_time() -> u64;
    pub fn mach_timebase_info(info: *mut TimebaseInfo) -> i32;
}
