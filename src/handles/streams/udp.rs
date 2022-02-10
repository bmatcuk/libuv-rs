use crate::{FromInner, HandleTrait, Inner, IntoInner, ToHandle, NREAD};
use std::convert::{TryFrom, TryInto};
use std::ffi::CString;
use std::net::SocketAddr;
use uv::{
    uv_udp_bind, uv_udp_connect, uv_udp_get_send_queue_count, uv_udp_get_send_queue_size,
    uv_udp_getpeername, uv_udp_getsockname, uv_udp_init, uv_udp_init_ex, uv_udp_recv_start,
    uv_udp_recv_stop, uv_udp_send, uv_udp_set_broadcast, uv_udp_set_membership,
    uv_udp_set_multicast_interface, uv_udp_set_multicast_loop, uv_udp_set_multicast_ttl,
    uv_udp_set_source_membership, uv_udp_set_ttl, uv_udp_t, uv_udp_try_send, uv_udp_using_recvmmsg,
    AF_INET, AF_INET6, AF_UNSPEC,
};

bitflags! {
    /// Flags to UdpHandle::new_ex()
    pub struct UdpFlags: u32 {
        const AF_INET = AF_INET as _;
        const AF_INET6 = AF_INET6 as _;
        const AF_UNSPEC = AF_UNSPEC as _;

        /// Indicates that recvmmsg should be used, if available.
        const RECVMMSG = uv::uv_udp_flags_UV_UDP_RECVMMSG as _;
    }
}

bitflags! {
    /// Flags to UdpHandle::bind()
    pub struct UdpBindFlags: u32 {
        /// Disables dual stack mode.
        const IPV6ONLY = uv::uv_udp_flags_UV_UDP_IPV6ONLY as _;

        /// Indicates if SO_REUSEADDR will be set when binding the handle in bind(). This sets the
        /// SO_REUSEPORT socket flag on the BSDs and OS X. On other Unix platforms, it sets the
        /// SO_REUSEADDR flag. What that means is that multiple threads or processes can bind to
        /// the same address without error (provided they all set the flag) but only the last one
        /// to bind will receive any traffic, in effect "stealing" the port from the previous
        /// listener.
        const REUSEADDR = uv::uv_udp_flags_UV_UDP_REUSEADDR as _;

        /// Indicates if IP_RECVERR/IPV6_RECVERR will be set when binding the handle.  This sets
        /// IP_RECVERR for IPv4 and IPV6_RECVERR for IPv6 UDP sockets on Linux. This stops the
        /// Linux kernel from supressing some ICMP error messages and enables full ICMP error
        /// reporting for faster failover.  This flag is no-op on platforms other than Linux.
        const RECVERR = uv::uv_udp_flags_UV_UDP_LINUX_RECVERR as _;
    }
}

bitflags! {
    /// Flags in RecvCB
    pub struct UdpRecvFlags: u32 {
        /// Indicates message was truncated because read buffer was too small. The remainder was
        /// discarded by the OS. Used in receive callback.
        const PARTIAL = uv::uv_udp_flags_UV_UDP_PARTIAL as _;

        /// Indicates that the message was received by recvmmsg, so the buffer provided must not be
        /// freed by the receive callback.
        const MMSG_CHUNK = uv::uv_udp_flags_UV_UDP_MMSG_CHUNK as _;

        /// Indicates that the buffer provided has been fully utilized by recvmmsg and that it
        /// should now be freed by the receive callback. When this flag is set in the receive
        /// callback, nread will always be 0 and addr will always be empty.
        const MMSG_FREE = uv::uv_udp_flags_UV_UDP_MMSG_FREE as _;
    }
}

#[repr(u32)]
pub enum Membership {
    Leave = uv::uv_membership_UV_LEAVE_GROUP as _,
    Join = uv::uv_membership_UV_JOIN_GROUP as _,
}

callbacks! {
    pub RecvCB(
        handle: UdpHandle,
        nread: crate::Result<usize>,
        buf: crate::ReadonlyBuf,
        addr: SocketAddr,
        flags: UdpRecvFlags
    );
}

