use crate::{FromInner, HandleTrait, Inner, IntoInner, ToHandle};
use std::convert::{TryFrom, TryInto};
use std::net::SocketAddr;
use uv::{
    uv_socketpair, uv_tcp_bind, uv_tcp_close_reset, uv_tcp_connect, uv_tcp_getpeername,
    uv_tcp_getsockname, uv_tcp_init, uv_tcp_init_ex, uv_tcp_keepalive, uv_tcp_keepalive_ex,
    uv_tcp_nodelay, uv_tcp_open, uv_tcp_simultaneous_accepts, uv_tcp_t, AF_INET, AF_INET6,
    AF_UNSPEC,
};

bitflags! {
    /// Flags to TcpHandle::new_ex()
    pub struct TcpFlags: u32 {
        const AF_INET = AF_INET as _;
        const AF_INET6 = AF_INET6 as _;
        const AF_UNSPEC = AF_UNSPEC as _;
    }
}

bitflags! {
    /// Flags to TcpHandle::bind()
    pub struct TcpBindFlags: u32 {
        /// Dual-stack support is disabled and only IPv6 is used.
        const IPV6ONLY = uv::uv_tcp_flags_UV_TCP_IPV6ONLY as _;

        /// Enable REUSEPORT socket option when binding the handle. This allows completely
        /// duplicate bindings by multiple processes or threads if they all set REUSEPORT before
        /// binding the port. Incoming connections are distributed across the participating
        /// listener sockets.
        ///
        /// This flag is available only on Linux 3.9+, DragonFlyBSD 3.6+, FreeBSD 12.0+, Solaris
        /// 11.4, and AIX 7.2.5+ for now.
        const REUSEPORT = uv::uv_tcp_flags_UV_TCP_REUSEPORT as _;
    }
}

bitflags! {
    /// Flags to pipe()
    pub struct SocketFlags: i32 {
        /// Opens the specified socket handle for OVERLAPPED or FIONBIO/O_NONBLOCK I/O usage. This
        /// is recommended for handles that will be used by libuv, and not usually recommended
        /// otherwise.
        const NONBLOCK_PIPE = uv::uv_stdio_flags_UV_NONBLOCK_PIPE as _;
    }
}

/// Create a pair of connected sockets with the specified properties. The resulting handles can be
/// passed to TcpHandle::open(), used with ProcessHandle::spawn(), or for any other purpose.
///
/// Valid values for flags0 and flags1 are:
///
/// Equivalent to socketpair(2) with a domain of AF_UNIX.
pub fn socketpair(
    socktype: u32,
    protocol: u32,
    flags0: SocketFlags,
    flags1: SocketFlags,
) -> crate::Result<(crate::Socket, crate::Socket)> {
    let mut socks = Vec::with_capacity(2);
    unsafe {
        crate::uvret(uv_socketpair(
            socktype as _,
            protocol as _,
            socks.as_mut_ptr(),
            flags0.bits(),
            flags1.bits(),
        ))?;
        socks.set_len(2);
    }
    Ok((socks[0] as _, socks[1] as _))
}

