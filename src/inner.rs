//! Internal utilities
use std::ffi::{CStr, CString};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use std::string::ToString;
use uv::{uv_ip4_addr, uv_ip4_name, uv_ip6_addr, uv_ip6_name, AF_INET, AF_INET6};

/// An internal version of From<T>
#[doc(hidden)]
pub(crate) trait FromInner<T>: Sized {
    fn from_inner(_: T) -> Self;
}

/// An internal version of Into<T>
#[doc(hidden)]
pub(crate) trait IntoInner<T>: Sized {
    fn into_inner(self) -> T;
}

/// An internal version of TryFrom<T>
#[doc(hidden)]
pub(crate) trait TryFromInner<T>: Sized {
    type Error;
    fn try_from_inner(_: T) -> Result<Self, Self::Error>;
}

/// An internal version of TryInto<T>
#[doc(hidden)]
pub(crate) trait TryIntoInner<T>: Sized {
    type Error;
    fn try_into_inner(self) -> Result<T, Self::Error>;
}

// FromInner implies IntoInner
impl<T, U> IntoInner<U> for T
where
    U: FromInner<T>,
{
    fn into_inner(self) -> U {
        U::from_inner(self)
    }
}

// FromInner (and thus IntoInner) is reflexive
impl<T> FromInner<T> for T {
    fn from_inner(t: T) -> T {
        t
    }
}

// TryFromInner implies TryIntoInner
impl<T, U> TryIntoInner<U> for T
where
    U: TryFromInner<T>,
{
    type Error = U::Error;

    fn try_into_inner(self) -> Result<U, U::Error> {
        U::try_from_inner(self)
    }
}

// Infallible conversions are semantically equivalent to fallible conversions with an unihabited
// error type
impl<T, U> TryFromInner<U> for T
where
    U: IntoInner<T>,
{
    type Error = std::convert::Infallible;

    fn try_from_inner(value: U) -> Result<Self, Self::Error> {
        Ok(U::into_inner(value))
    }
}

/// Many structs are thin wrappers around structs from libuv_sys2 - the Inner trait extracts the
/// wrapped struct.
#[doc(hidden)]
pub(crate) trait Inner<T>: Sized {
    fn inner(&self) -> T;
}

/// Inner lifts over &
#[doc(hidden)]
impl<T, U> Inner<U> for &T
where
    T: Inner<U>,
{
    fn inner(&self) -> U {
        <T as Inner<U>>::inner(*self)
    }
}

/// Inner lefts over &mut
#[doc(hidden)]
impl<T, U> Inner<U> for &mut T
where
    T: Inner<U>,
{
    fn inner(&self) -> U {
        <T as Inner<U>>::inner(*self)
    }
}

/// Fill a uv::sockaddr from a SocketAddr
pub(crate) fn fill_sockaddr(
    sockaddr: *mut uv::sockaddr,
    addr: &SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let s = addr.ip().to_string();
    let s = CString::new(s)?;
    match addr {
        SocketAddr::V4(addr) => {
            let sockaddr_in: *mut uv::sockaddr_in = sockaddr as _;
            crate::uvret(unsafe { uv_ip4_addr(s.as_ptr(), addr.port() as _, sockaddr_in) })
                .map_err(|e| Box::new(e) as _)
        }
        SocketAddr::V6(addr) => {
            let sockaddr_in6: *mut uv::sockaddr_in6 = sockaddr as _;
            crate::uvret(unsafe { uv_ip6_addr(s.as_ptr(), addr.port() as _, sockaddr_in6) })
                .map_err(|e| Box::new(e) as _)
        }
    }
}

/// Create a SocketAddr from a uv::sockaddr_storage
pub(crate) fn build_socketaddr(
    sockaddr: *const uv::sockaddr,
) -> Result<SocketAddr, Box<dyn std::error::Error>> {
    // sockaddr_in/sockaddr_in6 port are in network byte order, which is big endian. So, we need to
    // make sure to convert to "native endianness" (ne).
    match unsafe { (*sockaddr).sa_family as _ } {
        AF_INET => {
            let sockaddr_in: *const uv::sockaddr_in = sockaddr as _;
            let mut buf: Vec<std::os::raw::c_uchar> = Vec::with_capacity(16);
            unsafe {
                let port = u16::from_be((*sockaddr_in).sin_port) as _;
                buf.set_len(16);
                crate::uvret(uv_ip4_name(sockaddr_in, buf.as_mut_ptr() as _, 16))?;
                let s = CStr::from_bytes_with_nul_unchecked(&buf).to_string_lossy();
                let addr = Ipv4Addr::from_str(s.as_ref())?;
                Ok(SocketAddr::new(IpAddr::V4(addr), port))
            }
        }
        AF_INET6 => {
            let sockaddr_in6: *const uv::sockaddr_in6 = sockaddr as _;
            let mut buf: Vec<std::os::raw::c_uchar> = Vec::with_capacity(46);
            unsafe {
                let port = u16::from_be((*sockaddr_in6).sin6_port) as _;
                buf.set_len(46);
                crate::uvret(uv_ip6_name(sockaddr_in6, buf.as_mut_ptr() as _, 46))?;
                let s = CStr::from_bytes_with_nul_unchecked(&buf).to_string_lossy();
                let addr = Ipv6Addr::from_str(s.as_ref())?;
                Ok(SocketAddr::new(IpAddr::V6(addr), port))
            }
        }
        _ => Err(Box::new(crate::Error::ENOTSUP)),
    }
}
