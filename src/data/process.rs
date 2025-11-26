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
            ProcessStatus::Run => "Runnable",
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
