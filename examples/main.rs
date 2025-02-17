use sys_measure::{Measurement, PlatformMeasurement};

fn main() {
    println!("Hello, world!");
    let measuare = PlatformMeasurement::new();

    let cpu_load = measuare.cpu_load().unwrap();
    println!("{:?}", cpu_load.done());
    let mem = measuare.memory().unwrap();
    let swap = measuare.swap().unwrap();
    println!("total: {}, free: {}\n{:?}", mem.total, mem.free, mem);
    println!("total: {}, free: {}\n{:?}", swap.total, swap.free, swap);
}
