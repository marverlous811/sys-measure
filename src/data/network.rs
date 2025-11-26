use std::net::{Ipv4Addr, Ipv6Addr};

use bytesize::ByteSize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpAddr {
    Empty,
    Unsupported,
    V4(Ipv4Addr),
    V6(Ipv6Addr),
}

#[derive(Debug, Clone)]
pub struct NetworkAddr {
    pub addr: IpAddr,
    pub netmask: IpAddr,
}

#[derive(Debug, Clone)]
pub struct Network {
    pub name: String,
    pub addrs: Vec<NetworkAddr>,
}

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub rx_bytes: ByteSize,
    pub tx_bytes: ByteSize,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

#[derive(Debug, Default, Clone)]
pub struct SocketStats {
    pub tcp_sockets_in_use: usize,
    pub tcp_sockets_orphan: usize,
    pub tcp_sockets_time_wait: usize,
    pub udp_sockets_in_use: usize,
    pub tcp6_sockets_in_use: usize,
    pub udp6_sockets_in_use: usize,
}

impl SocketStats {
    pub fn with_tcp_in_use(mut self, tcp_in_use: usize) -> Self {
        self.tcp_sockets_in_use = tcp_in_use;
        self
    }

    pub fn with_udp_in_use(mut self, udp_in_use: usize) -> Self {
        self.udp_sockets_in_use = udp_in_use;
        self
    }

    pub fn with_tcp6_in_use(mut self, tcp6_in_use: usize) -> Self {
        self.tcp6_sockets_in_use = tcp6_in_use;
        self
    }

    pub fn with_udp6_in_use(mut self, udp6_in_use: usize) -> Self {
        self.udp6_sockets_in_use = udp6_in_use;
        self
    }

    pub fn with_tcp_orphan(mut self, tcp_orphan: usize) -> Self {
        self.tcp_sockets_orphan = tcp_orphan;
        self
    }

    pub fn with_tcp_time_wait(mut self, tcp_time_wait: usize) -> Self {
        self.tcp_sockets_time_wait = tcp_time_wait;
        self
    }
}
