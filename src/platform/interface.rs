use crate::data::*;

pub trait Measurement {
    fn new() -> Self;

    fn cpu_load(
        &self,
    ) -> std::io::Result<DelayedMeasurement<Vec<SystemCpuLoad>>>;

    fn memory(&self) -> std::io::Result<SystemMemory>;
    fn swap(&self) -> std::io::Result<SystemSwap>;
}
