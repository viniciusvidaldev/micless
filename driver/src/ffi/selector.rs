use super::fourcc;

// Any object
pub const BASE_CLASS: u32 = fourcc(b"bcls");
pub const CLASS: u32 = fourcc(b"clas");
pub const OWNER: u32 = fourcc(b"stdv");
pub const NAME: u32 = fourcc(b"lnam");
// pub const MODEL_NAME: u32 = fourcc(b"lmod");
pub const MANUFACTURER: u32 = fourcc(b"lmak");
pub const OWNED_OBJECTS: u32 = fourcc(b"ownd");
pub const CONTROL_LIST: u32 = fourcc(b"ctrl");
pub const CUSTOM_PROPERTY_INFO_LIST: u32 = fourcc(b"cust");

// Plugin
pub const DEVICE_LIST: u32 = fourcc(b"dev#");
pub const BOX_LIST: u32 = fourcc(b"box#");
pub const CLOCK_DEVICE_LIST: u32 = fourcc(b"clk#");
pub const RESOURCE_BUNDLE: u32 = fourcc(b"rsrc");
// pub const TRANSLATE_UID_TO_DEVICE: u32 = fourcc(b"uidd");

// Device
pub const DEVICE_UID: u32 = fourcc(b"uid ");
pub const MODEL_UID: u32 = fourcc(b"muid");
pub const TRANSPORT_TYPE: u32 = fourcc(b"tran");
pub const RELATED_DEVICES: u32 = fourcc(b"akin");
pub const CLOCK_DOMAIN: u32 = fourcc(b"clkd");
// pub const CLOCK_IS_STABLE: u32 = fourcc(b"cstb");
pub const IS_ALIVE: u32 = fourcc(b"livn");
pub const IS_RUNNING: u32 = fourcc(b"goin");
pub const IS_HIDDEN: u32 = fourcc(b"hidn");
pub const CAN_BE_DEFAULT: u32 = fourcc(b"dflt");
pub const CAN_BE_DEFAULT_SYSTEM: u32 = fourcc(b"sflt");
pub const SAFETY_OFFSET: u32 = fourcc(b"saft");
pub const STREAMS: u32 = fourcc(b"stm#");
pub const NOMINAL_SAMPLE_RATE: u32 = fourcc(b"nsrt");
pub const AVAILABLE_NOMINAL_SAMPLE_RATES: u32 = fourcc(b"nsr#");
pub const ZERO_TIME_STAMP_PERIOD: u32 = fourcc(b"ring");
// pub const PREFERRED_CHANNELS_FOR_STEREO: u32 = fourcc(b"dch2");
// pub const PREFERRED_CHANNEL_LAYOUT: u32 = fourcc(b"srnd");

// Device and stream
pub const LATENCY: u32 = fourcc(b"ltnc");

// Stream
pub const IS_ACTIVE: u32 = fourcc(b"sact");
pub const DIRECTION: u32 = fourcc(b"sdir");
pub const TERMINAL_TYPE: u32 = fourcc(b"term");
pub const STARTING_CHANNEL: u32 = fourcc(b"schn");
pub const VIRTUAL_FORMAT: u32 = fourcc(b"sfmt");
pub const AVAILABLE_VIRTUAL_FORMATS: u32 = fourcc(b"sfma");
pub const PHYSICAL_FORMAT: u32 = fourcc(b"pft ");
pub const AVAILABLE_PHYSICAL_FORMATS: u32 = fourcc(b"pfta");
