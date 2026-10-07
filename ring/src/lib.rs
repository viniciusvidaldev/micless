use std::{
    cell::UnsafeCell,
    ffi::CString,
    fs::File,
    io,
    os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd},
    ptr,
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
};

pub const SHM_NAME: &str = "/micless.ring";
pub const MAGIC: u32 = 0x4d49_434c; // MICL
pub const VERSION: u32 = 1;
pub const SAMPLE_RATE: u32 = 48_000;
pub const CHANNELS: u32 = 1;

// Must be a power of two.
pub const CAPACITY: usize = 8192;
const _: () = assert!(CAPACITY.is_power_of_two());

const MASK: usize = CAPACITY - 1;

#[repr(C)]
struct Ring {
    magic: AtomicU32,
    version: AtomicU32,
    /// Samples consumed. Stored only by the consumer, after copying out.
    read: AtomicU64,
    /// Samples produced. Stored only by the producer, after copying in.
    write: AtomicU64,
    samples: UnsafeCell<[i16; CAPACITY]>,
}

unsafe impl Sync for Ring {}

impl Ring {
    /// Opens the ring, creating and initializing it if needed. Producer only.
    fn create() -> io::Result<&'static Ring> {
        // The driver runs as _coreaudiod, so it needs read and write as "other".
        // The umask would strip those bits from the mode passed to shm_open.
        let old_mask = unsafe { libc::umask(0) };
        let file = shm_open(libc::O_RDWR | libc::O_CREAT);
        unsafe { libc::umask(old_mask) };
        let file = file?;

        Self::ensure_size(&file)?;
        let ring = Self::map(&file)?;

        if ring.magic.load(Ordering::Acquire) == 0 {
            ring.version.store(VERSION, Ordering::Relaxed);
            ring.magic.store(MAGIC, Ordering::Release);
        }

        ring.validate()?;
        Ok(ring)
    }

    /// Opens an existing ring. Fails if the producer hasn't created it yet.
    fn attach() -> io::Result<&'static Ring> {
        let file = shm_open(libc::O_RDWR)?;

        if file.metadata()?.len() < size_of::<Self>() as u64 {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "ring not sized yet",
            ));
        }

        let ring = Self::map(&file)?;
        ring.validate()?;
        Ok(ring)
    }

    fn map(file: &File) -> io::Result<&'static Ring> {
        let ptr = mmap_region(file.as_fd(), size_of::<Self>())?;
        Ok(unsafe { &*(ptr as *const Ring) })
    }

    fn validate(&self) -> io::Result<()> {
        let magic = self.magic.load(Ordering::Acquire);
        let version = self.version.load(Ordering::Relaxed);
        if magic != MAGIC || version != VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "ring header mismatch",
            ));
        }
        Ok(())
    }

    /// Ensures the object is large enough to hold a `Ring` before mapping, since
    /// touching unbacked pages crashes. Safe to race with other processes.
    fn ensure_size(file: &File) -> io::Result<()> {
        let size = size_of::<Self>() as u64;
        if file.metadata()?.len() >= size {
            return Ok(());
        }
        // macOS allows ftruncate only once per object, so this fails if another process
        // sized it first. That is fine as long as the object now has the required size.
        file.set_len(size).or_else(|e| {
            if file.metadata()?.len() >= size {
                Ok(())
            } else {
                Err(e)
            }
        })
    }

    fn slot(&self, i: usize) -> *mut i16 {
        debug_assert!(i < CAPACITY);
        // Pointer arithmetic only, no reference to the array is created.
        unsafe { self.samples.get().cast::<i16>().add(i) }
    }
}

fn shm_open(flags: libc::c_int) -> io::Result<File> {
    let name = CString::new(SHM_NAME).unwrap();
    let fd = unsafe { libc::shm_open(name.as_ptr(), flags, 0o666 as libc::c_uint) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(File::from(unsafe { OwnedFd::from_raw_fd(fd) }))
}

fn mmap_region(fd: BorrowedFd<'_>, size: usize) -> io::Result<*mut libc::c_void> {
    let ptr = unsafe {
        libc::mmap(
            ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd.as_raw_fd(),
            0,
        )
    };

    if ptr == libc::MAP_FAILED {
        return Err(io::Error::last_os_error());
    }

    Ok(ptr)
}

pub struct Producer(&'static Ring);
impl Producer {
    pub fn open() -> io::Result<Self> {
        Ok(Self(Ring::create()?))
    }

    pub fn free(&self) -> usize {
        let ring = self.0;
        let write = ring.write.load(Ordering::Relaxed);
        let read = ring.read.load(Ordering::Acquire);
        debug_assert!(write >= read);
        CAPACITY - (write - read) as usize
    }

    pub fn push(&mut self, samples: &[i16]) -> bool {
        if samples.len() > self.free() {
            return false;
        }

        let ring = self.0;
        let w = ring.write.load(Ordering::Relaxed);

        let start = (w as usize) & MASK;
        // Amount that fits without wrapping to the start of the ring.
        let before_wrap = samples.len().min(CAPACITY - start);
        let (head, tail) = samples.split_at(before_wrap);

        // SAFETY: these slots are free and stay exclusive to the
        // producer until `write` is published below.
        unsafe {
            ptr::copy_nonoverlapping(head.as_ptr(), ring.slot(start), head.len());
            ptr::copy_nonoverlapping(tail.as_ptr(), ring.slot(0), tail.len());
        };

        let written = w + (samples.len() as u64);
        ring.write.store(written, Ordering::Release);
        true
    }

    /// Removes the segment's name, so the next `open` creates a fresh one.
    /// Consumers that already mapped it keep their old mapping.
    pub fn unlink() -> io::Result<()> {
        let name = CString::new(SHM_NAME).unwrap();
        if unsafe { libc::shm_unlink(name.as_ptr()) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

pub struct Consumer(&'static Ring);

impl Consumer {
    pub fn open() -> io::Result<Self> {
        Ok(Self(Ring::attach()?))
    }

    pub fn len(&self) -> usize {
        let ring = self.0;
        let write = ring.write.load(Ordering::Acquire);
        let read = ring.read.load(Ordering::Relaxed);
        debug_assert!(write >= read);
        (write - read) as usize
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn pop(&mut self, out: &mut [i16]) -> usize {
        let n = out.len().min(self.len());
        let ring = self.0;
        let r = ring.read.load(Ordering::Relaxed);
        let start = (r as usize) & MASK;

        // Amount that fits without wrapping to the start of the ring.
        let before_wrap = n.min(CAPACITY - start);
        let (head, tail) = out[..n].split_at_mut(before_wrap);

        // SAFETY: these slots were published by the producer, acquired in `len()`.
        unsafe {
            ptr::copy_nonoverlapping(ring.slot(start), head.as_mut_ptr(), head.len());
            ptr::copy_nonoverlapping(ring.slot(0), tail.as_mut_ptr(), tail.len());
        }

        ring.read.store(r + (n as u64), Ordering::Release);
        n
    }

    /// Drops up to `count` samples without reading them, returns how many were dropped.
    pub fn discard(&mut self, count: usize) -> usize {
        let ring = self.0;
        let n = count.min(self.len());
        let r = ring.read.load(Ordering::Relaxed);
        ring.read.store(r + (n as u64), Ordering::Release);
        n
    }
}
