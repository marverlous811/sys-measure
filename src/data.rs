use std::io;

pub mod cpu;
pub mod memory;

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
}

impl<T> DelayedMeasurement<T> {
    #[inline(always)]
    pub fn new(f: DelayedCallback<T>) -> DelayedMeasurement<T> {
        DelayedMeasurement { res: f }
    }

    #[inline(always)]
    pub fn done(&self) -> io::Result<T> {
        (self.res)()
    }
}
