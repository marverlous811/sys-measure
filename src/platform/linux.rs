pub struct MeasurementImpl;

impl Measurement for MeasurementImpl {
    fn new() -> Self {
        MeasurementImpl
    }

    fn cpu_load(
        &self,
    ) -> std::io::Result<DelayedMeasurement<Vec<SystemCpuLoad>>> {
        todo!()
    }

    fn cpu_load_by_pid(
        &self,
        pid: u32,
    ) -> std::io::Result<DelayedMeasurement<f64>> {
        todo!()
    }

    fn memory(&self) -> std::io::Result<SystemMemory> {
        todo!()
    }

    fn memory_by_pid(&self, pid: u32) -> std::io::Result<(f64, f64)> {
        todo!()
    }

    fn swap(&self) -> std::io::Result<SystemSwap> {
        todo!()
    }
}
