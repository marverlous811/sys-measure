use bytesize::ByteSize;

#[derive(Debug, Clone)]
pub struct FileSystem {
    /// Used file nodes in filesystem
    pub files: usize,
    /// Total file nodes in filesystem
    pub files_total: usize,
    /// Free nodes available to non-superuser
    pub files_avail: usize,
    /// Free bytes in filesystem
    pub free: ByteSize,
    /// Free bytes available to non-superuser
    pub avail: ByteSize,
    /// Total bytes in filesystem
    pub total: ByteSize,
    /// Maximum filename length
    pub name_max: usize,
    pub fs_type: String,
    pub fs_mounted_from: String,
    pub fs_mounted_on: String,
}

pub struct BlockDeviceStats {
    pub name: String,
    pub read_ios: usize,
    pub read_merges: usize,
    pub read_sectors: usize,
    pub read_ticks: usize,
    pub write_ios: usize,
    pub write_merges: usize,
    pub write_sectors: usize,
    pub write_ticks: usize,
    pub in_flight: usize,
    pub io_ticks: usize,
    pub time_in_queue: usize,
}
