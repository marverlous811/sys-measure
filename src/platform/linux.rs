use std::{collections::BTreeMap, io, mem, path, str, time::Duration};

use bytesize::ByteSize;
use libc::{statvfs, sysinfo};
use nom::{
    bytes::{
        complete::{tag, take_until},
        take_till,
    },
    character::complete::{digit1, multispace0, not_line_ending, space1},
    combinator::{complete, map, map_res, opt, verify},
    error::ParseError,
    multi::{fold_many0, many1},
    sequence::{delimited, preceded},
    IResult, Parser,
};
use time::OffsetDateTime;

use crate::{
    disk::FileSystem,
    helper::read_file,
    network::{Network, NetworkStats, SocketStats},
    platform::unix,
    process::{ProcessInfo, ProcessStatus},
    saturating_sub_bytes, DelayedMeasurement, Measurement, PlatformMemory,
    SystemCpuLoad, SystemCpuTime, SystemMemory, SystemSwap,
};
pub struct MeasurementImpl;

impl From<&str> for ProcessStatus {
    fn from(status: &str) -> ProcessStatus {
        match status {
            "R" => ProcessStatus::Run,
            "S" => ProcessStatus::Sleep,
            "I" => ProcessStatus::Idle,
            "D" => ProcessStatus::UninterruptibleDiskSleep,
            "Z" => ProcessStatus::Zombie,
            "T" => ProcessStatus::Stop,
            "t" => ProcessStatus::Tracing,
            "X" | "x" => ProcessStatus::Dead,
            "K" => ProcessStatus::Wakekill,
            "W" => ProcessStatus::Waking,
            "P" => ProcessStatus::Parked,
            x => ProcessStatus::Unknown(x.parse().unwrap_or(0)),
        }
    }
}

fn value_from_file<T: str::FromStr>(path: &str) -> io::Result<T> {
    read_file(path)?
        .trim_end_matches('\n')
        .parse()
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("File: \"{}\" doesn't contain an int value", &path),
            )
        })
}

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

#[inline]
pub fn is_space(chr: u8) -> bool {
    chr == b' ' || chr == b'\t'
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

fn word_s(input: &str) -> IResult<&str, &str> {
    take_till(|c| is_space(c as u8)).parse(input)
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

fn get_process_cpu_time(input: &str) -> io::Result<(u64, u64)> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() > 22 {
        let utime: u64 = parts[13]
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?; // User mode time
        let stime: u64 = parts[14]
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?; // Kernel mode time
        return Ok((utime, stime));
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "Invalid stat line",
    ))
}

fn proc_cpu_time(pid: u32) -> io::Result<(u64, u64)> {
    read_file(format!("/proc/{pid}/stat").as_str())
        .and_then(|op| get_process_cpu_time(&op))
}

#[test]
fn test_process_cpu_time() {
    let input = "17263 (node) S 909 632 632 0 -1 4194304 609356 221020 1 0 4204 707 174 207 20 0 13 0 2637531 23598481408 62734 18446744073709551615 12062720 41943537 140724753557216 0 0 0 0 16781312 83458 0 0 0 17 2 0 0 0 0 0 90033704 90170560 425885696 140724753558276 140724753558444 140724753558444 140724753559514 0";
    let result = get_process_cpu_time(input).unwrap();
    assert_eq!(result.0, 4204);
    assert_eq!(result.1, 707)
}

