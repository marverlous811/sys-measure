use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    Idle,
    Run,
    Sleep,
    Stop,
    Zombie,
    Tracing,
    Dead,
    Wakekill,
    Waking,
    Parked,
    LockBlocked,
    UninterruptibleDiskSleep,
    Suspended,
    Unknown(u32),
}

impl fmt::Display for ProcessStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            ProcessStatus::Idle => "Idle",
            ProcessStatus::Run => "Running",
            ProcessStatus::Sleep => "Sleeping",
            ProcessStatus::Stop => "Stopped",
            ProcessStatus::Zombie => "Zombie",
            ProcessStatus::Tracing => "Tracing",
            ProcessStatus::Dead => "Dead",
            ProcessStatus::Wakekill => "Wakekill",
            ProcessStatus::Waking => "Waking",
            ProcessStatus::Parked => "Parked",
            ProcessStatus::UninterruptibleDiskSleep => {
                "UninterruptibleDiskSleep"
            }
            _ => "Unknown",
        })
    }
}

impl Into<u64> for ProcessStatus {
    fn into(self) -> u64 {
        match self {
            ProcessStatus::Idle => (0 << 32) | 0,
            ProcessStatus::Run => (1 << 32) | 0,
            ProcessStatus::Sleep => (2 << 32) | 0,
            ProcessStatus::Stop => (3 << 32) | 0,
            ProcessStatus::Zombie => (4 << 32) | 0,
            ProcessStatus::Tracing => (5 << 32) | 0,
            ProcessStatus::Dead => (6 << 32) | 0,
            ProcessStatus::Wakekill => (7 << 32) | 0,
            ProcessStatus::Waking => (8 << 32) | 0,
            ProcessStatus::Parked => (9 << 32) | 0,
            ProcessStatus::LockBlocked => (10 << 32) | 0,
            ProcessStatus::UninterruptibleDiskSleep => (11 << 32) | 0,
            ProcessStatus::Suspended => (12 << 32) | 0,
            ProcessStatus::Unknown(code) => (13 << 32) | code as u64,
        }
    }
}

impl From<u64> for ProcessStatus {
    fn from(value: u64) -> Self {
        let high = (value >> 32) as u32;
        let low = (value & 0xFFFF_FFFF) as u32;
        match high {
            0 => ProcessStatus::Idle,
            1 => ProcessStatus::Run,
            2 => ProcessStatus::Sleep,
            3 => ProcessStatus::Stop,
            4 => ProcessStatus::Zombie,
            5 => ProcessStatus::Tracing,
            6 => ProcessStatus::Dead,
            7 => ProcessStatus::Wakekill,
            8 => ProcessStatus::Waking,
            9 => ProcessStatus::Parked,
            10 => ProcessStatus::LockBlocked,
            11 => ProcessStatus::UninterruptibleDiskSleep,
            12 => ProcessStatus::Suspended,
            _ => ProcessStatus::Unknown(low),
        }
    }
}

#[test]
fn test_process_status() {
    let statuses = [
        ProcessStatus::Idle,
        ProcessStatus::Run,
        ProcessStatus::Sleep,
        ProcessStatus::Stop,
        ProcessStatus::Zombie,
        ProcessStatus::Tracing,
        ProcessStatus::Dead,
        ProcessStatus::Wakekill,
        ProcessStatus::Waking,
        ProcessStatus::Parked,
        ProcessStatus::LockBlocked,
        ProcessStatus::UninterruptibleDiskSleep,
        ProcessStatus::Suspended,
        ProcessStatus::Unknown(42),
    ];

    for status in &statuses {
        let encoded: u64 = (*status).into();
        // println!("Encoded {:?} into {}", status, encoded);
        let decoded: ProcessStatus = encoded.into();
        assert_eq!(*status, decoded);
    }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub state: ProcessStatus,
    pub vm_size: u64,
    pub vm_rss: u64,
    pub rss_anon: u64,
    pub rss_file: u64,
    pub rss_shmem: u64,
}

impl ProcessInfo {
    pub fn with_state(mut self, state: ProcessStatus) -> Self {
        self.state = state;
        self
    }

    pub fn with_vm_rss(mut self, vm_rss: u64) -> Self {
        self.vm_rss = vm_rss;
        self
    }

    pub fn with_rss_anon(mut self, rss_anon: u64) -> Self {
        self.rss_anon = rss_anon;
        self
    }

    pub fn with_rss_file(mut self, rss_file: u64) -> Self {
        self.rss_file = rss_file;
        self
    }

    pub fn with_rss_shmem(mut self, rss_shmem: u64) -> Self {
        self.rss_shmem = rss_shmem;
        self
    }

    pub fn with_vm_size(mut self, vm_size: u64) -> Self {
        self.vm_size = vm_size;
        self
    }
}

impl Default for ProcessInfo {
    fn default() -> Self {
        ProcessInfo {
            state: ProcessStatus::Unknown(0),
            vm_size: 0,
            vm_rss: 0,
            rss_anon: 0,
            rss_file: 0,
            rss_shmem: 0,
        }
    }
}
