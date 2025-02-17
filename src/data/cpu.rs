use std::ops::Sub;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct SystemCpuLoad {
    pub user: f32,
    pub nice: f32,
    pub system: f32,
    pub interrupt: f32,
    pub idle: f32,
    pub platform: PlatformCpuLoad,
}

#[cfg(not(target_os = "linux"))]
#[derive(Debug, Clone)]
pub struct PlatformCpuLoad {}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
pub struct PlatformCpuLoad {
    pub iowait: f32,
}
