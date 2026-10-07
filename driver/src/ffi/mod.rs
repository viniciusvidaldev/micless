use std::ffi::c_void;

mod cf;
pub mod com;
mod error;
pub mod format;
mod interface;
pub mod mach;
pub mod selector;

pub use cf::*;
pub use error::*;
pub use format::{RangedFormat, StreamFormat};
pub use interface::*;

// Apple's status code. 0 is success, errors are usually fourccs.
pub type OSStatus = i32;

// Apple's unsigned char boolean. Any nonzero byte is true.
pub type Boolean = u8;

// AudioObjectID, the number naming one of the driver's objects.
pub type ObjectId = u32;

// pid_t of the process asking, often 0 for coreaudiod itself.
pub type Pid = i32;

// AudioServerPlugInDriverRef, the pointer the factory returned, passed back on every call.
pub type Driver = *mut c_void;

// AudioServerPlugInHostRef, coreaudiod's function table, received in initialize.
pub type Host = *const c_void;

// kAudioObjectUnknown
pub const UNKNOWN_OBJECT: ObjectId = 0;

// kAudioObjectPlugInObject: the plugin itself always has this ID.
pub const PLUGIN_ID: ObjectId = 1;

// Packs a four character code like b"who?" into a u32, big endian so it reads in order.
pub const fn fourcc(code: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*code)
}

// AudioObjectPropertyAddress: identifies one property of an object.
#[repr(C)]
pub struct PropertyAddress {
    // Which property, as a fourcc.
    pub selector: u32,
    // Which side: global, input, or output.
    pub scope: u32,
    // Which channel, 0 is the main element.
    pub element: u32,
}

// AudioValueRange
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ValueRange {
    pub min: f64,
    pub max: f64,
}

pub mod scope {
    use super::fourcc;

    pub const GLOBAL: u32 = fourcc(b"glob");
    pub const INPUT: u32 = fourcc(b"inpt");
    pub const OUTPUT: u32 = fourcc(b"outp");
}

pub mod transport {
    use super::fourcc;

    pub const VIRTUAL: u32 = fourcc(b"virt");
}

pub mod terminal {
    use super::fourcc;

    pub const MICROPHONE: u32 = fourcc(b"micr");
}

pub mod class {
    use super::fourcc;

    pub const OBJECT: u32 = fourcc(b"aobj");
    pub const PLUGIN: u32 = fourcc(b"aplg");
    pub const DEVICE: u32 = fourcc(b"adev");
    pub const STREAM: u32 = fourcc(b"astr");
}

pub mod operation {
    use super::fourcc;

    pub const READ_INPUT: u32 = fourcc(b"read");
}
