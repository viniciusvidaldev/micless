use crate::{ffi::*, log};

use std::{
    ffi::c_void,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

pub static HOST: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

pub unsafe extern "C" fn initialize(_driver: Driver, host: Host) -> OSStatus {
    HOST.store(host as *mut c_void, Ordering::Release);
    log(c"initialize called");
    crate::io::attach();
    NO_ERROR
}

pub unsafe extern "C" fn create_device(
    _driver: Driver,
    _description: *const c_void,
    _client_info: *const c_void,
    _out_device: *mut ObjectId,
) -> OSStatus {
    Error::Unsupported.code()
}

pub unsafe extern "C" fn destroy_device(_driver: Driver, _device: ObjectId) -> OSStatus {
    Error::Unsupported.code()
}

pub unsafe extern "C" fn add_device_client(
    _driver: Driver,
    _device: ObjectId,
    _client_info: *const c_void,
) -> OSStatus {
    NO_ERROR
}

pub unsafe extern "C" fn remove_device_client(
    _driver: Driver,
    _device: ObjectId,
    _client_info: *const c_void,
) -> OSStatus {
    NO_ERROR
}

pub unsafe extern "C" fn perform_device_configuration_change(
    _driver: Driver,
    _device: ObjectId,
    _action: u64,
    _info: *mut c_void,
) -> OSStatus {
    NO_ERROR
}

pub unsafe extern "C" fn abort_device_configuration_change(
    _driver: Driver,
    _device: ObjectId,
    _action: u64,
    _info: *mut c_void,
) -> OSStatus {
    NO_ERROR
}
