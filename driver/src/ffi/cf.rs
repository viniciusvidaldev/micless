use std::ffi::{c_char, c_void};

// CoreFoundation allocator. Null means the default one.
pub type CFAllocatorRef = *const c_void;

// Opaque CoreFoundation UUID object.
pub type CFUUIDRef = *const c_void;

// Raw 16 bytes of a UUID, passed by value.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CFUUIDBytes(pub [u8; 16]);

// CoreFoundation string object. Opaque and reference counted.
pub type CFStringRef = *const c_void;

// Text encoding identifier used when creating a CFString.
pub type CFStringEncoding = u32;

// kCFStringEncodingUTF8
pub const UTF8: CFStringEncoding = 0x0800_0100;

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    pub fn CFUUIDGetUUIDBytes(uuid: CFUUIDRef) -> CFUUIDBytes;

    // Creates a CFString from a null terminated C string. The caller owns the result.
    pub fn CFStringCreateWithCString(
        alloc: CFAllocatorRef,
        s: *const c_char,
        encoding: CFStringEncoding,
    ) -> CFStringRef;
}
