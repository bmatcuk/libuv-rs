use crate::{FromInner, IntoInner};
use libc::{AF_INET, AF_INET6, AF_UNSPEC};
use std::net::SocketAddr;
use uv::addrinfo;

bitflags! {
    pub struct AddrInfoFlags: i32 {
    }
}

#[repr(i32)]
pub enum AddrFamily {
    INET = AF_INET,
    INET6 = AF_INET6,
    Unspecified = AF_UNSPEC,
}

pub struct AddrInfo {
    pub flags: AddrInfoFlags,
    pub family: AddrFamily,
    pub socktype: AddrSockType,
    pub protocol: AddrProtocol,
    pub canonical_name: Option<String>,
    pub addr: SocketAddr,
}
