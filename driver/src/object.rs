use crate::ffi::*;
use crate::properties::Value;

pub const DEVICE_ID: ObjectId = 2;
pub const STREAM_ID: ObjectId = 3;

pub static DEVICES: [ObjectId; 1] = [DEVICE_ID];
pub static STREAMS: [ObjectId; 1] = [STREAM_ID];

pub const FORMAT: StreamFormat = StreamFormat {
    sample_rate: ring::SAMPLE_RATE as f64,
    format_id: format::LINEAR_PCM,
    format_flags: format::FLOAT | format::PACKED,
    bytes_per_packet: size_of::<f32>() as u32 * ring::CHANNELS,
    frames_per_packet: 1,
    bytes_per_frame: size_of::<f32>() as u32 * ring::CHANNELS,
    channels_per_frame: ring::CHANNELS,
    bits_per_channel: 32,
    reserved: 0,
};

const RATE_RANGE: ValueRange = ValueRange {
    min: FORMAT.sample_rate,
    max: FORMAT.sample_rate,
};

static RATES: [ValueRange; 1] = [RATE_RANGE];

static FORMATS: [RangedFormat; 1] = [RangedFormat {
    format: FORMAT,
    rate_range: RATE_RANGE,
}];

pub fn lookup(object: ObjectId, address: &PropertyAddress) -> Option<Value> {
    match object {
        PLUGIN_ID => plugin(address),
        DEVICE_ID => device(address),
        STREAM_ID => stream(address),
        _ => None,
    }
}

fn plugin(address: &PropertyAddress) -> Option<Value> {
    let value = match address.selector {
        selector::BASE_CLASS => Value::U32(class::OBJECT),
        selector::CLASS => Value::U32(class::PLUGIN),
        selector::OWNER => Value::U32(UNKNOWN_OBJECT),
        selector::MANUFACTURER => Value::Str(c"micless"),
        selector::RESOURCE_BUNDLE => Value::Str(c""),
        selector::OWNED_OBJECTS => Value::Ids(&DEVICES),
        selector::DEVICE_LIST => Value::Ids(&DEVICES),
        selector::BOX_LIST => Value::Ids(&[]),
        selector::CLOCK_DEVICE_LIST => Value::Ids(&[]),
        selector::CUSTOM_PROPERTY_INFO_LIST => Value::Ids(&[]),
        _ => return None,
    };
    Some(value)
}

fn device(address: &PropertyAddress) -> Option<Value> {
    let value = match address.selector {
        selector::BASE_CLASS => Value::U32(class::OBJECT),
        selector::CLASS => Value::U32(class::DEVICE),
        selector::OWNER => Value::U32(PLUGIN_ID),
        selector::NAME => Value::Str(c"micless"),
        selector::MANUFACTURER => Value::Str(c"micless"),
        selector::DEVICE_UID => Value::Str(c"micless.device"),
        selector::MODEL_UID => Value::Str(c"micless.model"),
        selector::TRANSPORT_TYPE => Value::U32(transport::VIRTUAL),
        selector::RELATED_DEVICES => Value::Ids(&DEVICES),
        selector::CLOCK_DOMAIN => Value::U32(0),
        selector::IS_ALIVE => Value::U32(1),
        selector::IS_RUNNING => Value::U32(crate::io::is_running() as u32),
        selector::IS_HIDDEN => Value::U32(0),
        selector::CAN_BE_DEFAULT => Value::U32(1),
        selector::CAN_BE_DEFAULT_SYSTEM => Value::U32(0),
        selector::LATENCY => Value::U32(0),
        selector::SAFETY_OFFSET => Value::U32(0),
        selector::NOMINAL_SAMPLE_RATE => Value::F64(FORMAT.sample_rate),
        selector::AVAILABLE_NOMINAL_SAMPLE_RATES => Value::Ranges(&RATES),
        selector::ZERO_TIME_STAMP_PERIOD => Value::U32(crate::io::CLOCK_PERIOD),
        selector::CONTROL_LIST => Value::Ids(&[]),
        selector::OWNED_OBJECTS => Value::Ids(&STREAMS),
        selector::STREAMS if address.scope == scope::OUTPUT => Value::Ids(&[]),
        selector::STREAMS => Value::Ids(&STREAMS),
        _ => return None,
    };
    Some(value)
}

fn stream(address: &PropertyAddress) -> Option<Value> {
    let value = match address.selector {
        selector::BASE_CLASS => Value::U32(class::OBJECT),
        selector::CLASS => Value::U32(class::STREAM),
        selector::OWNER => Value::U32(DEVICE_ID),
        selector::IS_ACTIVE => Value::U32(1),
        // 1 is input, 0 is output.
        selector::DIRECTION => Value::U32(1),
        selector::TERMINAL_TYPE => Value::U32(terminal::MICROPHONE),
        selector::STARTING_CHANNEL => Value::U32(1),
        selector::LATENCY => Value::U32(0),
        selector::VIRTUAL_FORMAT => Value::Format(FORMAT),
        selector::PHYSICAL_FORMAT => Value::Format(FORMAT),
        selector::AVAILABLE_VIRTUAL_FORMATS => Value::Formats(&FORMATS),
        selector::AVAILABLE_PHYSICAL_FORMATS => Value::Formats(&FORMATS),
        _ => return None,
    };
    Some(value)
}
