use std::{
    collections::BTreeMap,
    io,
    net::{Ipv4Addr, Ipv6Addr},
    ptr,
};

use libc::{freeifaddrs, getifaddrs, ifaddrs, sockaddr, AF_INET, AF_INET6};

use crate::network::{IpAddr, Network, NetworkAddr};

pub fn networks() -> io::Result<BTreeMap<String, Network>> {
    let mut ifap: *mut ifaddrs = ptr::null_mut();
    if unsafe { getifaddrs(&mut ifap) } != 0 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "getifaddrs() failed",
        ));
    }

    let ifirst = ifap;
    let mut result = BTreeMap::new();

    while !ifap.is_null() {
        let ifa = unsafe { *ifap };
        let name = unsafe {
            std::ffi::CStr::from_ptr(ifa.ifa_name)
                .to_string_lossy()
                .into_owned()
        };
        let entry = result.entry(name.clone()).or_insert(Network {
            name,
            addrs: Vec::new(),
        });
        let addr = parse_addr(ifa.ifa_addr);
        if addr != IpAddr::Unsupported {
            entry.addrs.push(NetworkAddr {
                addr,
                netmask: parse_addr(ifa.ifa_netmask),
            });
        }
        ifap = unsafe { (*ifap).ifa_next };
    }

    unsafe { freeifaddrs(ifirst) };
    Ok(result)
}

fn parse_addr(aptr: *const sockaddr) -> IpAddr {
    if aptr.is_null() {
        return IpAddr::Empty;
    }

    let addr = unsafe { *aptr };
    match addr.sa_family as i32 {
        AF_INET => IpAddr::V4(Ipv4Addr::new(
            addr.sa_data[2] as u8,
            addr.sa_data[3] as u8,
            addr.sa_data[4] as u8,
            addr.sa_data[5] as u8,
        )),
        AF_INET6 => {
            let sin6 = unsafe { &*(aptr as *const libc::sockaddr_in6) };
            IpAddr::V6(Ipv6Addr::from(sin6.sin6_addr.s6_addr))
        }
        _ => IpAddr::Unsupported,
    }
}