/// Additional data to store on the stream
#[derive(Default)]
pub(crate) struct UdpDataFields<'a> {
    recv_cb: RecvCB<'a>,
}

/// Callback for uv_udp_recv_start
extern "C" fn uv_udp_recv_cb(
    handle: *mut uv_udp_t,
    nread: NREAD,
    buf: *const uv::uv_buf_t,
    addr: *const uv::sockaddr,
    flags: std::os::raw::c_uint,
) {
    let dataptr = crate::StreamHandle::get_data(uv_handle!(handle));
    if !dataptr.is_null() {
        if let super::UdpData(d) = unsafe { &mut (*dataptr).addl } {
            let sockaddr = crate::build_socketaddr(addr);
            if let Ok(sockaddr) = sockaddr {
                let nread = if nread < 0 {
                    Err(crate::Error::from_inner(nread as uv::uv_errno_t))
                } else {
                    Ok(nread as usize)
                };
                d.recv_cb.call(
                    handle.into_inner(),
                    nread,
                    buf.into_inner(),
                    sockaddr,
                    UdpRecvFlags::from_bits_truncate(flags),
                );
            }
        }
    }
}

/// UDP handles encapsulate UDP communication for both clients and servers.
#[derive(Clone, Copy)]
pub struct UdpHandle {
    handle: *mut uv_udp_t,
}

