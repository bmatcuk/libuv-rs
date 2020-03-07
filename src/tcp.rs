use crate::{FromInner, IntoInner};
use libc::{AF_INET, AF_INET6};
use std::net::SocketAddr;
use uv::{
    uv_ip4_addr, uv_ip6_addr, uv_tcp_bind, uv_tcp_close_reset, uv_tcp_connect, uv_tcp_getpeername,
    uv_tcp_getsockname, uv_tcp_init, uv_tcp_keepalive, uv_tcp_nodelay, uv_tcp_simultaneous_accepts,
    uv_tcp_t,
};

/// Additional data to store on the stream
#[derive(Default)]
pub(crate) struct TcpDataFields {}

bitflags! {
    /// Flags to TcpHandle::bind()
    pub struct TcpBindFlags: u32 {
        /// Dual-stack support is disabled and only IPv6 is used.
        const IPV6ONLY = uv::uv_tcp_flags_UV_TCP_IPV6ONLY;
    }
}

/// Fill a uv::sockaddr from a SocketAddr
fn fill_sockaddr(sockaddr: *mut uv::sockaddr, addr: &SocketAddr) {
    match addr {
        SocketAddr::V4(addr) => {
            let sockaddr_in: *mut uv::sockaddr_in = sockaddr as _;
            (*sockaddr_in).sin_family = AF_INET;
            (*sockaddr_in).sin_port = addr.port();
            (*sockaddr_in).sin_addr.s_addr = u32::from_ne_bytes(addr.ip().octets());
        }
        SocketAddr::V6(addr) => {
            let sockaddr_in6: *mut uv::sockaddr_in6 = sockaddr as _;
            (*sockaddr_in6).sin6_family = AF_INET6;
            (*sockaddr_in6).sin6_port = addr.port();
            (*sockaddr_in6).sin6_addr.__u6_addr.__u6_addr8 = addr.ip().octets();
        }
    }
}

pub struct TcpHandle {
    handle: *mut uv_tcp_t,
}