fn get_process_status(input: &str) -> io::Result<ProcessInfo> {
    let get_data_fn = |line: &str| -> io::Result<String> {
        line.split_whitespace().nth(1).map_or_else(
            || {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid line format",
                ))
            },
            |d| Ok(d.to_string()),
        )
    };
    let mut retval = ProcessInfo::default();
    for line in input.lines() {
        match line {
            line if line.starts_with("State:") => {
                let status_char = get_data_fn(line)?;
                let status = ProcessStatus::from(status_char.as_str());
                retval = retval.with_state(status);
            }
            line if line.starts_with("VmSize:") => {
                let size_str = get_data_fn(line)?;
                let vm_size = size_str.parse().map_err(|e| {
                    io::Error::new(io::ErrorKind::InvalidData, e)
                })?;
                retval = retval.with_vm_size(vm_size);
            }
            line if line.starts_with("VmRSS:") => {
                let size_str = get_data_fn(line)?;
                let vm_rss = size_str.parse().map_err(|e| {
                    io::Error::new(io::ErrorKind::InvalidData, e)
                })?;
                retval = retval.with_vm_rss(vm_rss);
            }
            line if line.starts_with("RssAnon:") => {
                let size_str = get_data_fn(line)?;
                let rss_anon = size_str.parse().map_err(|e| {
                    io::Error::new(io::ErrorKind::InvalidData, e)
                })?;
                retval = retval.with_rss_anon(rss_anon);
            }
            line if line.starts_with("RssFile:") => {
                let size_str = get_data_fn(line)?;
                let rss_file = size_str.parse().map_err(|e| {
                    io::Error::new(io::ErrorKind::InvalidData, e)
                })?;
                retval = retval.with_rss_file(rss_file);
            }
            line if line.starts_with("RssShmem:") => {
                let size_str = get_data_fn(line)?;
                let rss_shmem = size_str.parse().map_err(|e| {
                    io::Error::new(io::ErrorKind::InvalidData, e)
                })?;
                retval = retval.with_rss_shmem(rss_shmem);
            }
            _ => {}
        }
    }
    return Ok(retval);
}

fn proc_status(pid: u32) -> io::Result<ProcessInfo> {
    read_file(format!("/proc/{pid}/status").as_str())
        .and_then(|op| get_process_status(&op))
}

#[test]
fn test_get_process_status() {
    let input = "Name:   node
Umask:  0022
State:  S (sleeping)
Tgid:   17263
Ngid:   0
Pid:    17263
PPid:   909
TracerPid:      0
Uid:    0       0       0       0
Gid:    0       0       0       0
FDSize: 64
Groups:  
NStgid: 17263
NSpid:  17263
NSpgid: 632
NSsid:  632
VmPeak: 23063084 kB
VmSize: 23041424 kB
VmLck:         0 kB
VmPin:         0 kB
VmHWM:    267636 kB
VmRSS:    253928 kB
RssAnon:          199140 kB
RssFile:           54788 kB
RssShmem:              0 kB
VmData:   300588 kB
VmStk:       992 kB
VmExe:     29184 kB
VmLib:      5308 kB
VmPTE:      4808 kB
VmSwap:        0 kB
HugetlbPages:          0 kB
CoreDumping:    0
THP_enabled:    1
Threads:        13
SigQ:   0/31592
SigPnd: 0000000000000000
ShdPnd: 0000000000000000
SigBlk: 0000000000000000
SigIgn: 0000000001001000
SigCgt: 0000000100014602
CapInh: 0000000000000000
CapPrm: 000001ffffffffff
CapEff: 000001ffffffffff
CapBnd: 000001ffffffffff
CapAmb: 0000000000000000
NoNewPrivs:     0
Seccomp:        0
Seccomp_filters:        0
Speculation_Store_Bypass:       vulnerable
SpeculationIndirectBranch:      always enabled
Cpus_allowed:   f
Cpus_allowed_list:      0-3
Mems_allowed:   00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000001
Mems_allowed_list:      0
voluntary_ctxt_switches:        709093
nonvoluntary_ctxt_switches:     109016";

    let res = get_process_status(input).unwrap();

    assert_eq!(res.vm_size, 23041424);
    assert_eq!(res.vm_rss, 253928);
    assert_eq!(res.state, ProcessStatus::Sleep);
    assert_eq!(res.rss_anon, 199140);
    assert_eq!(res.rss_file, 54788);
    assert_eq!(res.rss_shmem, 0);
}

struct ProcMountsData {
    source: String,
    target: String,
    fstype: String,
}

fn proc_mounts_line(input: &str) -> IResult<&str, ProcMountsData> {
    map(
        (ws(word_s), ws(word_s), ws(word_s)),
        |(source, target, fstype)| ProcMountsData {
            source: source.to_string(),
            target: target.to_string(),
            fstype: fstype.to_string(),
        },
    )
    .parse(input)
}

fn proc_mounts(input: &str) -> IResult<&str, Vec<ProcMountsData>> {
    many1(map_res(ws(not_line_ending), |input| {
        if input.is_empty() {
            Err(())
        } else {
            proc_mounts_line(input).map(|(_, res)| res).map_err(|_| ())
        }
    }))
    .parse(input)
}

