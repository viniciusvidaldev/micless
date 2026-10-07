use std::ffi::c_void;

use super::{
    Boolean, CFUUIDBytes, Driver, Host, OSStatus, ObjectId, Pid, PropertyAddress, com::HResult,
};

#[repr(C)]
pub struct DriverInterface {
    pub _reserved: *mut c_void,

    pub query_interface: unsafe extern "C" fn(
        driver: *mut c_void,
        uuid: CFUUIDBytes,
        out_interface: *mut *mut c_void,
    ) -> HResult,

    pub add_ref: unsafe extern "C" fn(driver: *mut c_void) -> u32,

    pub release: unsafe extern "C" fn(driver: *mut c_void) -> u32,

    pub initialize: unsafe extern "C" fn(driver: Driver, host: Host) -> OSStatus,

    pub create_device: unsafe extern "C" fn(
        driver: Driver,
        description: *const c_void,
        client_info: *const c_void,
        out_device: *mut ObjectId,
    ) -> OSStatus,

    pub destroy_device: unsafe extern "C" fn(driver: Driver, device: ObjectId) -> OSStatus,

    pub add_device_client: unsafe extern "C" fn(
        driver: Driver,
        device: ObjectId,
        client_info: *const c_void,
    ) -> OSStatus,

    pub remove_device_client: unsafe extern "C" fn(
        driver: Driver,
        device: ObjectId,
        client_info: *const c_void,
    ) -> OSStatus,

    pub perform_device_configuration_change: unsafe extern "C" fn(
        driver: Driver,
        device: ObjectId,
        action: u64,
        info: *mut c_void,
    ) -> OSStatus,

    pub abort_device_configuration_change: unsafe extern "C" fn(
        driver: Driver,
        device: ObjectId,
        action: u64,
        info: *mut c_void,
    ) -> OSStatus,

    pub has_property: unsafe extern "C" fn(
        driver: Driver,
        object: ObjectId,
        client_pid: Pid,
        address: *const PropertyAddress,
    ) -> Boolean,

    pub is_property_settable: unsafe extern "C" fn(
        driver: Driver,
        object: ObjectId,
        client_pid: Pid,
        address: *const PropertyAddress,
        out_settable: *mut Boolean,
    ) -> OSStatus,

    pub get_property_data_size: unsafe extern "C" fn(
        driver: Driver,
        object: ObjectId,
        client_pid: Pid,
        address: *const PropertyAddress,
        qualifier_size: u32,
        qualifier: *const c_void,
        out_size: *mut u32,
    ) -> OSStatus,

    pub get_property_data: unsafe extern "C" fn(
        driver: Driver,
        object: ObjectId,
        client_pid: Pid,
        address: *const PropertyAddress,
        qualifier_size: u32,
        qualifier: *const c_void,
        data_size: u32,
        out_size: *mut u32,
        out_data: *mut c_void,
    ) -> OSStatus,

    pub set_property_data: unsafe extern "C" fn(
        driver: Driver,
        object: ObjectId,
        client_pid: Pid,
        address: *const PropertyAddress,
        qualifier_size: u32,
        qualifier: *const c_void,
        data_size: u32,
        data: *const c_void,
    ) -> OSStatus,

    pub start_io: unsafe extern "C" fn(driver: Driver, device: ObjectId, client: u32) -> OSStatus,

    pub stop_io: unsafe extern "C" fn(driver: Driver, device: ObjectId, client: u32) -> OSStatus,

    pub get_zero_time_stamp: unsafe extern "C" fn(
        driver: Driver,
        device: ObjectId,
        client: u32,
        out_sample_time: *mut f64,
        out_host_time: *mut u64,
        out_seed: *mut u64,
    ) -> OSStatus,

    pub will_do_io_operation: unsafe extern "C" fn(
        driver: Driver,
        device: ObjectId,
        client: u32,
        operation: u32,
        out_will_do: *mut Boolean,
        out_in_place: *mut Boolean,
    ) -> OSStatus,

    pub begin_io_operation: unsafe extern "C" fn(
        driver: Driver,
        device: ObjectId,
        client: u32,
        operation: u32,
        frames: u32,
        cycle: *const c_void,
    ) -> OSStatus,

    pub do_io_operation: unsafe extern "C" fn(
        driver: Driver,
        device: ObjectId,
        stream: ObjectId,
        client: u32,
        operation: u32,
        frames: u32,
        cycle: *const c_void,
        main_buffer: *mut c_void,
        secondary_buffer: *mut c_void,
    ) -> OSStatus,

    pub end_io_operation: unsafe extern "C" fn(
        driver: Driver,
        device: ObjectId,
        client: u32,
        operation: u32,
        frames: u32,
        cycle: *const c_void,
    ) -> OSStatus,
}

// SAFETY: only used as an immutable static, and _reserved is always null.
unsafe impl Sync for DriverInterface {}
