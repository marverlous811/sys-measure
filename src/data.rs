use std::io;

pub mod cpu;
pub mod memory;

pub use cpu::*;
pub use memory::*;

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