#[test]
fn test_proc_mounts() {
    let test_input_1 = r#"/dev/md0 / btrfs rw,noatime,space_cache,subvolid=15192,subvol=/var/lib/docker/btrfs/subvolumes/df6eb8d3ce1a295bcc252e51ba086cb7705a046a79a342b74729f3f738129f04 0 0
proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0
tmpfs /dev tmpfs rw,nosuid,size=65536k,mode=755,inode64 0 0
devpts /dev/pts devpts rw,nosuid,noexec,relatime,gid=5,mode=620,ptmxmode=666 0 0
sysfs /sys sysfs ro,nosuid,nodev,noexec,relatime 0 0
tmpfs /sys/fs/cgroup tmpfs rw,nosuid,nodev,noexec,relatime,mode=755,inode64 0 0"#;
    let mounts = proc_mounts(test_input_1).unwrap().1;
    assert!(mounts.len() == 6);
    let root = mounts.iter().find(|m| m.target == "/").unwrap();
    assert!(root.source == "/dev/md0");
    assert!(root.target == "/");
    assert!(root.fstype == "btrfs");
}

#[derive(Debug, Default)]
struct ProcNetSockStat {
    tcp_in_use: usize,
    tcp_orphaned: usize,
    tcp_time_wait: usize,
    udp_in_use: usize,
}

#[allow(dead_code)]
fn proc_net_sockstat(input: &str) -> IResult<&str, ProcNetSockStat> {
    map(
        preceded(
            not_line_ending,
            (
                preceded(ws(tag("TCP: inuse")), num),
                preceded(ws(tag("orphan")), num),
                delimited(ws(tag("tw")), num, not_line_ending),
                preceded(ws(tag("UDP: inuse")), num),
            ),
        ),
        |(tcp_in_use, tcp_orphaned, tcp_time_wait, udp_in_use)| {
            ProcNetSockStat {
                tcp_in_use,
                tcp_orphaned,
                tcp_time_wait,
                udp_in_use,
            }
        },
    )
    .parse(input)
}

fn stat_mount(mount: ProcMountsData) -> io::Result<FileSystem> {
    let mut info = unsafe { mem::zeroed::<libc::statvfs>() };
    let target = format!("{}\0", mount.target);
    let result = unsafe { statvfs(target.as_ptr() as *const i8, &mut info) };
    match result {
        0 => Ok(FileSystem {
            files: (info.f_files as usize)
                .saturating_sub(info.f_ffree as usize),
            files_total: info.f_files as usize,
            files_avail: info.f_favail as usize,
            free: ByteSize::b(info.f_bfree as u64 * info.f_bsize as u64),
            avail: ByteSize::b(info.f_bavail as u64 * info.f_bsize as u64),
            total: ByteSize::b(info.f_blocks as u64 * info.f_bsize as u64),
            name_max: info.f_namemax as usize,
            fs_type: mount.fstype,
            fs_mounted_from: mount.source,
            fs_mounted_on: mount.target,
        }),
        _ => Err(io::Error::last_os_error()),
    }
}

#[test]
fn test_proc_net_sockstat() {
    let input = "sockets: used 925
TCP: inuse 20 orphan 0 tw 12 alloc 23 mem 2
UDP: inuse 1 mem 2
UDPLITE: inuse 0
RAW: inuse 0
FRAG: inuse 0 memory 0
";
    let result = proc_net_sockstat(input).unwrap().1;
    println!("{:?}", result);
    assert_eq!(result.tcp_in_use, 20);
    assert_eq!(result.tcp_orphaned, 0);
    assert_eq!(result.tcp_time_wait, 12);
    assert_eq!(result.udp_in_use, 1);
}

struct ProcNetSockStat6 {
    tcp_in_use: usize,
    udp_in_use: usize,
}

#[allow(dead_code)]
fn proc_net_sockstat6(input: &str) -> IResult<&str, ProcNetSockStat6> {
    map(
        ws((
            preceded(ws(tag("TCP6: inuse")), num),
            preceded(ws(tag("UDP6: inuse")), num),
        )),
        |(tcp_in_use, udp_in_use)| ProcNetSockStat6 {
            tcp_in_use,
            udp_in_use,
        },
    )
    .parse(input)
}

