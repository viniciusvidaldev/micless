use std::{
    ffi::{CStr, c_void},
    ptr,
};

use crate::{
    ffi::{
        format::{RangedFormat, StreamFormat},
        *,
    },
    object::lookup,
};

pub enum Value {
    U32(u32),
    F64(f64),
    Str(&'static CStr),
    Ids(&'static [ObjectId]),
    Format(StreamFormat),
    Ranges(&'static [ValueRange]),
    Formats(&'static [RangedFormat]),
}

impl Value {
    fn size(&self) -> u32 {
        let bytes = match self {
            Self::U32(_) => size_of::<u32>(),
            Value::Str(_) => size_of::<CFStringRef>(),
            Self::Format(_) => size_of::<StreamFormat>(),
            Self::Ids(items) => size_of_val(*items),
            Self::Ranges(items) => size_of_val(*items),
            Self::Formats(items) => size_of_val(*items),
            Self::F64(_) => size_of::<f64>(),
        };
        bytes as u32
    }

    unsafe fn write(&self, out: *mut c_void, capacity: u32) -> Result<u32, Error> {
        // coreaudiod expects lists to be truncated to whatever fits, not rejected.
        match self {
            Self::Ids(items) => return Ok(unsafe { write_list(items, out, capacity) }),
            Self::Ranges(items) => return Ok(unsafe { write_list(items, out, capacity) }),
            Self::Formats(items) => return Ok(unsafe { write_list(items, out, capacity) }),
            _ => {}
        }

        // Single values must fit whole, a partial write would be meaningless.
        let size = self.size();
        if capacity < size {
            return Err(Error::BadPropertySize);
        }

        unsafe {
            match self {
                Self::U32(v) => out.cast::<u32>().write_unaligned(*v),
                Self::F64(v) => out.cast::<f64>().write_unaligned(*v),
                Self::Format(f) => out.cast::<StreamFormat>().write_unaligned(*f),
                Self::Str(s) => {
                    // The caller takes ownership and releases it.
                    let string = CFStringCreateWithCString(ptr::null(), s.as_ptr(), UTF8);
                    out.cast::<CFStringRef>().write_unaligned(string);
                }
                Self::Ids(_) | Self::Ranges(_) | Self::Formats(_) => unreachable!(),
            }
        }

        Ok(size)
    }
}

unsafe fn write_list<T: Copy>(items: &[T], out: *mut c_void, capacity: u32) -> u32 {
    let fits = capacity as usize / size_of::<T>();
    let count = items.len().min(fits);

    let out = out.cast::<T>();
    for (i, item) in items[..count].iter().enumerate() {
        unsafe { out.add(i).write_unaligned(*item) };
    }

    let bytes_copied = count * size_of::<T>();
    bytes_copied as u32
}

fn log_query(object: ObjectId, address: &PropertyAddress) {
    let code = address.selector.to_be_bytes();
    let text = [code[0], code[1], code[2], code[3], 0];
    unsafe {
        libc::syslog(
            libc::LOG_NOTICE,
            c"micless: has_property object %u selector %s".as_ptr(),
            object,
            text.as_ptr(),
        )
    };
}

pub unsafe extern "C" fn has_property(
    _driver: Driver,
    object: ObjectId,
    _client_pid: Pid,
    address: *const PropertyAddress,
) -> Boolean {
    let address = unsafe { &*address };
    log_query(object, address);
    lookup(object, address).is_some() as Boolean
}

pub unsafe extern "C" fn is_property_settable(
    _driver: Driver,
    object: ObjectId,
    _client_pid: Pid,
    address: *const PropertyAddress,
    out_settable: *mut Boolean,
) -> OSStatus {
    let address = unsafe { &*address };
    let result = lookup(object, address).ok_or(Error::UnknownProperty);
    status(result.map(|_| unsafe { out_settable.write(0) }))
}

pub unsafe extern "C" fn get_property_data_size(
    _driver: Driver,
    object: ObjectId,
    _client_pid: Pid,
    address: *const PropertyAddress,
    _qualifier_size: u32,
    _qualifier: *const c_void,
    out_size: *mut u32,
) -> OSStatus {
    let address = unsafe { &*address };
    let result = lookup(object, address).ok_or(Error::UnknownProperty);
    status(result.map(|value| unsafe { out_size.write(value.size()) }))
}

pub unsafe extern "C" fn get_property_data(
    _driver: Driver,
    object: ObjectId,
    _client_pid: Pid,
    address: *const PropertyAddress,
    _qualifier_size: u32,
    _qualifier: *const c_void,
    data_size: u32,
    out_size: *mut u32,
    out_data: *mut c_void,
) -> OSStatus {
    let address = unsafe { &*address };
    let result = lookup(object, address)
        .ok_or(Error::UnknownProperty)
        .and_then(|value| unsafe { value.write(out_data, data_size) });
    status(result.map(|written| unsafe { out_size.write(written) }))
}

pub unsafe extern "C" fn set_property_data(
    _driver: Driver,
    _object: ObjectId,
    _client_pid: Pid,
    _address: *const PropertyAddress,
    _qualifier_size: u32,
    _qualifier: *const c_void,
    _data_size: u32,
    _data: *const c_void,
) -> OSStatus {
    Error::Unsupported.code()
}
