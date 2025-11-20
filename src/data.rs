use std::{io, thread::sleep, time::Duration};

pub mod cpu;
pub mod disk;
pub mod memory;
pub mod network;
pub mod process;

use bytesize::ByteSize;
pub use cpu::*;
pub use memory::*;

#[inline(always)]
pub fn saturating_sub_bytes(l: ByteSize, r: ByteSize) -> ByteSize {
    ByteSize::b(l.as_u64().saturating_sub(r.as_u64()))
}

type DelayedCallback<T> = Box<dyn Fn() -> io::Result<T> + Send>;

pub struct DelayedMeasurement<T> {
    res: DelayedCallback<T>,
    duration: Duration,
}

impl<T> DelayedMeasurement<T> {
    #[inline(always)]
    pub fn new(
        f: DelayedCallback<T>,
        duration_sec: Option<u64>,
    ) -> DelayedMeasurement<T> {
        let sec = duration_sec.map_or_else(|| 1, |s| s);
        DelayedMeasurement {
            res: f,
            duration: Duration::from_secs(sec),
        }
    }

    #[inline(always)]
    pub fn done(&self) -> io::Result<T> {
        sleep(self.duration);
        (self.res)()
    }
}
