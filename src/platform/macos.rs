use std::{io, mem, ptr};

use bytesize::ByteSize;
use libc::{
    c_int, host_processor_info, host_statistics64, mach_host_self,
    mach_msg_type_number_t, natural_t, processor_cpu_load_info_data_t, sysconf,
    vm_statistics64, HOST_VM_INFO64, KERN_SUCCESS, PROCESSOR_CPU_LOAD_INFO,
    _SC_PHYS_PAGES,
};
use mach2::traps::mach_task_self;

use crate::{
    data::SystemCpuLoad, PlatformMemory, PlatformSwap, SystemMemory, SystemSwap,
};

use super::interface::Measurement;

pub struct MeasurementImpl;

fn fetch_cpu_load() -> std::io::Result<Vec<crate::data::SystemCpuLoad>> {
    unsafe {
        let mut num_cpu: natural_t = 0;
        let mut cpu_info: *mut processor_cpu_load_info_data_t = ptr::null_mut();
        let mut num_cpu_info: mach_msg_type_number_t = 0;

        let result = host_processor_info(
            mach_host_self(),
            PROCESSOR_CPU_LOAD_INFO,
            &mut num_cpu,
            &mut cpu_info as *mut _ as *mut *mut _,
            &mut num_cpu_info,
        );

        if result != 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to fetch CPU load",
            ));
        }

        let cpu_info_array =
            std::slice::from_raw_parts(cpu_info, num_cpu as usize);

        // Sum up the CPU times for each processor
        let mut cpu_loads = vec![];

        for cpu in cpu_info_array {
            log::debug!(
                "cpu: {:?}",
                cpu.cpu_ticks.iter().map(|x| x).collect::<Vec<_>>()
            );
            cpu_loads.push(SystemCpuLoad {
                user: cpu.cpu_ticks[0] as f32,
                system: cpu.cpu_ticks[1] as f32,
                idle: cpu.cpu_ticks[2] as f32,
                nice: cpu.cpu_ticks[3] as f32,
                interrupt: 0.0,
                platform: crate::data::PlatformCpuLoad {},
            });
        }

        libc::vm_deallocate(
            mach_task_self(),
            cpu_info as usize,
            (num_cpu_info as usize
                * mem::size_of::<processor_cpu_load_info_data_t>())
                as usize,
        );

        Ok(cpu_loads)
    }
}

impl Measurement for MeasurementImpl {
    fn new() -> Self {
        MeasurementImpl
    }

    fn cpu_load(
        &self,
    ) -> std::io::Result<
        crate::data::DelayedMeasurement<Vec<crate::data::SystemCpuLoad>>,
    > {
        Ok(crate::data::DelayedMeasurement::new(Box::new(
            fetch_cpu_load,
        )))
    }

    fn cpu_load_by_pid(
        &self,
        _pid: u32,
    ) -> std::io::Result<crate::DelayedMeasurement<f64>> {
        Err(io::Error::new(io::ErrorKind::Other, "Not supported"))
    }

    fn memory_by_pid(&self, _pid: u32) -> std::io::Result<(f64, f64)> {
        Err(io::Error::new(io::ErrorKind::Other, "Not supported"))
    }

    fn memory(&self) -> std::io::Result<crate::SystemMemory> {
        let total = match unsafe { sysconf(_SC_PHYS_PAGES) } {
            -1 => return Err(io::Error::last_os_error()),
            x => x as u64,
        };

        let mut vm_stats: vm_statistics64 = unsafe { mem::zeroed() };
        let mut count = mem::size_of::<vm_statistics64>() as u32
            / mem::size_of::<c_int>() as u32;

        let result = unsafe {
            libc::host_statistics64(
                mach_host_self(),
                libc::HOST_VM_INFO64,
                &mut vm_stats as *mut _ as *mut c_int,
                &mut count,
            )
        };

        match result {
            KERN_SUCCESS => {
                let page_size =
                    unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as u64;
                let pmem = PlatformMemory {
                    total: ByteSize::b(total * page_size),
                    free: ByteSize::b(vm_stats.free_count as u64 * page_size),
                    active: ByteSize::b(
                        vm_stats.active_count as u64 * page_size,
                    ),
                    inactive: ByteSize::b(
                        vm_stats.inactive_count as u64 * page_size,
                    ),
                    wired: ByteSize::b(vm_stats.wire_count as u64 * page_size),
                    compressor: ByteSize::b(
                        vm_stats.compressor_page_count as u64 * page_size,
                    ),
                };

                Ok(SystemMemory {
                    total: pmem.total,
                    free: pmem.free,
                    platform: pmem,
                })
            }
            _ => return Err(io::Error::last_os_error()),
        }
    }

    fn swap(&self) -> std::io::Result<crate::SystemSwap> {
        let mut vm_stats: vm_statistics64 = unsafe { mem::zeroed() };
        let mut count = mem::size_of::<vm_statistics64>() as u32
            / mem::size_of::<c_int>() as u32;

        // Get virtual memory statistics
        let result = unsafe {
            host_statistics64(
                mach_host_self(),
                HOST_VM_INFO64,
                &mut vm_stats as *mut _ as *mut c_int,
                &mut count,
            )
        };

        match result {
            KERN_SUCCESS => {
                let page_size =
                    unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as u64;

                let pswap = PlatformSwap {
                    total: ByteSize::b(vm_stats.swapins as u64 * page_size),
                    avail: ByteSize::b(vm_stats.swapouts as u64 * page_size),
                };
                Ok(SystemSwap {
                    total: pswap.total,
                    free: pswap.avail,
                    platform_swap: pswap,
                })
            }
            _ => return Err(io::Error::last_os_error()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory() {
        let m = MeasurementImpl::new();
        let mem = m.memory().unwrap();
        let swap = m.swap().unwrap();
        println!("total: {}, free: {}\n{:?}", mem.total, mem.free, mem);
        println!("total: {}, free: {}\n{:?}", swap.total, swap.free, swap);
    }
}
