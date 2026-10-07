use crate::{driver_ptr, ffi::*};

use std::{ffi::c_void, ptr};

/// # Safety
/// `out_interface` must be valid for a pointer write.
pub unsafe extern "C" fn query_interface(
    _driver: *mut c_void,
    uuid: CFUUIDBytes,
    out_interface: *mut *mut c_void,
) -> com::HResult {
    if uuid == com::IUNKNOWN || uuid == com::DRIVER_INTERFACE {
        unsafe { *out_interface = driver_ptr() };
        com::S_OK
    } else {
        unsafe { *out_interface = ptr::null_mut() };
        com::E_NOINTERFACE
    }
}

pub unsafe extern "C" fn add_ref(_driver: *mut c_void) -> u32 {
    1
}

pub unsafe extern "C" fn release(_driver: *mut c_void) -> u32 {
    1
}
