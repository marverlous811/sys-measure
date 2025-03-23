use std::io;

use crate::data::*;

pub trait Measurement {
    fn new() -> Self;

    fn cpu_load(
        &self,
    ) -> std::io::Result<DelayedMeasurement<Vec<SystemCpuLoad>>>;

    fn cpu_load_aggregate(
        &self,
    ) -> io::Result<DelayedMeasurement<SystemCpuLoad>> {
        let measurement = self.cpu_load()?;
        Ok(DelayedMeasurement::new(Box::new(move || {
            measurement.done().map(|ls| {
                let mut it = ls.iter();
                let first = it.next().unwrap().clone(); // has to be a variable, rust moves the iterator otherwise
                it.fold(first, |acc, l| acc.avg_add(l))
            })
        })))
    }

    fn cpu_load_by_pid(
        &self,
        pid: u32,
    ) -> std::io::Result<DelayedMeasurement<f64>>;

    fn memory(&self) -> std::io::Result<SystemMemory>;
    fn memory_by_pid(&self, pid: u32) -> std::io::Result<(f64, f64)>;
    fn swap(&self) -> std::io::Result<SystemSwap>;
}