impl TcpHandle {
    /// Initialize the handle. No socket is created as of yet.
    pub fn new(r#loop: &crate::Loop) -> crate::Result<TcpHandle> {
        let layout = std::alloc::Layout::new::<uv_tcp_t>();
        let handle = unsafe { std::alloc::alloc(layout) as *mut uv_tcp_t };
        if handle.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let ret = unsafe { uv_tcp_init(r#loop.into_inner(), handle) };
        if ret < 0 {
            unsafe { std::alloc::dealloc(handle as _, layout) };
            return Err(crate::Error::from(ret as _));
        }

        crate::StreamHandle::initialize_data(
            uv_handle!(handle),
            crate::TcpData(Default::default()),
        );

        Ok(TcpHandle { handle })
    }

    /// Enable TCP_NODELAY, which disables Nagle’s algorithm.
    pub fn nodelay(&mut self, enable: bool) -> crate::Result<()> {
        crate::uvret(unsafe { uv_tcp_nodelay(self.handle, if enable { 1 } else { 0 }) })
    }

    /// Enable / disable TCP keep-alive. delay is the initial delay in seconds, ignored when enable
    /// is zero.
    ///
    /// After delay has been reached, 10 successive probes, each spaced 1 second from the previous
    /// one, will still happen. If the connection is still lost at the end of this procedure, then
    /// the handle is destroyed with a ETIMEDOUT error passed to the corresponding callback.
    pub fn keepalive(&mut self, enable: bool, delay: u32) -> crate::Result<()> {
        crate::uvret(unsafe { uv_tcp_keepalive(self.handle, if enable { 1 } else { 0 }, delay) })
    }

    /// Enable / disable simultaneous asynchronous accept requests that are queued by the operating
    /// system when listening for new TCP connections.
    ///
    /// This setting is used to tune a TCP server for the desired performance. Having simultaneous
    /// accepts can significantly improve the rate of accepting connections (which is why it is
    /// enabled by default) but may lead to uneven load distribution in multi-process setups.
    pub fn simultaneous_accepts(&mut self, enable: bool) -> crate::Result<()> {
        crate::uvret(unsafe {
            uv_tcp_simultaneous_accepts(self.handle, if enable { 1 } else { 0 })
        })
    }

    /// Bind the handle to an address and port.
    ///
    /// When the port is already taken, you can expect to see an EADDRINUSE error from either
    /// bind(), listen() or connect(). That is, a successful call to this function does not
    /// guarantee that the call to listen() or connect() will succeed as well.
    ///
    /// flags can contain IPV6ONLY, in which case dual-stack support is disabled and only IPv6 is
    /// used.
    pub fn bind(&mut self, addr: &SocketAddr, flags: TcpBindFlags) -> crate::Result<()> {
        let mut sockaddr: uv::sockaddr = std::mem::zeroed();
        fill_sockaddr(&mut sockaddr, addr);
        crate::uvret(unsafe { uv_tcp_bind(self.handle, &sockaddr as _, flags.bits()) })
    }

    /// Get the current address to which the handle is bound.
    pub fn getsockname(&self) -> crate::Result<SocketAddr> {
        let mut sockaddr: uv::sockaddr_storage = std::mem::zeroed();
        let mut sockaddr_len: std::os::raw::c_int =
            std::mem::size_of::<uv::sockaddr_storage>() as _;
        crate::uvret(unsafe {
            uv_tcp_getsockname(
                self.handle,
                uv_handle!(&mut sockaddr),
                &mut sockaddr_len as _,
            )
        })?;

        match sockaddr.ss_family {
            AF_INET => {
                let sockaddr_in: *const uv::sockaddr_in = uv_handle!(&sockaddr);
                Ok((
                    (*sockaddr_in).sin_addr.s_addr.to_ne_bytes(),
                    (*sockaddr_in).sin_port,
                )
                    .into())
            }
            AF_INET6 => {
                let sockaddr_in6: *const uv::sockaddr_in6 = uv_handle!(&sockaddr);
                Ok((
                    (*sockaddr_in6).sin6_addr.__u6_addr.__u6_addr8,
                    (*sockaddr_in6).sin6_port,
                )
                    .into())
            }
            _ => Err(crate::Error::ENOTSUP),
        }
    }

    /// Get the address of the peer connected to the handle.
    pub fn getpeername(&self) -> crate::Result<SocketAddr> {
        let mut sockaddr: uv::sockaddr_storage = std::mem::zeroed();
        let mut sockaddr_len: std::os::raw::c_int =
            std::mem::size_of::<uv::sockaddr_storage>() as _;
        crate::uvret(unsafe {
            uv_tcp_getpeername(
                self.handle,
                uv_handle!(&mut sockaddr),
                &mut sockaddr_len as _,
            )
        })?;

        match sockaddr.ss_family {
            AF_INET => {
                let sockaddr_in: *const uv::sockaddr_in = uv_handle!(&sockaddr);
                Ok((
                    (*sockaddr_in).sin_addr.s_addr.to_ne_bytes(),
                    (*sockaddr_in).sin_port,
                )
                    .into())
            }
            AF_INET6 => {
                let sockaddr_in6: *const uv::sockaddr_in6 = uv_handle!(&sockaddr);
                Ok((
                    (*sockaddr_in6).sin6_addr.__u6_addr.__u6_addr8,
                    (*sockaddr_in6).sin6_port,
                )
                    .into())
            }
            _ => Err(crate::Error::ENOTSUP),
        }
    }

    /// Establish an IPv4 or IPv6 TCP connection.
    ///
    /// On Windows if the addr is initialized to point to an unspecified address (0.0.0.0 or ::) it
    /// will be changed to point to localhost. This is done to match the behavior of Linux systems.
    ///
    /// The callback is made when the connection has been established or when a connection error
    /// happened.
    pub fn connect(
        &mut self,
        addr: &SocketAddr,
        cb: Option<impl FnMut(crate::ConnectReq, i32) + 'static>,
    ) -> crate::Result<crate::ConnectReq> {
        let req = crate::ConnectReq::new(cb)?;
        let sockaddr: uv::sockaddr = std::mem::zeroed();
        fill_sockaddr(&mut sockaddr, addr);

        let result = crate::uvret(unsafe {
            uv_tcp_connect(
                req.into_inner(),
                self.handle,
                &sockaddr as _,
                Some(crate::uv_connect_cb),
            )
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Resets a TCP connection by sending a RST packet. This is accomplished by setting the
    /// SO_LINGER socket option with a linger interval of zero and then calling close(). Due to
    /// some platform inconsistencies, mixing of shutdown() and close_reset() calls is not allowed.
    pub fn close_reset(
        &mut self,
        cb: Option<impl FnMut(crate::Handle) + 'static>,
    ) -> crate::Result<()> {
        let cb = cb.map(|f| Box::new(f) as _);
        let dataptr = crate::Handle::get_data(uv_handle!(self.handle));
        if !dataptr.is_null() {
            unsafe { (*dataptr).close_cb = cb };
        }

        crate::uvret(unsafe { uv_tcp_close_reset(self.handle, Some(crate::uv_close_cb)) })
    }
}

impl FromInner<*mut uv_tcp_t> for TcpHandle {
    fn from_inner(handle: *mut uv_tcp_t) -> TcpHandle {
        TcpHandle { handle }
    }
}

impl IntoInner<*mut uv_tcp_t> for TcpHandle {
    fn into_inner(self) -> *mut uv_tcp_t {
        return self.handle;
    }
}

impl IntoInner<*mut uv::uv_stream_t> for TcpHandle {
    fn into_inner(self) -> *mut uv::uv_stream_t {
        uv_handle!(self.handle)
    }
}

impl IntoInner<*mut uv::uv_handle_t> for TcpHandle {
    fn into_inner(self) -> *mut uv::uv_handle_t {
        uv_handle!(self.handle)
    }
}

impl crate::StreamTrait for TcpHandle {}
impl crate::HandleTrait for TcpHandle {}

impl crate::Loop {
    /// Initialize the handle. No socket is created as of yet.
    pub fn tcp(&self) -> crate::Result<TcpHandle> {
        TcpHandle::new(self)
    }
}