#[test]
fn test_proc_net_sockstat6() {
    let input = "TCP6: inuse 3
UDP6: inuse 1
UDPLITE6: inuse 0
RAW6: inuse 1
FRAG6: inuse 0 memory 0
";
    let result = proc_net_sockstat6(input).unwrap().1;
    assert_eq!(result.tcp_in_use, 3);
    assert_eq!(result.udp_in_use, 1);
}

struct TcpSockStat {
    in_use: usize,
    orphaned: usize,
    time_wait: usize,
}

impl Default for TcpSockStat {
    fn default() -> Self {
        TcpSockStat {
            in_use: 0,
            orphaned: 0,
            time_wait: 0,
        }
    }
}

impl From<TcpSockStat> for ProcNetSockStat {
    fn from(stat: TcpSockStat) -> Self {
        ProcNetSockStat {
            tcp_in_use: stat.in_use,
            tcp_orphaned: stat.orphaned,
            tcp_time_wait: stat.time_wait,
            ..Default::default()
        }
    }
}

fn parse_tcp_state(line: &str) -> Option<u8> {
    let parts = line.split_whitespace().collect::<Vec<&str>>();
    if parts.len() < 4 {
        return None;
    }
    u8::from_str_radix(parts[3], 16).ok()
}

fn tcp_sock_from_raw(content: &str) -> std::io::Result<TcpSockStat> {
    let mut stat = TcpSockStat::default();
    for line in content.lines().skip(1) {
        if let Some(state) = parse_tcp_state(line) {
            // stat.in_use += 1;
            match state {
                0x06 => stat.time_wait += 1, // TIME_WAIT
                0x0B => stat.orphaned += 1,  // CLOSE_WAIT (as an example)
                _ => {
                    stat.in_use += 1;
                }
            }
        }
    }

    Ok(stat)
}

#[test]
fn test_tcp_sock_raw() {
    let content = r#"sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode                                                     
   0: 00000000:1392 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 21291 1 ffff967b6f8c2300 100 0 0 10 0                     
   1: 3500007F:0035 00000000:0000 0A 00000000:00000000 00:00000000 00000000   101        0 22069 1 ffff967b68b8cec0 100 0 0 10 0                     
   2: 00000000:0016 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 26714 1 ffff967b6f7571c0 100 0 0 10 0                     
   3: 0100007F:4E20 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 90595452 1 ffff967944b6bd40 100 0 0 10 0                  
   4: 00000000:1FA4 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 50557766 1 ffff967b6f69f1c0 100 0 0 10 0                  
   5: 0100007F:9C45 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 96605041 1 ffff9679cd933d40 100 0 0 10 0                  
   6: 0100007F:2710 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 381283 1 ffff967b69a73d40 100 0 0 10 0                    
   7: 00000000:6DB0 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 30879 1 ffff967b68b8d780 100 0 0 10 0                     
   8: 4F000A0A:E9E8 8FDE3967:8236 06 00000000:00000000 03:00000215 00000000     0        0 0 3 ffff967b6c31f740                                      
   9: 4F000A0A:0016 01000A0A:D8E7 01 00000000:00000000 02:0006D744 00000000     0        0 96592557 2 ffff967b6f8c71c0 21 4 28 5 4                   
  10: 4F000A0A:D168 1672528C:01BB 01 00000000:00000000 02:00000780 00000000  1000        0 96678604 2 ffff96796f0b0000 53 4 24 10 -1                 
  11: 4F000A0A:0016 01000A0A:EC8F 01 00000024:00000000 01:00000014 00000000     0        0 96540557 4 ffff967b6f8c1180 21 4 31 7 7                   
  12: 4F000A0A:A27A 4077FA14:01BB 01 00000000:00000000 02:00000439 00000000  1000        0 96622127 2 ffff96796f0b71c0 41 4 1 10 48                  
  13: 4F000A0A:9532 4077FA14:01BB 01 00000000:00000000 02:0000075B 00000000  1000        0 96671498 2 ffff967a6e211180 40 4 25 13 -1                 
  14: 4F000A0A:ACEA 8FDE3967:8236 06 00000000:00000000 03:00001478 00000000     0        0 0 3 ffff967a46fe22e8                                      
  15: 0100007F:B87E 0100007F:9C45 01 00000000:00000000 00:00000000 00000000  1000        0 96620248 1 ffff9679cd934ec0 21 4 30 10 16                 
  16: 0100007F:9C45 0100007F:B87E 01 00000000:00000000 00:00000000 00000000  1000        0 96620250 1 ffff967b6f752300 21 4 0 10 16                  
  17: 0100007F:6DB0 0100007F:E14E 06 00000000:00000000 03:00000C5E 00000000     0        0 0 3 ffff967b2563bb20                                      
  18: 0100007F:6DB0 0100007F:BD34 06 00000000:00000000 03:00000B89 00000000     0        0 0 3 ffff967b25639550                                      
  19: 0100007F:6DB0 0100007F:E152 06 00000000:00000000 03:00000C61 00000000     0        0 0 3 ffff967b2563ad90                                      
  20: 4F000A0A:B412 2916ED04:01BB 01 00000000:00000000 02:0000090E 00000000  1000        0 96625872 2 ffff967a6e213480 36 4 1 10 24                  
  21: 4F000A0A:D43C 69825514:01BB 01 00000000:00000000 02:0000093F 00000000  1000        0 96682339 2 ffff9679cd939a40 55 4 12 14 -1"#;
    let res = tcp_sock_from_raw(content).unwrap();
    assert_eq!(res.in_use, 17);
    assert_eq!(res.orphaned, 0);
    assert_eq!(res.time_wait, 5);
}

