//! Internal utilities
use std::net::SocketAddr;
use uv::{AF_INET, AF_INET6};

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
pub(crate) fn fill_sockaddr(sockaddr: *mut uv::sockaddr, addr: &SocketAddr) {
    // sockaddr_in/sockaddr_in6 port and addr must be in network byte order, which is big endian.
    // addr.ip().octets() returns the bytes in big endian already so we're good there.
    match addr {
        SocketAddr::V4(addr) => {
            let sockaddr_in: *mut uv::sockaddr_in = sockaddr as _;
            unsafe {
                (*sockaddr_in).sin_family = AF_INET as _;
                (*sockaddr_in).sin_port = addr.port().to_be();
                (*sockaddr_in).sin_addr.s_addr = u32::from_ne_bytes(addr.ip().octets());
            }
        }
        SocketAddr::V6(addr) => {
            let sockaddr_in6: *mut uv::sockaddr_in6 = sockaddr as _;
            unsafe {
                (*sockaddr_in6).sin6_family = AF_INET6 as _;
                (*sockaddr_in6).sin6_port = addr.port().to_be();
                (*sockaddr_in6).sin6_addr.__u6_addr.__u6_addr8 = addr.ip().octets();
            }
        }
    }
}

/// Create a SocketAddr from a uv::sockaddr_storage
pub(crate) fn build_socketaddr(sockaddr: *const uv::sockaddr) -> crate::Result<SocketAddr> {
    // sockaddr_in/sockaddr_in6 port and addr are in network byte order, which is big endian. So,
    // we need to make sure to convert to "native endianness" (ne).
    match unsafe { (*sockaddr).sa_family as _ } {
        AF_INET => {
            let sockaddr_in: *const uv::sockaddr_in = sockaddr as _;
            unsafe {
                Ok((
                    (*sockaddr_in).sin_addr.s_addr.to_ne_bytes(),
                    u16::from_be((*sockaddr_in).sin_port),
                )
                    .into())
            }
        }
        AF_INET6 => {
            let sockaddr_in6: *const uv::sockaddr_in6 = sockaddr as _;
            unsafe {
                Ok((
                    (*sockaddr_in6).sin6_addr.__u6_addr.__u6_addr8,
                    u16::from_be((*sockaddr_in6).sin6_port),
                )
                    .into())
            }
        }
        _ => Err(crate::Error::ENOTSUP),
    }
}
