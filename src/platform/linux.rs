use std::{collections::BTreeMap, io, mem, str, thread::sleep, time::Duration};

use bytesize::ByteSize;
use libc::sysinfo;
use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{digit1, multispace0, not_line_ending, space1},
    combinator::{complete, map, map_res, opt, verify},
    error::ParseError,
    multi::{fold_many0, many1},
    sequence::{delimited, preceded},
    IResult, Parser,
};

use crate::{
    helper::read_file, saturating_sub_bytes, DelayedMeasurement, Measurement,
    PlatformMemory, SystemCpuLoad, SystemCpuTime, SystemMemory, SystemSwap,
};
pub struct MeasurementImpl;

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
pub fn ws<'a, O, E: ParseError<&'a str>, F>(
    inner: F,
) -> impl Parser<&'a str, Output = O, Error = E>
where
    F: Parser<&'a str, Output = O, Error = E>,
{
    delimited(multispace0, inner, multispace0)
}

fn proc_stat_cpu_prefix(input: &str) -> IResult<&str, ()> {
    map((tag("cpu"), digit1), |_| ()).parse(input)
}

fn proc_stat_cpu_aggregate(input: &str) -> IResult<&str, ()> {
    map((tag("cpu"), space1), |_| ()).parse(input)
}

fn num<T: str::FromStr>(input: &str) -> IResult<&str, T> {
    map_res(
        map_res(map(ws(digit1), str::as_bytes), str::from_utf8),
        str::FromStr::from_str,
    )
    .parse(input)
}

fn proc_stat_cpu_time(input: &str) -> IResult<&str, SystemCpuTime> {
    map(
        preceded(ws(proc_stat_cpu_prefix), (num, num, num, num, num, num)),
        |(user, nice, system, idle, iowait, irq)| SystemCpuTime {
            user,
            nice,
            system,
            idle,
            interrupt: irq,
            other: iowait,
        },
    )
    .parse(input)
}

fn proc_stat_cpu_times(input: &str) -> IResult<&str, Vec<SystemCpuTime>> {
    preceded(
        map(ws(not_line_ending), proc_stat_cpu_aggregate),
        many1(map_res(ws(not_line_ending), |input| {
            proc_stat_cpu_time(input)
                .map(|(_, res)| res)
                .map_err(|_| ())
        })),
    )
    .parse(input)
}

fn cpu_time() -> io::Result<Vec<SystemCpuTime>> {
    read_file("/proc/stat").and_then(|data| {
        proc_stat_cpu_times(&data)
            .map(|(_, res)| res)
            .map_err(|err| {
                io::Error::new(io::ErrorKind::InvalidData, err.to_string())
            })
    })
}

#[test]
fn test_proc_cpu_time() {
    let input = "cpu  571797 40417 361029 1709174488 192878 0 16794 2218 0 0
cpu0 139405 9696 97760 427330798 42717 0 5108 503 0 0
cpu1 141346 11865 81146 427319846 54009 0 5850 555 0 0
cpu2 150499 10066 99126 427182897 45689 0 2658 631 0 0
cpu3 140546 8788 82996 427340946 50462 0 3176 527 0 0
intr 273179808 36 9 0 0 802 0 3 0 1 0 2137722 0 15 0 0 4244315 0 0 0 0 0 0 0 0 0 0 0 997919 1499347 1107934 1382240 0 16591327 7669930 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
ctxt 392734296
btime 1738426511
processes 157384
procs_running 6
procs_blocked 0
softirq 191683246 1 74513529 13 24377461 2137122 0 199 78364024 64545 12226352";
    let result = proc_stat_cpu_times(input).unwrap().1;
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].user, 139405);
    assert_eq!(result[0].nice, 9696)
}

fn proc_meminfo_line(input: &str) -> IResult<&str, (&str, ByteSize)> {
    complete(map(
        (take_until(":"), delimited(tag(":"), num, ws(tag("kB")))),
        |(key, value)| (key, ByteSize::kib(value)),
    ))
    .parse(input)
}

fn proc_meminfo_line_opt(
    input: &str,
) -> IResult<&str, Option<(&str, ByteSize)>> {
    opt(proc_meminfo_line).parse(input)
}

fn proc_meminfo(input: &str) -> IResult<&str, BTreeMap<String, ByteSize>> {
    fold_many0(
        map_res(
            verify(ws(not_line_ending), |item: &str| !item.is_empty()),
            |input| {
                proc_meminfo_line_opt(input)
                    .map(|(_, res)| res)
                    .map_err(|_| ())
            },
        ),
        BTreeMap::new,
        |mut map: BTreeMap<String, ByteSize>, opt| {
            if let Some((key, val)) = opt {
                map.insert(key.to_string(), val);
            }
            map
        },
    )
    .parse(input)
}

fn memory_stats() -> io::Result<BTreeMap<String, ByteSize>> {
    read_file("/proc/meminfo").and_then(|data| {
        proc_meminfo(&data).map(|(_, res)| res).map_err(|err| {
            io::Error::new(io::ErrorKind::InvalidData, err.to_string())
        })
    })
}

