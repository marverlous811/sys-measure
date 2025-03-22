use crate::data::*;

pub trait Measurement {
    fn new() -> Self;

    fn cpu_load(
        &self,
    ) -> std::io::Result<DelayedMeasurement<Vec<SystemCpuLoad>>>;

    fn cpu_load_by_pid(
        &self,
        pid: u32,
    ) -> std::io::Result<DelayedMeasurement<f64>>;

    fn memory(&self) -> std::io::Result<SystemMemory>;
    fn memory_by_pid(&self, pid: u32) -> std::io::Result<(f64, f64)>;
    fn swap(&self) -> std::io::Result<SystemSwap>;
}
