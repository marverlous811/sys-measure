use std::{thread::sleep, time::Duration};

use sys_measure::{Measurement, PlatformMeasurement};

fn main() {
    let measuare = PlatformMeasurement::new();
    let pid = 17263;

    loop {
        // let cpu_load = measuare.cpu_load().expect("must collectable cpu load");
        // let load = cpu_load.done().expect("must ");

        // let cpu_usages = load
        //     .iter()
        //     .map(|d| {
        //         let total = d.user + d.nice + d.system + d.interrupt + d.idle;
        //         if total == 0.0 {
        //             0.0
        //         } else {
        //             100.0 * (1.0 - d.idle as f64 / total as f64)
        //         }
        //     })
        //     .collect::<Vec<_>>();
        // println!("cpu usage: {:?}", cpu_usages);

        let pid_usage = measuare
            .cpu_load_by_pid(pid)
            .expect("must collectable cpu load from pid");

        let pid_cpu_usages = pid_usage
            .done()
            .expect("must collectable cpu load from pid");
        println!("pid {pid} has {pid_cpu_usages}%");

        let (vm_size, vm_rss) = measuare
            .memory_by_pid(pid)
            .expect("must collectable memory stats from pid");

        println!("  Virtual Memory: {} KB", vm_size);
        println!("  Resident Set Size (RSS): {} KB", vm_rss);

        sleep(Duration::from_secs(10));
    }
}
