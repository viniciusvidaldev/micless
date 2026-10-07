mod com;
mod ffi;
mod io;
mod object;
mod plugin;
mod properties;

use std::{
    ffi::{CStr, c_void},
    ptr,
};

use crate::ffi::{CFAllocatorRef, CFUUIDRef, Driver, DriverInterface};

/// # Safety
/// `requested_type` must be a valid CFUUIDRef.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn micless_factory(
    _alloc: CFAllocatorRef,
    requested_type: CFUUIDRef,
) -> Driver {
    let request = unsafe { ffi::CFUUIDGetUUIDBytes(requested_type) };
    if request != ffi::com::AUDIO_SERVER_PLUGIN_TYPE {
        return ptr::null_mut();
    }

    log(c"factory called");
    driver_ptr()
}

fn log(msg: &CStr) {
    unsafe { libc::syslog(libc::LOG_NOTICE, c"micless: %s".as_ptr(), msg.as_ptr()) };
}

static INTERFACE: DriverInterface = DriverInterface {
    _reserved: ptr::null_mut(),
    query_interface: com::query_interface,
    add_ref: com::add_ref,
    release: com::release,
    initialize: plugin::initialize,
    create_device: plugin::create_device,
    destroy_device: plugin::destroy_device,
    add_device_client: plugin::add_device_client,
    remove_device_client: plugin::remove_device_client,
    perform_device_configuration_change: plugin::perform_device_configuration_change,
    abort_device_configuration_change: plugin::abort_device_configuration_change,
    has_property: properties::has_property,
    is_property_settable: properties::is_property_settable,
    get_property_data_size: properties::get_property_data_size,
    get_property_data: properties::get_property_data,
    set_property_data: properties::set_property_data,
    start_io: io::start_io,
    stop_io: io::stop_io,
    get_zero_time_stamp: io::get_zero_time_stamp,
    will_do_io_operation: io::will_do_io_operation,
    begin_io_operation: io::begin_io_operation,
    do_io_operation: io::do_io_operation,
    end_io_operation: io::end_io_operation,
};

static DRIVER: &DriverInterface = &INTERFACE;

fn driver_ptr() -> *mut c_void {
    &DRIVER as *const &DriverInterface as *mut c_void
}
