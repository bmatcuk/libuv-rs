use crate::{FromInner, IntoInner};
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

    /// Bind the handle to an address and port. addr should be an IPv4 address. See bind6().
    ///
    /// When the port is already taken, you can expect to see an EADDRINUSE error from either
    /// bind(), listen() or connect(). That is, a successful call to this function does not
    /// guarantee that the call to listen() or connect() will succeed as well.
    ///
    /// flags can contain IPV6ONLY, in which case dual-stack support is disabled and only IPv6 is
    /// used.
    pub fn bind4(
        &mut self,
        addr: &str,
        port: i32,
        flags: TcpBindFlags,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // convert addr to a CString for null-termination
        let addr = std::ffi::CString::new(addr)?;

        // create the sockaddr_in struct and fill it
        let mut sockaddr: uv::sockaddr_in = std::mem::zeroed();
        crate::uvret(unsafe { uv_ip4_addr(addr.as_ptr(), port, &mut sockaddr as _) })?;

        // call bind
        self.bind(uv_handle!(&sockaddr), flags)
            .map_err(|e| Box::new(e) as _)
    }

    /// Bind the handle to an address and port. addr should be an IPv6 address. See bind4().
    ///
    /// When the port is already taken, you can expect to see an EADDRINUSE error from either
    /// bind(), listen() or connect(). That is, a successful call to this function does not
    /// guarantee that the call to listen() or connect() will succeed as well.
    ///
    /// flags can contain IPV6ONLY, in which case dual-stack support is disabled and only IPv6 is
    /// used.
    pub fn bind6(
        &mut self,
        addr: &str,
        port: i32,
        flags: TcpBindFlags,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // convert addr to a CString for null-termination
        let addr = std::ffi::CString::new(addr)?;

        // create the sockaddr_in struct and fill it
        let mut sockaddr: uv::sockaddr_in6 = std::mem::zeroed();
        crate::uvret(unsafe { uv_ip6_addr(addr.as_ptr(), port, &mut sockaddr as _) })?;

        // call bind
        self.bind(uv_handle!(&sockaddr), flags)
            .map_err(|e| Box::new(e) as _)
    }

    /// Private function to actually call uv_tcp_bind
    fn bind(&mut self, addr: *const uv::sockaddr, flags: TcpBindFlags) -> crate::Result<()> {
        crate::uvret(unsafe { uv_tcp_bind(self.handle, addr, flags.bits()) })
    }

    fn getsockname(&self) -> crate::Result<SocketAddr> {
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
