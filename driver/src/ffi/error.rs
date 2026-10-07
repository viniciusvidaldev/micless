use super::{OSStatus, fourcc};

// kAudioHardwareNoError: success.
pub const NO_ERROR: OSStatus = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Error {
    /// kAudioHardwareUnknownPropertyError
    UnknownProperty = fourcc(b"who?") as i32,
    /// kAudioHardwareUnsupportedOperationError
    Unsupported = fourcc(b"unop") as i32,
    /// kAudioHardwareBadPropertySizeError
    BadPropertySize = fourcc(b"!siz") as i32,
}

impl Error {
    pub fn code(self) -> OSStatus {
        self as OSStatus
    }
}

pub fn status(result: Result<(), Error>) -> OSStatus {
    match result {
        Ok(()) => NO_ERROR,
        Err(e) => e.code(),
    }
}