/// TCP handles are used to represent both TCP streams and servers.
#[derive(Clone, Copy)]
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
            return Err(crate::Error::from_inner(ret as uv::uv_errno_t));
        }

        crate::StreamHandle::initialize_data(uv_handle!(handle), super::NoAddlStreamData);

        Ok(TcpHandle { handle })
    }

    /// Initialize the handle with the specified flags. A socket will be created for the given
    /// domain. If the specified domain is AF_UNSPEC no socket is created, just like new().
    pub fn new_ex(r#loop: &crate::Loop, flags: TcpFlags) -> crate::Result<TcpHandle> {
        let layout = std::alloc::Layout::new::<uv_tcp_t>();
        let handle = unsafe { std::alloc::alloc(layout) as *mut uv_tcp_t };
        if handle.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let ret = unsafe { uv_tcp_init_ex(r#loop.into_inner(), handle, flags.bits()) };
        if ret < 0 {
            unsafe { std::alloc::dealloc(handle as _, layout) };
            return Err(crate::Error::from_inner(ret as uv::uv_errno_t));
        }

        crate::StreamHandle::initialize_data(uv_handle!(handle), super::NoAddlStreamData);

        Ok(TcpHandle { handle })
    }

    /// Open an existing file descriptor or SOCKET as a TCP handle.
    ///
    /// Changed in version 1.2.1: the file descriptor is set to non-blocking mode.
    ///
    /// Note The passed file descriptor or SOCKET is not checked for its type, but it’s required
    /// that it represents a valid stream socket.
    pub fn open(&mut self, socket: crate::Socket) -> crate::Result<()> {
        crate::uvret(unsafe { uv_tcp_open(self.handle, socket) })
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
    ///
    /// If `delay` is less than 1 then EINVAL is returned.
    pub fn keepalive(&mut self, enable: bool, delay: u32) -> crate::Result<()> {
        crate::uvret(unsafe { uv_tcp_keepalive(self.handle, if enable { 1 } else { 0 }, delay) })
    }

    /// Enable / disable TCP keep-alive with all socket options: TCP_KEEPIDLE, TCP_KEEPINTVL and
    /// TCP_KEEPCNT. idle is the value for TCP_KEEPIDLE, intvl is the value for TCP_KEEPINTVL, cnt
    /// is the value for TCP_KEEPCNT, ignored when on false.
    ///
    /// With TCP keep-alive enabled, idle is the time (in seconds) the connection needs to remain
    /// idle before TCP starts sending keep-alive probes. intvl is the time (in seconds) between
    /// individual keep-alive probes. TCP will drop the connection after sending cnt probes without
    /// getting any replies from the peer, then the handle is destroyed with a ETIMEDOUT error
    /// passed to the corresponding callback.
    ///
    /// If one of idle, intvl, or cnt is less than 1, EINVAL is returned.
    pub fn keepalive_ex(&mut self, on: bool, idle: u32, intvl: u32, cnt: u32) -> crate::Result<()> {
        crate::uvret(unsafe { uv_tcp_keepalive_ex(self.handle, if on { 1 } else { 0 }, idle, intvl, cnt) })
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
    /// When the port is already taken, you can expect to see an EADDRINUSE error from listen or
    /// connect unless you specify REUSEPORT in `flags` for all the binding sockets. That is, a
    /// successful call to this function does not guarantee that the call to listen or connect will
    /// succeed as well.
    ///
    /// Note: REUSEPORT flag is available only on Linux 3.9+, DragonFlyBSD 3.6+, FreeBSD 12.0+,
    /// Solaris 11.4, and AIX 7.2.5+ at the moment. On other platforms this function will return an
    /// ENOTSUP error.
    pub fn bind(
        &mut self,
        addr: &SocketAddr,
        flags: TcpBindFlags,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut sockaddr: uv::sockaddr_storage = unsafe { std::mem::zeroed() };
        crate::fill_sockaddr(&mut sockaddr as _, addr)?;
        crate::uvret(unsafe { uv_tcp_bind(self.handle, uv_handle!(&sockaddr), flags.bits()) })
            .map_err(|e| Box::new(e) as _)
    }

    /// Get the current address to which the handle is bound.
    pub fn getsockname(&self) -> Result<SocketAddr, Box<dyn std::error::Error>> {
        let mut sockaddr: uv::sockaddr_storage = unsafe { std::mem::zeroed() };
        let mut sockaddr_len: std::os::raw::c_int =
            std::mem::size_of::<uv::sockaddr_storage>() as _;
        crate::uvret(unsafe {
            uv_tcp_getsockname(
                self.handle,
                uv_handle!(&mut sockaddr),
                &mut sockaddr_len as _,
            )
        })?;

        crate::build_socketaddr(uv_handle!(&sockaddr))
    }

    /// Get the address of the peer connected to the handle.
    pub fn getpeername(&self) -> Result<SocketAddr, Box<dyn std::error::Error>> {
        let mut sockaddr: uv::sockaddr_storage = unsafe { std::mem::zeroed() };
        let mut sockaddr_len: std::os::raw::c_int =
            std::mem::size_of::<uv::sockaddr_storage>() as _;
        crate::uvret(unsafe {
            uv_tcp_getpeername(
                self.handle,
                uv_handle!(&mut sockaddr),
                &mut sockaddr_len as _,
            )
        })?;

        crate::build_socketaddr(uv_handle!(&sockaddr))
    }

    /// Establish an IPv4 or IPv6 TCP connection.
    ///
    /// On Windows if the addr is initialized to point to an unspecified address (0.0.0.0 or ::) it
    /// will be changed to point to localhost. This is done to match the behavior of Linux systems.
    ///
    /// The callback is made when the connection has been established or when a connection error
    /// happened.
    pub fn connect<CB: Into<crate::ConnectCB<'static>>>(
        &mut self,
        addr: &SocketAddr,
        cb: CB,
    ) -> Result<crate::ConnectReq, Box<dyn std::error::Error>> {
        let mut req = crate::ConnectReq::new(cb)?;
        let mut sockaddr: uv::sockaddr_storage = unsafe { std::mem::zeroed() };
        crate::fill_sockaddr(&mut sockaddr, addr)?;

        let result = crate::uvret(unsafe {
            uv_tcp_connect(
                req.inner(),
                self.handle,
                uv_handle!(&sockaddr),
                Some(crate::uv_connect_cb),
            )
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req).map_err(|e| Box::new(e) as _)
    }

    /// Resets a TCP connection by sending a RST packet. This is accomplished by setting the
    /// SO_LINGER socket option with a linger interval of zero and then calling close(). Due to
    /// some platform inconsistencies, mixing of shutdown() and close_reset() calls is not allowed.
    pub fn close_reset<CB: Into<crate::CloseCB<'static>>>(&mut self, cb: CB) -> crate::Result<()> {
        let cb = cb.into();
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

impl Inner<*mut uv_tcp_t> for TcpHandle {
    fn inner(&self) -> *mut uv_tcp_t {
        return self.handle;
    }
}

impl Inner<*mut uv::uv_stream_t> for TcpHandle {
    fn inner(&self) -> *mut uv::uv_stream_t {
        uv_handle!(self.handle)
    }
}

impl Inner<*mut uv::uv_handle_t> for TcpHandle {
    fn inner(&self) -> *mut uv::uv_handle_t {
        uv_handle!(self.handle)
    }
}

impl From<TcpHandle> for crate::StreamHandle {
    fn from(tcp: TcpHandle) -> crate::StreamHandle {
        crate::StreamHandle::from_inner(Inner::<*mut uv::uv_stream_t>::inner(&tcp))
    }
}

impl From<TcpHandle> for crate::Handle {
    fn from(tcp: TcpHandle) -> crate::Handle {
        crate::Handle::from_inner(Inner::<*mut uv::uv_handle_t>::inner(&tcp))
    }
}

impl crate::ToStream for TcpHandle {
    fn to_stream(&self) -> crate::StreamHandle {
        crate::StreamHandle::from_inner(Inner::<*mut uv::uv_stream_t>::inner(self))
    }
}

impl ToHandle for TcpHandle {
    fn to_handle(&self) -> crate::Handle {
        crate::Handle::from_inner(Inner::<*mut uv::uv_handle_t>::inner(self))
    }
}

impl TryFrom<crate::Handle> for TcpHandle {
    type Error = crate::ConversionError;

    fn try_from(handle: crate::Handle) -> Result<Self, Self::Error> {
        let t = handle.get_type();
        if t != crate::HandleType::TCP {
            Err(crate::ConversionError::new(t, crate::HandleType::TCP))
        } else {
            Ok((handle.inner() as *mut uv_tcp_t).into_inner())
        }
    }
}

impl TryFrom<crate::StreamHandle> for TcpHandle {
    type Error = crate::ConversionError;

    fn try_from(stream: crate::StreamHandle) -> Result<Self, Self::Error> {
        stream.to_handle().try_into()
    }
}

impl crate::StreamTrait for TcpHandle {}
impl HandleTrait for TcpHandle {}

impl crate::Loop {
    /// Initialize the handle. No socket is created as of yet.
    pub fn tcp(&self) -> crate::Result<TcpHandle> {
        TcpHandle::new(self)
    }
}
