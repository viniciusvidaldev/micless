use crate::{
    ffi::{mach, *},
    log,
    object::FORMAT,
};
use std::{
    ffi::c_void,
    ptr,
    sync::{
        OnceLock,
        atomic::{AtomicPtr, AtomicU32, AtomicU64, Ordering},
    },
};

pub const CLOCK_PERIOD: u32 = 16_384;

static CLIENTS: AtomicU32 = AtomicU32::new(0);
static ANCHOR: AtomicU64 = AtomicU64::new(0);
static PERIODS: AtomicU64 = AtomicU64::new(0);
static TICKS_PER_FRAME: OnceLock<f64> = OnceLock::new();
static CONSUMER: AtomicPtr<ring::Consumer> = AtomicPtr::new(ptr::null_mut());

const NANOS_PER_SECOND: f64 = 1_000_000_000.0;

pub fn is_running() -> bool {
    CLIENTS.load(Ordering::Acquire) > 0
}

fn ticks_per_frame() -> f64 {
    *TICKS_PER_FRAME.get_or_init(|| {
        let mut info = mach::TimebaseInfo { numer: 0, denom: 0 };
        unsafe { mach::mach_timebase_info(&mut info) };
        let ticks_per_second = NANOS_PER_SECOND * info.denom as f64 / info.numer as f64;
        ticks_per_second / FORMAT.sample_rate
    })
}

pub fn attach() {
    if !CONSUMER.load(Ordering::Acquire).is_null() {
        return;
    }

    let consumer = match ring::Consumer::open() {
        Ok(consumer) => Box::into_raw(Box::new(consumer)),
        Err(_) => {
            log(c"ring not available yet");
            return;
        }
    };

    let swap = CONSUMER.compare_exchange(
        ptr::null_mut(),
        consumer,
        Ordering::AcqRel,
        Ordering::Acquire,
    );
    if swap.is_err() {
        // Another thread attached first, ours was never published.
        drop(unsafe { Box::from_raw(consumer) });
        return;
    }
    log(c"ring attached");
}

pub unsafe extern "C" fn start_io(_driver: Driver, _device: ObjectId, _client: u32) -> OSStatus {
    attach();
    if CLIENTS.fetch_add(1, Ordering::AcqRel) == 0 {
        // The clock must be anchored before the IO thread's first get_zero_time_stamp.
        PERIODS.store(0, Ordering::Relaxed);
        ANCHOR.store(unsafe { mach::mach_absolute_time() }, Ordering::Release);
    }
    NO_ERROR
}

pub unsafe extern "C" fn stop_io(_driver: Driver, _device: ObjectId, _client: u32) -> OSStatus {
    CLIENTS.fetch_sub(1, Ordering::AcqRel);
    NO_ERROR
}

pub unsafe extern "C" fn get_zero_time_stamp(
    _driver: Driver,
    _device: ObjectId,
    _client: u32,
    out_sample_time: *mut f64,
    out_host_time: *mut u64,
    out_seed: *mut u64,
) -> OSStatus {
    let now = unsafe { mach::mach_absolute_time() } as f64;
    let anchor = ANCHOR.load(Ordering::Acquire) as f64;
    let ticks_per_period = ticks_per_frame() * CLOCK_PERIOD as f64;

    let mut periods = PERIODS.load(Ordering::Relaxed);
    let next = anchor + (periods + 1) as f64 * ticks_per_period;
    if next <= now {
        periods += 1;
        PERIODS.store(periods, Ordering::Relaxed);
    }

    unsafe {
        out_sample_time.write((periods * CLOCK_PERIOD as u64) as f64);
        out_host_time.write((anchor + periods as f64 * ticks_per_period) as u64);
        out_seed.write(1);
    }
    NO_ERROR
}

pub unsafe extern "C" fn will_do_io_operation(
    _driver: Driver,
    _device: ObjectId,
    _client: u32,
    op: u32,
    out_will_do: *mut Boolean,
    out_in_place: *mut Boolean,
) -> OSStatus {
    let will_do = (op == operation::READ_INPUT) as Boolean;
    unsafe {
        out_will_do.write(will_do);
        out_in_place.write(1);
    }
    NO_ERROR
}

pub unsafe extern "C" fn begin_io_operation(
    _driver: Driver,
    _device: ObjectId,
    _client: u32,
    _op: u32,
    _frames: u32,
    _cycle: *const c_void,
) -> OSStatus {
    NO_ERROR
}

pub unsafe extern "C" fn do_io_operation(
    _driver: Driver,
    _device: ObjectId,
    _stream: ObjectId,
    _client: u32,
    op: u32,
    frames: u32,
    _cycle: *const c_void,
    main_buffer: *mut c_void,
    _secondary_buffer: *mut c_void,
) -> OSStatus {
    if op != operation::READ_INPUT {
        return NO_ERROR;
    }

    let len = (frames * FORMAT.channels_per_frame) as usize;
    let samples = unsafe { std::slice::from_raw_parts_mut(main_buffer.cast::<f32>(), len) };

    // SAFETY: only the IO thread pops, so this is the ring's single consumer.
    let Some(consumer) = (unsafe { CONSUMER.load(Ordering::Acquire).as_mut() }) else {
        samples.fill(0.0);
        return NO_ERROR;
    };

    let mut pcm = [0i16; 512];
    for chunk in samples.chunks_mut(pcm.len()) {
        let got = consumer.pop(&mut pcm[..chunk.len()]);
        for (i, out) in chunk.iter_mut().enumerate() {
            *out = if i < got {
                pcm[i] as f32 / 32768.0
            } else {
                0.0
            };
        }
    }
    NO_ERROR
}

pub unsafe extern "C" fn end_io_operation(
    _driver: Driver,
    _device: ObjectId,
    _client: u32,
    _op: u32,
    _frames: u32,
    _cycle: *const c_void,
) -> OSStatus {
    NO_ERROR
}