fn udp_sock_from_raw(content: &str) -> std::io::Result<usize> {
    let mut in_use = 0;
    for line in content.lines().skip(1) {
        let parts = line.split_whitespace().collect::<Vec<&str>>();
        if parts.len() >= 4 {
            in_use += 1;
        }
    }
    Ok(in_use)
}

#[test]
fn test_udp_sock_raw() {
    let content = r#" sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode ref pointer drops            
382: 00000000:EB90 00000000:0000 07 00000000:00000000 00:00000000 00000000     0        0 90696615 2 ffff9679c4dc9b00 0      
1038: 0100007F:4E20 00000000:0000 07 00000000:00000000 00:00000000 00000000     0        0 90595453 2 ffff967942300480 0      
1571: 3500007F:0035 00000000:0000 07 00000000:00000000 00:00000000 00000000   101        0 22068 2 ffff967b626e3180 0         
2104: 00000000:E24A 00000000:0000 07 00000000:00000000 00:00000000 00000000     0        0 92612858 2 ffff967a6c186300 0      
3326: 0100007F:2710 00000000:0000 07 00000000:00000000 00:00000000 00000000     0        0 381284 2 ffff967b69441b00 0        
4092: 00000000:CA0E 00000000:0000 07 00000000:00000000 00:00000000 00000000     0        0 95211316 2 ffff9679858b1680 0"#;
    let res = udp_sock_from_raw(content).unwrap();
    assert_eq!(res, 6);
}

fn proc_sockstat_from_raw() -> io::Result<ProcNetSockStat> {
    let tcp_content = read_file("/proc/net/tcp")?;
    let tcp_stat = tcp_sock_from_raw(&tcp_content)?;
    let mut retval: ProcNetSockStat = tcp_stat.into();

    let udp_content = read_file("/proc/net/udp")?;
    let udp_in_use = udp_sock_from_raw(&udp_content)?;
    retval.udp_in_use = udp_in_use;

    Ok(retval)
}

#[test]
#[ignore]
fn test_proc_raw() {
    let res = proc_sockstat_from_raw().unwrap();
    let sockstats = read_file("/proc/net/sockstat")
        .and_then(|data| {
            proc_net_sockstat(&data).map(|(_, res)| res).map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, e.to_string())
            })
        })
        .unwrap();

    assert_eq!(res.tcp_in_use, sockstats.tcp_in_use);
    assert_eq!(res.tcp_orphaned, sockstats.tcp_orphaned);
    assert_eq!(res.tcp_time_wait, sockstats.tcp_time_wait);
    assert_eq!(res.udp_in_use, sockstats.udp_in_use);
}

fn proc_sockstat6_from_raw() -> io::Result<ProcNetSockStat6> {
    let tcp_content = read_file("/proc/net/tcp6")?;
    let tcp_in_use = tcp_content.lines().skip(1).count();

    let udp_content = read_file("/proc/net/udp6")?;
    let udp_in_use = udp_content.lines().skip(1).count();

    Ok(ProcNetSockStat6 {
        tcp_in_use,
        udp_in_use,
    })
}

