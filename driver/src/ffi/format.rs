use super::{ValueRange, fourcc};

pub const LINEAR_PCM: u32 = fourcc(b"lpcm");

pub const FLOAT: u32 = 1 << 0;
pub const PACKED: u32 = 1 << 3;

// AudioStreamBasicDescription
#[repr(C)]
#[derive(Clone, Copy)]
pub struct StreamFormat {
    pub sample_rate: f64,
    pub format_id: u32,
    pub format_flags: u32,
    pub bytes_per_packet: u32,
    pub frames_per_packet: u32,
    pub bytes_per_frame: u32,
    pub channels_per_frame: u32,
    pub bits_per_channel: u32,
    pub reserved: u32,
}

// AudioStreamRangedDescription
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RangedFormat {
    pub format: StreamFormat,
    pub rate_range: ValueRange,
}
