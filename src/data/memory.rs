use bytesize::ByteSize;

#[derive(Debug, Clone)]
pub struct SystemMemory {
    pub total: ByteSize,
    pub free: ByteSize,
    pub platform: PlatformMemory,
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub struct PlatformMemory {
    pub total: ByteSize,
    pub free: ByteSize,
    pub active: ByteSize,
    pub inactive: ByteSize,
    pub wired: ByteSize,
    pub compressor: ByteSize,
}

#[cfg(target_os = "linux")]
pub use std::collections::BTreeMap;

#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
pub struct PlatformMemory {
    pub meminfo: BTreeMap<String, ByteSize>,
}

#[derive(Debug, Clone)]
pub struct SystemSwap {
    pub total: ByteSize,
    pub free: ByteSize,
    pub platform_swap: PlatformSwap,
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub struct PlatformSwap {
    pub total: ByteSize,
    pub avail: ByteSize,
}

#[cfg(target_os = "linux")]
pub type PlatformSwap = PlatformMemory;
