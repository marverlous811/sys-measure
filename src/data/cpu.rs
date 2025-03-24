use std::ops::Sub;

#[derive(Debug, Clone, Copy)]
pub struct SystemCpuTime {
    pub user: usize,
    pub nice: usize,
    pub system: usize,
    pub interrupt: usize,
    pub idle: usize,
    pub other: usize,
}

impl<'a> Sub<&'a SystemCpuTime> for SystemCpuTime {
    type Output = SystemCpuTime;

    #[inline(always)]
    fn sub(self, rhs: &SystemCpuTime) -> SystemCpuTime {
        println!("left: {:?} - right: {:?}", self, rhs);
        SystemCpuTime {
            user: self.user.saturating_sub(rhs.user),
            nice: self.nice.saturating_sub(rhs.nice),
            system: self.system.saturating_sub(rhs.system),
            interrupt: self.interrupt.saturating_sub(rhs.interrupt),
            idle: self.idle.saturating_sub(rhs.idle),
            other: self.other.saturating_sub(rhs.other),
        }
    }
}

impl Into<SystemCpuLoad> for SystemCpuTime {
    fn into(self) -> SystemCpuLoad {
        let total = self.user
            + self.nice
            + self.system
            + self.interrupt
            + self.idle
            + self.other;

        if total == 0 {
            SystemCpuLoad {
                user: 0.0,
                nice: 0.0,
                system: 0.0,
                interrupt: 0.0,
                idle: 0.0,
                platform: PlatformCpuLoad::default(),
            }
        } else {
            SystemCpuLoad {
                user: self.user as f32 / total as f32,
                nice: self.nice as f32 / total as f32,
                system: self.system as f32 / total as f32,
                interrupt: self.interrupt as f32 / total as f32,
                idle: self.idle as f32 / total as f32,
                platform: PlatformCpuLoad::from(
                    self.other as f32 / total as f32,
                ),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemCpuLoad {
    pub user: f32,
    pub nice: f32,
    pub system: f32,
    pub interrupt: f32,
    pub idle: f32,
    pub platform: PlatformCpuLoad,
}

impl SystemCpuLoad {
    #[inline(always)]
    pub fn avg_add(self, rhs: &Self) -> Self {
        SystemCpuLoad {
            user: (self.user + rhs.user) / 2.0,
            nice: (self.nice + rhs.nice) / 2.0,
            system: (self.system + rhs.system) / 2.0,
            interrupt: (self.interrupt + rhs.interrupt) / 2.0,
            idle: (self.idle + rhs.idle) / 2.0,
            platform: self.platform.avg_add(&rhs.platform),
        }
    }
}

#[cfg(not(target_os = "linux"))]
#[derive(Debug, Clone, Default)]
pub struct PlatformCpuLoad {}

impl PlatformCpuLoad {
    #[cfg(not(target_os = "linux"))]
    #[inline(always)]
    pub fn avg_add(self, _rhs: &Self) -> Self {
        self
    }

    #[cfg(not(target_os = "linux"))]
    #[inline(always)]
    pub fn from(_input: f32) -> Self {
        PlatformCpuLoad {}
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Default)]
pub struct PlatformCpuLoad {
    pub iowait: f32,
}

impl PlatformCpuLoad {
    #[cfg(target_os = "linux")]
    #[inline(always)]
    pub fn avg_add(self, rhs: &Self) -> Self {
        PlatformCpuLoad {
            iowait: (self.iowait + rhs.iowait) / 2.0,
        }
    }

    #[cfg(target_os = "linux")]
    #[inline(always)]
    pub fn from(input: f32) -> Self {
        PlatformCpuLoad { iowait: input }
    }
}