#[test]
#[ignore]
fn test_proc_sockstat6_raw() {
    let res = proc_sockstat6_from_raw().unwrap();
    let sockstats6 = read_file("/proc/net/sockstat6")
        .and_then(|data| {
            proc_net_sockstat6(&data).map(|(_, res)| res).map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, e.to_string())
            })
        })
        .unwrap();
    assert_eq!(res.tcp_in_use, sockstats6.tcp_in_use);
    assert_eq!(res.udp_in_use, sockstats6.udp_in_use);
}

impl Measurement for MeasurementImpl {
    fn new() -> Self {
        MeasurementImpl
    }

    fn cpu_load(
        &self,
    ) -> std::io::Result<DelayedMeasurement<Vec<SystemCpuLoad>>> {
        cpu_time().map(|times| {
            DelayedMeasurement::new(
                Box::new(move || {
                    cpu_time().map(|delay_times| {
                        delay_times
                            .iter()
                            .zip(times.iter())
                            .map(|(now, prev)| (*now - prev).into())
                            .collect::<Vec<_>>()
                    })
                }),
                None,
            )
        })
    }

    fn cpu_load_by_pid(
        &self,
        pid: u32,
    ) -> std::io::Result<DelayedMeasurement<f64>> {
        let total_core = cpu_time().iter().len();
        let clock_ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as f64;
        proc_cpu_time(pid).map(|(utime, stime)| {
            DelayedMeasurement::new(
                Box::new(move || {
                    proc_cpu_time(pid).map(|(delayed_utime, delayed_stime)| {
                        log::debug!("before: {utime} {stime}");
                        log::debug!("after: {delayed_utime} {delayed_stime}");

                        let used_time = delayed_utime
                            .saturating_sub(utime)
                            .saturating_add(delayed_stime.saturating_sub(stime))
                            as f64
                            / clock_ticks;
                        // default delay measure is 1 sec
                        (used_time * 100.0f64) / total_core as f64
                    })
                }),
                None,
            )
        })
    }

    fn memory(&self) -> std::io::Result<SystemMemory> {
        PlatformMemory::new().map(PlatformMemory::to_memory)
    }

    fn memory_by_pid(&self, pid: u32) -> std::io::Result<(u64, u64)> {
        let status = proc_status(pid)?;
        Ok((status.vm_rss, status.vm_size))
    }

    fn swap(&self) -> std::io::Result<SystemSwap> {
        PlatformMemory::new().map(PlatformMemory::to_swap)
    }

    fn mounts(&self) -> io::Result<Vec<FileSystem>> {
        read_file("/proc/mounts")
            .and_then(|data| {
                proc_mounts(&data).map(|(_, mounts)| mounts).map_err(|e| {
                    io::Error::new(io::ErrorKind::InvalidData, e.to_string())
                })
            })
            .and_then(|mounts| {
                let res = mounts
                    .into_iter()
                    .filter_map(|mount| stat_mount(mount).ok())
                    .collect::<Vec<_>>();
                Ok(res)
            })
    }