impl UdpHandle {
    /// Initialize a new UDP handle. The actual socket is created lazily.
    pub fn new(r#loop: &crate::Loop) -> crate::Result<UdpHandle> {
        let layout = std::alloc::Layout::new::<uv_udp_t>();
        let handle = unsafe { std::alloc::alloc(layout) as *mut uv_udp_t };
        if handle.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let ret = unsafe { uv_udp_init(r#loop.into_inner(), handle) };
        if ret < 0 {
            unsafe { std::alloc::dealloc(handle as _, layout) };
            return Err(crate::Error::from_inner(ret as uv::uv_errno_t));
        }

        crate::StreamHandle::initialize_data(
            uv_handle!(handle),
            super::UdpData(Default::default()),
        );

        Ok(UdpHandle { handle })
    }

    /// Initialize the handle with the specified flags. A socket will be created for the given
    /// domain. If the specified domain is AF_UNSPEC no socket is created, just like new().
    pub fn new_ex(r#loop: &crate::Loop, flags: UdpFlags) -> crate::Result<UdpHandle> {
        let layout = std::alloc::Layout::new::<uv_udp_t>();
        let handle = unsafe { std::alloc::alloc(layout) as *mut uv_udp_t };
        if handle.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let ret = unsafe { uv_udp_init_ex(r#loop.into_inner(), handle, flags.bits()) };
        if ret < 0 {
            unsafe { std::alloc::dealloc(handle as _, layout) };
            return Err(crate::Error::from_inner(ret as uv::uv_errno_t));
        }

        crate::StreamHandle::initialize_data(
            uv_handle!(handle),
            super::UdpData(Default::default()),
        );

        Ok(UdpHandle { handle })
    }

    /// Bind the UDP handle to an IP address and port.
    pub fn bind(
        &mut self,
        addr: &SocketAddr,
        flags: UdpBindFlags,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut sockaddr: uv::sockaddr = unsafe { std::mem::zeroed() };
        crate::fill_sockaddr(&mut sockaddr, addr)?;
        crate::uvret(unsafe { uv_udp_bind(self.handle, &sockaddr as _, flags.bits()) })
            .map_err(|e| Box::new(e) as _)
    }

    /// Associate the UDP handle to a remote address and port, so every message sent by this handle
    /// is automatically sent to that destination. Calling this function with a None addr
    /// disconnects the handle. Trying to call connect() on an already connected handle will result
    /// in an EISCONN error. Trying to disconnect a handle that is not connected will return an
    /// ENOTCONN error.
    pub fn connect(&mut self, addr: Option<&SocketAddr>) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(addr) = addr {
            let mut sockaddr: uv::sockaddr = unsafe { std::mem::zeroed() };
            crate::fill_sockaddr(&mut sockaddr, addr)?;
            crate::uvret(unsafe { uv_udp_connect(self.handle, &sockaddr as _) })
        } else {
            crate::uvret(unsafe { uv_udp_connect(self.handle, std::ptr::null()) })
        }
        .map_err(|e| Box::new(e) as _)
    }

    /// Get the remote IP and port of the UDP handle on connected UDP handles. On unconnected
    /// handles, it returns ENOTCONN.
    pub fn getpeername(&self) -> Result<SocketAddr, Box<dyn std::error::Error>> {
        let mut sockaddr: uv::sockaddr_storage = unsafe { std::mem::zeroed() };
        let mut sockaddr_len: std::os::raw::c_int =
            std::mem::size_of::<uv::sockaddr_storage>() as _;
        crate::uvret(unsafe {
            uv_udp_getpeername(
                self.handle,
                uv_handle!(&mut sockaddr),
                &mut sockaddr_len as _,
            )
        })?;

        crate::build_socketaddr(uv_handle!(&sockaddr))
    }

    /// Get the local IP and port of the UDP handle.
    pub fn getsockname(&self) -> Result<SocketAddr, Box<dyn std::error::Error>> {
        let mut sockaddr: uv::sockaddr_storage = unsafe { std::mem::zeroed() };
        let mut sockaddr_len: std::os::raw::c_int =
            std::mem::size_of::<uv::sockaddr_storage>() as _;
        crate::uvret(unsafe {
            uv_udp_getsockname(
                self.handle,
                uv_handle!(&mut sockaddr),
                &mut sockaddr_len as _,
            )
        })?;

        crate::build_socketaddr(uv_handle!(&sockaddr))
    }

    /// Set membership for a multicast address
    pub fn set_membership(
        &mut self,
        multicast_addr: &str,
        interface_addr: &str,
        membership: Membership,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let multicast_addr = CString::new(multicast_addr)?;
        let interface_addr = CString::new(interface_addr)?;
        crate::uvret(unsafe {
            uv_udp_set_membership(
                self.handle,
                multicast_addr.as_ptr(),
                interface_addr.as_ptr(),
                membership as _,
            )
        })
        .map_err(|f| Box::new(f) as _)
    }

    /// Set membership for a source-specific multicast group.
    pub fn set_source_membership(
        &mut self,
        multicast_addr: &str,
        interface_addr: &str,
        source_addr: &str,
        membership: Membership,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let multicast_addr = CString::new(multicast_addr)?;
        let interface_addr = CString::new(interface_addr)?;
        let source_addr = CString::new(source_addr)?;
        crate::uvret(unsafe {
            uv_udp_set_source_membership(
                self.handle,
                multicast_addr.as_ptr(),
                interface_addr.as_ptr(),
                source_addr.as_ptr(),
                membership as _,
            )
        })
        .map_err(|f| Box::new(f) as _)
    }

    /// Set IP multicast loop flag. Makes multicast packets loop back to local sockets.
    pub fn set_multicast_loop(&mut self, enable: bool) -> crate::Result<()> {
        crate::uvret(unsafe { uv_udp_set_multicast_loop(self.handle, if enable { 1 } else { 0 }) })
    }

    /// Set the multicast ttl.
    pub fn set_multicast_ttl(&mut self, ttl: i32) -> crate::Result<()> {
        crate::uvret(unsafe { uv_udp_set_multicast_ttl(self.handle, ttl as _) })
    }

    /// Set the multicast interface to send or receive data on.
    pub fn set_multicast_interface(
        &mut self,
        interface_addr: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let interface_addr = CString::new(interface_addr)?;
        crate::uvret(unsafe {
            uv_udp_set_multicast_interface(self.handle, interface_addr.as_ptr())
        })
        .map_err(|f| Box::new(f) as _)
    }

    /// Set broadcast on or off.
    pub fn set_broadcast(&mut self, enable: bool) -> crate::Result<()> {
        crate::uvret(unsafe { uv_udp_set_broadcast(self.handle, if enable { 1 } else { 0 }) })
    }

    /// Set the time to live.
    pub fn set_ttl(&mut self, ttl: i32) -> crate::Result<()> {
        crate::uvret(unsafe { uv_udp_set_ttl(self.handle, ttl as _) })
    }

    /// Send data over the UDP socket. If the socket has not previously been bound with bind() it
    /// will be bound to 0.0.0.0 (the “all interfaces” IPv4 address) and a random port number.
    ///
    /// On Windows if the addr is initialized to point to an unspecified address (0.0.0.0 or ::) it
    /// will be changed to point to localhost. This is done to match the behavior of Linux systems.
    ///
    /// For connected UDP handles, addr must be set to None, otherwise it will return EISCONN
    /// error.
    ///
    /// For connectionless UDP handles, addr cannot be None, otherwise it will return EDESTADDRREQ
    /// error.
    pub fn send<CB: Into<crate::UdpSendCB<'static>>>(
        &self,
        addr: Option<&SocketAddr>,
        bufs: &[impl crate::BufTrait],
        cb: CB,
    ) -> Result<crate::UdpSendReq, Box<dyn std::error::Error>> {
        let mut req = crate::UdpSendReq::new(bufs, cb)?;
        let mut sockaddr: uv::sockaddr = unsafe { std::mem::zeroed() };
        let mut sockaddr_ptr: *const uv::sockaddr = std::ptr::null();
        if let Some(addr) = addr {
            crate::fill_sockaddr(&mut sockaddr, addr)?;
            sockaddr_ptr = &sockaddr as _;
        }

        let result = crate::uvret(unsafe {
            uv_udp_send(
                req.inner(),
                self.handle,
                req.bufs_ptr,
                bufs.len() as _,
                sockaddr_ptr,
                Some(crate::uv_udp_send_cb),
            )
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req).map_err(|e| Box::new(e) as _)
    }

    /// Same as send(), but won’t queue a send request if it can’t be completed immediately.
    ///
    /// For connected UDP handles, addr must be set to None, otherwise it will return EISCONN
    /// error.
    ///
    /// For connectionless UDP handles, addr cannot be None, otherwise it will return EDESTADDRREQ
    /// error.
    pub fn try_send(
        &self,
        addr: Option<&SocketAddr>,
        bufs: &[impl crate::BufTrait],
    ) -> Result<i32, Box<dyn std::error::Error>> {
        let (bufs_ptr, bufs_len, bufs_capacity) = bufs.into_inner();
        let mut sockaddr: uv::sockaddr = unsafe { std::mem::zeroed() };
        let mut sockaddr_ptr: *const uv::sockaddr = std::ptr::null();
        if let Some(addr) = addr {
            crate::fill_sockaddr(&mut sockaddr, addr)?;
            sockaddr_ptr = &sockaddr as _;
        }

        let result = unsafe { uv_udp_try_send(self.handle, bufs_ptr, bufs_len as _, sockaddr_ptr) };

        unsafe { std::mem::drop(Vec::from_raw_parts(bufs_ptr, bufs_len, bufs_capacity)) };

        crate::uvret(result)
            .map(|_| result as _)
            .map_err(|e| Box::new(e) as _)
    }

    /// Prepare for receiving data. If the socket has not previously been bound with bind() it is
    /// bound to 0.0.0.0 (the “all interfaces” IPv4 address) and a random port number.
    ///
    /// using_recvmmsg() can be used in the allocation callback to determine if a buffer sized for
    /// use with recvmmsg should be allocated for the current handle/platform. The use of recvmmsg
    /// requires a buffer larger than 2 * 64KB to be passed to the allocation callback.
    ///
    /// Note: When using recvmmsg, the number of messages received at a time is limited by the
    /// number of max size dgrams that will fit into the buffer allocated in allocation callback,
    /// and suggested_size in alloc_cb for udp_recv is always set to the size of 1 max size dgram.
    pub fn recv_start<ACB: Into<crate::AllocCB<'static>>, CB: Into<RecvCB<'static>>>(
        &mut self,
        alloc_cb: ACB,
        recv_cb: CB,
    ) -> crate::Result<()> {
        // uv_alloc_cb is either Some(alloc_cb) or None
        // uv_recv_cb is either Some(udp_recv_cb) or None
        let alloc_cb = alloc_cb.into();
        let recv_cb = recv_cb.into();
        let uv_alloc_cb = use_c_callback!(crate::uv_alloc_cb, alloc_cb);
        let uv_recv_cb = use_c_callback!(uv_udp_recv_cb, recv_cb);

        // alloc_cb is either Some(closure) or None
        // recv_cb is either Some(closure) or None
        let dataptr = crate::StreamHandle::get_data(uv_handle!(self.handle));
        if !dataptr.is_null() {
            unsafe { (*dataptr).alloc_cb = alloc_cb };
            if let super::UdpData(d) = unsafe { &mut (*dataptr).addl } {
                d.recv_cb = recv_cb;
            }
        }

        crate::uvret(unsafe { uv_udp_recv_start(self.handle, uv_alloc_cb, uv_recv_cb) })
    }

    /// Stop listening for incoming datagrams.
    pub fn recv_stop(&mut self) -> crate::Result<()> {
        crate::uvret(unsafe { uv_udp_recv_stop(self.handle) })
    }

    /// Returns the size of the send queue
    pub fn get_send_queue_size(&self) -> usize {
        unsafe { uv_udp_get_send_queue_size(self.handle) as _ }
    }

    /// Returns the count of the send queue
    pub fn get_send_queue_count(&self) -> usize {
        unsafe { uv_udp_get_send_queue_count(self.handle) as _ }
    }

    /// Returns true if the UDP handle was created with the
    pub fn using_mmsg(&self) -> bool {
        unsafe { uv_udp_using_recvmmsg(self.handle) != 0 }
    }
}

impl FromInner<*mut uv_udp_t> for UdpHandle {
    fn from_inner(handle: *mut uv_udp_t) -> UdpHandle {
        UdpHandle { handle }
    }
}

impl Inner<*mut uv_udp_t> for UdpHandle {
    fn inner(&self) -> *mut uv_udp_t {
        self.handle
    }
}

impl Inner<*mut uv::uv_stream_t> for UdpHandle {
    fn inner(&self) -> *mut uv::uv_stream_t {
        uv_handle!(self.handle)
    }
}

impl Inner<*mut uv::uv_handle_t> for UdpHandle {
    fn inner(&self) -> *mut uv::uv_handle_t {
        uv_handle!(self.handle)
    }
}

impl From<UdpHandle> for crate::StreamHandle {
    fn from(udp: UdpHandle) -> crate::StreamHandle {
        crate::StreamHandle::from_inner(Inner::<*mut uv::uv_stream_t>::inner(&udp))
    }
}

impl From<UdpHandle> for crate::Handle {
    fn from(udp: UdpHandle) -> crate::Handle {
        crate::Handle::from_inner(Inner::<*mut uv::uv_handle_t>::inner(&udp))
    }
}

impl crate::ToStream for UdpHandle {
    fn to_stream(&self) -> crate::StreamHandle {
        crate::StreamHandle::from_inner(Inner::<*mut uv::uv_stream_t>::inner(self))
    }
}

impl ToHandle for UdpHandle {
    fn to_handle(&self) -> crate::Handle {
        crate::Handle::from_inner(Inner::<*mut uv::uv_handle_t>::inner(self))
    }
}

impl TryFrom<crate::Handle> for UdpHandle {
    type Error = crate::ConversionError;

    fn try_from(handle: crate::Handle) -> Result<Self, Self::Error> {
        let t = handle.get_type();
        if t != crate::HandleType::UDP {
            Err(crate::ConversionError::new(t, crate::HandleType::UDP))
        } else {
            Ok((handle.inner() as *mut uv_udp_t).into_inner())
        }
    }
}

impl TryFrom<crate::StreamHandle> for UdpHandle {
    type Error = crate::ConversionError;

    fn try_from(stream: crate::StreamHandle) -> Result<Self, Self::Error> {
        stream.to_handle().try_into()
    }
}

impl crate::StreamTrait for UdpHandle {}
impl HandleTrait for UdpHandle {}

impl crate::Loop {
    /// Initialize a new UDP handle. The actual socket is created lazily.
    pub fn udp(&self) -> crate::Result<UdpHandle> {
        UdpHandle::new(self)
    }
}
