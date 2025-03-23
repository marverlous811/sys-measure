use std::{thread::sleep, time::Duration};

use sys_measure::{Measurement, PlatformMeasurement};

fn main() {
    let measuare = PlatformMeasurement::new();

    loop {
        let cpu_load = measuare.cpu_load().expect("must collectable cpu load");
        let load = cpu_load.done().expect("must ");

        let cpu_usages = load
            .iter()
            .map(|d| {
                let total = d.user + d.nice + d.system + d.interrupt + d.idle;
                if total == 0.0 {
                    0.0
                } else {
                    100.0 * (1.0 - d.idle as f64 / total as f64)
                }
            })
            .collect::<Vec<_>>();

        println!("cpu usage: {:?}", cpu_usages);

        sleep(Duration::from_secs(10));
    }
}
