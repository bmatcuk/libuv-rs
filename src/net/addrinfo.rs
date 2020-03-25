use crate::{FromInner, IntoInner};
use uv::addrinfo;

pub struct AddrInfo {
    /// Bitwise OR of uv::AI_* flags
    pub flags: u32,

    /// One of the uv::AF_* constants
    pub family: u32,

    /// One of the uv::SOCK_* constants
    pub socktype: u32,

    /// One of the uv::IPPROTO_* constants
    pub protocol: u32,

    /// The canonical name of the host
    pub canonical_name: Option<String>,
    // pub addr: SocketAddr,
}

impl FromInner<*mut addrinfo> for AddrInfo {
    fn from_inner(info: *mut addrinfo) -> AddrInfo {
        unsafe {
            let canonical_name = if (*info).ai_canonname.is_null() {
                None
            } else {
                Some(
                    std::ffi::CStr::from_ptr((*info).ai_canonname as _)
                        .to_string_lossy()
                        .into_owned(),
                )
            };
            AddrInfo {
                flags: (*info).ai_flags as _,
                family: (*info).ai_family as _,
                socktype: (*info).ai_socktype as _,
                protocol: (*info).ai_protocol as _,
                canonical_name,
            }
        }
    }
}

impl IntoInner<addrinfo> for AddrInfo {
    fn into_inner(self) -> addrinfo {
        let mut ai: addrinfo = unsafe { std::mem::zeroed() };
        ai.ai_flags = self.flags as _;
        ai.ai_family = self.family as _;
        ai.ai_socktype = self.socktype as _;
        ai.ai_protocol = self.protocol as _;
        ai
    }
}
