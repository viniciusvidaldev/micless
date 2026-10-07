use super::CFUUIDBytes;

// COM status code, only used by query_interface.
pub type HResult = i32;

// COM success.
pub const S_OK: HResult = 0;

// COM error: the requested interface isn't implemented.
pub const E_NOINTERFACE: HResult = 0x8000_0004_u32 as HResult;

// kAudioServerPlugInTypeUUID: the factory is asked for this type.
pub const AUDIO_SERVER_PLUGIN_TYPE: CFUUIDBytes = CFUUIDBytes([
    0x44, 0x3A, 0xBA, 0xB8, 0xE7, 0xB3, 0x49, 0x1A, 0xB9, 0x85, 0xBE, 0xB9, 0x18, 0x70, 0x30, 0xDB,
]);

// kAudioServerPlugInDriverInterfaceUUID: query_interface is asked for this interface.
pub const DRIVER_INTERFACE: CFUUIDBytes = CFUUIDBytes([
    0xEE, 0xA5, 0x77, 0x3D, 0xCC, 0x43, 0x49, 0xF1, 0x8E, 0x00, 0x8F, 0x96, 0xE7, 0xD2, 0x3B, 0x17,
]);

// IUnknownUUID: the base COM interface, query_interface must accept it too.
pub const IUNKNOWN: CFUUIDBytes = CFUUIDBytes([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46,
]);
