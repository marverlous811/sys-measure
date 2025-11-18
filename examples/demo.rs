use std::{process, thread::sleep, time::Duration};

use sys_measure::{cpu, Measurement, PlatformMeasurement};
use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

fn fetch_process_info(
    measurement: &PlatformMeasurement,
    pid: u32,
) -> anyhow::Result<()> {
    log::info!("Fetching info for pid: {pid}");

    let pid_usage = measurement.cpu_load_by_pid(pid)?;

    let pid_cpu_usages = pid_usage.done()?;
    log::info!("pid {pid} has {pid_cpu_usages}%");

    let (vm_size, vm_rss) = measurement.memory_by_pid(pid)?;

    log::info!("Virtual Memory: {} KB", vm_size);
    log::info!("Resident Set Size (RSS): {} KB", vm_rss);
    log::info!("-----------------------------------");
    Ok(())
}

fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        }
    }

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().boxed())
        .with(EnvFilter::from_default_env())
        .init();

    let measuare = PlatformMeasurement::new();
    // let pid = 2403002;
    let cur = process::id();

    log::info!("Current process ID: {cur}");

    loop {
        // let _ = fetch_process_info(&measuare, pid);
        let _ = fetch_process_info(&measuare, cur);
        let cpu_load = measuare.cpu_load_aggregate().unwrap();
        let done = cpu_load.done().unwrap();
        let total_cpu = done.user + done.system + done.nice + done.idle;
        let usage = (1.0 - (done.idle / total_cpu)) * 100.0;
        log::info!("Aggregate CPU Usage: {:.2}%", usage);

        let memory = measuare.memory().unwrap();

        log::info!(
            "Memory - Total: {}, Free: {}, Used: {}",
            memory.total,
            memory.free,
            memory.used
        );
        log::info!("==================================================================");

        sleep(Duration::from_secs(10));
    }
}