#[test]
fn test_proc_meminfo() {
    let input = "MemTotal:       32345596 kB
MemFree:        13160208 kB
MemAvailable:   27792164 kB
Buffers:            4724 kB
Cached:         14776312 kB
SwapCached:            0 kB
Active:          8530160 kB
Inactive:        9572028 kB
Active(anon):      18960 kB
Inactive(anon):  3415400 kB
Active(file):    8511200 kB
Inactive(file):  6156628 kB
Unevictable:           0 kB
Mlocked:               0 kB
SwapTotal:       6143996 kB
SwapFree:        6143996 kB
Dirty:             66124 kB
Writeback:             0 kB
AnonPages:       3313376 kB
Mapped:           931060 kB
Shmem:            134716 kB
KReclaimable:     427080 kB
Slab:             648316 kB
SReclaimable:     427080 kB
SUnreclaim:       221236 kB
KernelStack:       18752 kB
PageTables:        30576 kB
NFS_Unstable:          0 kB
Bounce:                0 kB
WritebackTmp:          0 kB
CommitLimit:    22316792 kB
Committed_AS:    7944504 kB
VmallocTotal:   34359738367 kB
VmallocUsed:       78600 kB
VmallocChunk:          0 kB
Percpu:            10496 kB
HardwareCorrupted:     0 kB
AnonHugePages:         0 kB
ShmemHugePages:        0 kB
ShmemPmdMapped:        0 kB
FileHugePages:         0 kB
FilePmdMapped:         0 kB
HugePages_Total:       0
HugePages_Free:        0
HugePages_Rsvd:        0
HugePages_Surp:        0
Hugepagesize:       2048 kB
Hugetlb:               0 kB
DirectMap4k:     1696884 kB
DirectMap2M:    17616896 kB
DirectMap1G:    13631488 kB
";
    let result = proc_meminfo(input).unwrap().1;
    assert_eq!(result.len(), 47);
    assert_eq!(
        result.get(&"Buffers".to_string()),
        Some(&ByteSize::kib(4724))
    );
    assert_eq!(
        result.get(&"KReclaimable".to_string()),
        Some(&ByteSize::kib(427080))
    );
}

impl Measurement for MeasurementImpl {
    fn new() -> Self {
        MeasurementImpl
    }

    fn cpu_load(
        &self,
    ) -> std::io::Result<DelayedMeasurement<Vec<SystemCpuLoad>>> {
        cpu_time().map(|times| {
            sleep(Duration::from_secs(1));
            DelayedMeasurement::new(Box::new(move || {
                cpu_time().map(|delay_times| {
                    delay_times
                        .iter()
                        .zip(times.iter())
                        .map(|(now, prev)| (*now - prev).into())
                        .collect::<Vec<_>>()
                })
            }))
        })
    }

    fn cpu_load_by_pid(
        &self,
        pid: u32,
    ) -> std::io::Result<DelayedMeasurement<f64>> {
        todo!()
    }

    fn memory(&self) -> std::io::Result<SystemMemory> {
        PlatformMemory::new().map(PlatformMemory::to_memory)
    }

    fn memory_by_pid(&self, pid: u32) -> std::io::Result<(f64, f64)> {
        todo!()
    }

    fn swap(&self) -> std::io::Result<SystemSwap> {
        PlatformMemory::new().map(PlatformMemory::to_swap)
    }
}

impl PlatformMemory {
    fn new() -> io::Result<Self> {
        memory_stats()
            .or_else(|_| {
                let mut meminfo = BTreeMap::new();
                let mut info: sysinfo = unsafe { mem::zeroed() };
                unsafe { sysinfo(&mut info) };
                let unit = info.mem_unit as u64;
                meminfo.insert(
                    "MemTotal".to_owned(),
                    ByteSize::b(info.totalram as u64 * unit),
                );
                meminfo.insert(
                    "MemFree".to_owned(),
                    ByteSize::b(info.freeram as u64 * unit),
                );
                meminfo.insert(
                    "Shmem".to_owned(),
                    ByteSize::b(info.sharedram as u64 * unit),
                );
                meminfo.insert(
                    "Buffers".to_owned(),
                    ByteSize::b(info.bufferram as u64 * unit),
                );
                meminfo.insert(
                    "SwapTotal".to_owned(),
                    ByteSize::b(info.totalswap as u64 * unit),
                );
                meminfo.insert(
                    "SwapFree".to_owned(),
                    ByteSize::b(info.freeswap as u64 * unit),
                );
                Ok(meminfo)
            })
            .map(|meminfo| PlatformMemory { meminfo })
    }

    fn to_memory(self) -> SystemMemory {
        let meminfo = &self.meminfo;
        SystemMemory {
            total: meminfo.get("MemTotal").copied().unwrap_or(ByteSize::b(0)),
            free: saturating_sub_bytes(
                meminfo.get("MemFree").copied().unwrap_or(ByteSize::b(0))
                    + meminfo.get("Buffers").copied().unwrap_or(ByteSize::b(0))
                    + meminfo.get("Cached").copied().unwrap_or(ByteSize::b(0))
                    + meminfo
                        .get("SReclaimable")
                        .copied()
                        .unwrap_or(ByteSize::b(0)),
                meminfo.get("Shmem").copied().unwrap_or(ByteSize::b(0)),
            ),
            platform: self,
        }
    }

    // Convert the platform memory information to Swap
    fn to_swap(self) -> SystemSwap {
        let meminfo = &self.meminfo;
        SystemSwap {
            total: meminfo.get("SwapTotal").copied().unwrap_or(ByteSize::b(0)),
            free: meminfo.get("SwapFree").copied().unwrap_or(ByteSize::b(0)),
            platform_swap: self,
        }
    }
}
