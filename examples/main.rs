use sys_measure::{Measurement, PlatformMeasurement};

fn main() {
    println!("Hello, world!");
    let measuare = PlatformMeasurement::new();

    let cpu_load = measuare.cpu_load().unwrap();
    println!("{:?}", cpu_load.done());

    let cpu_load_aggregate = measuare.cpu_load_aggregate().unwrap();
    println!("agrregate: {:?}", cpu_load_aggregate.done());
    let mem = measuare.memory().unwrap();
    let swap = measuare.swap().unwrap();
    println!("total: {}, free: {}\n{:?}", mem.total, mem.free, mem);
    println!("total: {}, free: {}\n{:?}", swap.total, swap.free, swap);
}