    fn mount_at<P: AsRef<path::Path>>(
        &self,
        path: P,
    ) -> io::Result<FileSystem> {
        read_file("/proc/mounts")
            .and_then(|data| {
                proc_mounts(&data).map(|(_, res)| res).map_err(|err| {
                    io::Error::new(io::ErrorKind::InvalidData, err.to_string())
                })
            })
            .and_then(|mounts| {
                mounts
                    .into_iter()
                    .find(|mount| {
                        path::Path::new(&mount.target) == path.as_ref()
                    })
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::NotFound, "No such mount")
                    })
            })
            .and_then(stat_mount)
    }

    fn networks(&self) -> io::Result<BTreeMap<String, Network>> {
        unix::networks()
    }

    fn network_stats(&self, interface: &str) -> io::Result<NetworkStats> {
        let path_root: String =
            ("/sys/class/net/".to_string() + interface) + "/statistics/";
        let stats_file = |file: &str| (&path_root).to_string() + file;

        let rx_bytes: u64 = value_from_file::<u64>(&stats_file("rx_bytes"))?;
        let tx_bytes: u64 = value_from_file::<u64>(&stats_file("tx_bytes"))?;
        let rx_packets: u64 =
            value_from_file::<u64>(&stats_file("rx_packets"))?;
        let tx_packets: u64 =
            value_from_file::<u64>(&stats_file("tx_packets"))?;
        let rx_errors: u64 = value_from_file::<u64>(&stats_file("rx_errors"))?;
        let tx_errors: u64 = value_from_file::<u64>(&stats_file("tx_errors"))?;

        Ok(NetworkStats {
            rx_bytes: ByteSize::b(rx_bytes),
            tx_bytes: ByteSize::b(tx_bytes),
            rx_packets,
            tx_packets,
            rx_errors,
            tx_errors,
        })
    }

    fn socket_stats(&self) -> io::Result<SocketStats> {
        let sockstats = proc_sockstat_from_raw()?;
        let sockstats6 = proc_sockstat6_from_raw()?;

        Ok(SocketStats {
            tcp_sockets_in_use: sockstats.tcp_in_use,
            tcp_sockets_orphan: sockstats.tcp_orphaned,
            tcp_sockets_time_wait: sockstats.tcp_time_wait,
            udp_sockets_in_use: sockstats.udp_in_use,
            tcp6_sockets_in_use: sockstats6.tcp_in_use,
            udp6_sockets_in_use: sockstats6.udp_in_use,
        })
    }

    fn boot_time(&self) -> io::Result<time::OffsetDateTime> {
        read_file("/proc/stat").and_then(|data| {
            data.lines()
                .find(|line| line.starts_with("btime "))
                .ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "btime not found in /proc/stat",
                ))
                .and_then(|line| {
                    let timestamp_str = line
                        .strip_prefix("btime ")
                        .expect("line starts with btime");
                    timestamp_str
                        .parse::<i64>()
                        .map_err(|err| {
                            io::Error::new(
                                io::ErrorKind::InvalidData,
                                err.to_string(),
                            )
                        })
                        .and_then(|ts| {
                            OffsetDateTime::from_unix_timestamp(ts).map_err(
                                |err| {
                                    io::Error::new(
                                        io::ErrorKind::InvalidData,
                                        err.to_string(),
                                    )
                                },
                            )
                        })
                })
        })
    }

    fn process_uptime(&self, pid: u32) -> io::Result<std::time::Duration> {
        let uptime_content = read_file("/proc/uptime")?;
        let system_uptime_secs: f64 = uptime_content
            .split_whitespace()
            .next()
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid /proc/uptime format",
            ))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let process_stat = read_file(format!("/proc/{}/stat", pid).as_str())?;
        let parts: Vec<&str> = process_stat.split_whitespace().collect();
        let start_time_ticks: u64 = parts
            .get(21)
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid /proc/[pid]/stat format",
            ))?
            .parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let clock_ticks_per_second =
            unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as f64;

        let process_start_time_secs =
            start_time_ticks as f64 / clock_ticks_per_second;
        let process_uptime_secs = system_uptime_secs - process_start_time_secs;
        Ok(Duration::from_secs_f64(process_uptime_secs.max(0.0)))
    }

    fn process_pid(&self, cmd: &str) -> io::Result<Vec<usize>> {
        let mut pids = Vec::new();
        for entry in std::fs::read_dir("/proc")? {
            let entry = entry?;
            let pid_str = entry.file_name().to_string_lossy().to_string();
            if let Ok(pid) = pid_str.parse::<usize>() {
                let cmd_path = format!("/proc/{}/cmdline", pid);
                if let Ok(cmdline) = read_file(&cmd_path) {
                    if cmdline.contains(cmd) {
                        pids.push(pid);
                    }
                }
            }
        }

        Ok(pids)
    }

    fn process_status(&self, pid: u32) -> io::Result<ProcessInfo> {
        proc_status(pid)
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
            free: meminfo.get("MemFree").copied().unwrap_or(ByteSize::b(0)),
            used: saturating_sub_bytes(
                meminfo.get("MemTotal").copied().unwrap_or(ByteSize::b(0)),
                meminfo.get("MemFree").copied().unwrap_or(ByteSize::b(0))
                    + meminfo.get("Buffers").copied().unwrap_or(ByteSize::b(0))
                    + meminfo.get("Cached").copied().unwrap_or(ByteSize::b(0)),
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
