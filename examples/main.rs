use sys_measure::{platform::unix, Measurement, PlatformMeasurement};

fn main() {
    println!("Hello, world!");
    let start = std::time::Instant::now();
    let cur = std::process::id();
    let measuare = PlatformMeasurement::new();

    let cpu_load = measuare.cpu_load().unwrap();
    println!("{:?}", cpu_load.done());

    let cpu_load_aggregate = measuare.cpu_load_aggregate().unwrap();
    println!("agrregate: {:?}\n", cpu_load_aggregate.done());
    let mem = measuare.memory().unwrap();
    let swap = measuare.swap().unwrap();
    println!("total: {}, free: {}\n", mem.total, mem.free);
    println!("total: {}, free: {}\n", swap.total, swap.free);

    let networks = measuare.networks().unwrap();
    for (name, network) in networks {
        println!("Network: {}", name);
        for addr in network.addrs {
            println!("  Addr: {:?}, Netmask: {:?}", addr.addr, addr.netmask)
        }
        let stat = measuare.network_stats(&name).unwrap();
        println!("  Stats: {:?}", stat);
    }

    let socket_stats = measuare.socket_stats().unwrap();
    println!("Socket Stats: {:?}", socket_stats);

    let mounts = measuare.mounts().unwrap();
    for fs in mounts {
        println!(
            "Mount: {} on {} type {}",
            fs.fs_mounted_from, fs.fs_mounted_on, fs.fs_type
        );
        println!("  Total: {}, Free: {}\n", fs.total, fs.free);
        println!("File system: {:?}\n", fs);
    }

    let root = measuare.mount_at("/").unwrap();
    println!("Root mount: {:?}\n", root);

    let boot_time = measuare.boot_time().unwrap();
    println!("Boot time: {}\n", boot_time);

    let process_uptime = measuare.process_uptime(cur).unwrap();
    println!("Process uptime: {:?}\n", process_uptime);

    println!("Elapsed: {:?}\n", start.elapsed());
}
