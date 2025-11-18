use crate::{
    data::*,
    disk::FileSystem,
    network::{Network, NetworkStats, SocketStats},
};
use std::{io, path, time::Duration};
use time::OffsetDateTime;

pub trait Measurement {
    fn new() -> Self;

    fn cpu_load(
        &self,
    ) -> std::io::Result<DelayedMeasurement<Vec<SystemCpuLoad>>>;

    fn cpu_load_aggregate(
        &self,
    ) -> io::Result<DelayedMeasurement<SystemCpuLoad>> {
        let measurement = self.cpu_load()?;
        Ok(DelayedMeasurement::new(
            Box::new(move || {
                measurement.done().map(|ls| {
                    let mut it = ls.iter();
                    let first = it.next().unwrap().clone(); // has to be a variable, rust moves the iterator otherwise
                    it.fold(first, |acc, l| acc.avg_add(l))
                })
            }),
            Some(0),
        ))
    }

    fn cpu_load_by_pid(
        &self,
        pid: u32,
    ) -> std::io::Result<DelayedMeasurement<f64>>;

    fn memory(&self) -> std::io::Result<SystemMemory>;
    fn memory_by_pid(&self, pid: u32) -> std::io::Result<(u64, u64)>;
    fn swap(&self) -> std::io::Result<SystemSwap>;
    fn mounts(&self) -> io::Result<Vec<FileSystem>>;
    fn mount_at<P: AsRef<path::Path>>(
        &self,
        path: P,
    ) -> io::Result<FileSystem> {
        self.mounts().and_then(|mounts| {
            mounts
                .into_iter()
                .find(|mount| {
                    path::Path::new(&mount.fs_mounted_on) == path.as_ref()
                })
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "No such mount")
                })
        })
    }
    fn networks(&self) -> io::Result<BTreeMap<String, Network>>;
    fn network_stats(&self, interface: &str) -> io::Result<NetworkStats>;
    fn socket_stats(&self) -> io::Result<SocketStats>;
    fn boot_time(&self) -> io::Result<OffsetDateTime>;
    fn process_uptime(&self, pid: u32) -> io::Result<Duration>;
}
