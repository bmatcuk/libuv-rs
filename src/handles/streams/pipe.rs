use crate::{FromInner, IntoInner};
use std::ffi::CString;
use std::net::SocketAddr;
use uv::{
    uv_pipe_bind, uv_pipe_chmod, uv_pipe_connect, uv_pipe_getpeername, uv_pipe_getsockname,
    uv_pipe_init, uv_pipe_pending_count, uv_pipe_pending_instances, uv_pipe_pending_type,
    uv_pipe_t,
};

bitflags! {
    /// Flags to PipeHandle::chmod()
    pub struct ChmodFlags: u32 {
        const READABLE = uv::uv_poll_event_UV_READABLE;
        const WRITABLE = uv::uv_poll_event_UV_WRITABLE;
    }
}

/// Pipe handles provide an abstraction over streaming files on Unix (including local domain
/// sockets, pipes, and FIFOs) and named pipes on Windows.
pub struct PipeHandle {
    handle: *mut uv_pipe_t,
}

impl PipeHandle {
    /// Create and initialize a pipe handle. The ipc argument is a boolean to indicate if this pipe
    /// will be used for handle passing between processes (which may change the bytes on the wire).
    /// Only a connected pipe that will be passing the handles should have this flag set, not the
    /// listening pipe that accept() is called on.
    pub fn new(r#loop: &crate::Loop, ipc: bool) -> crate::Result<PipeHandle> {
        let layout = std::alloc::Layout::new::<uv_pipe_t>();
        let handle = unsafe { std::alloc::alloc(layout) as *mut uv_pipe_t };
        if handle.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let ret = unsafe { uv_pipe_init(r#loop.into_inner(), handle, if ipc { 1 } else { 0 }) };
        if ret < 0 {
            unsafe { std::alloc::dealloc(handle as _, layout) };
            return Err(crate::Error::from_inner(ret as uv::uv_errno_t));
        }

        crate::StreamHandle::initialize_data(uv_handle!(handle), super::NoAddlStreamData);

        Ok(PipeHandle { handle })
    }

    /// Bind the pipe to a file path (Unix) or a name (Windows).
    ///
    /// Note: Paths on Unix get truncated to sizeof(sockaddr_un.sun_path) bytes, typically between
    /// 92 and 108 bytes.
    pub fn bind(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let name = CString::new(name)?;
        crate::uvret(unsafe { uv_pipe_bind(self.handle, name.as_ptr()) })
            .map_err(|e| Box::new(e) as _)
    }

    /// Connect to the Unix domain socket or the named pipe.
    ///
    /// Note: Paths on Unix get truncated to sizeof(sockaddr_un.sun_path) bytes, typically between
    /// 92 and 108 bytes.
    pub fn connect(
        &mut self,
        name: &str,
        cb: Option<impl FnMut(crate::ConnectReq, i32) + 'static>,
    ) -> Result<crate::ConnectReq, Box<dyn std::error::Error>> {
        let req = crate::ConnectReq::new(cb)?;
        let name = CString::new(name)?;
        let result = crate::uvret(unsafe {
            uv_pipe_connect(
                req.into_inner(),
                self.handle,
                name.as_ptr(),
                Some(crate::uv_connect_cb),
            )
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Get the name of the Unix domain socket or the named pipe.
    pub fn getsockname(&self) -> crate::Result<SocketAddr> {
        let mut sockaddr: uv::sockaddr_storage = std::mem::zeroed();
        let mut sockaddr_len: std::os::raw::c_int =
            std::mem::size_of::<uv::sockaddr_storage>() as _;
        crate::uvret(unsafe {
            uv_pipe_getsockname(
                self.handle,
                uv_handle!(&mut sockaddr),
                &mut sockaddr_len as _,
            )
        })?;

        crate::build_socketaddr(uv_handle!(&sockaddr))
    }

    /// Get the name of the Unix domain socket or the named pipe to which the handle is connected.
    pub fn getpeername(&self) -> crate::Result<SocketAddr> {
        let mut sockaddr: uv::sockaddr_storage = std::mem::zeroed();
        let mut sockaddr_len: std::os::raw::c_int =
            std::mem::size_of::<uv::sockaddr_storage>() as _;
        crate::uvret(unsafe {
            uv_pipe_getpeername(
                self.handle,
                uv_handle!(&mut sockaddr),
                &mut sockaddr_len as _,
            )
        })?;

        crate::build_socketaddr(uv_handle!(&sockaddr))
    }

    /// Set the number of pending pipe instance handles when the pipe server is waiting for
    /// connections.
    ///
    /// Note: This setting applies to Windows only.
    pub fn pending_instances(&mut self, count: i32) {
        unsafe { uv_pipe_pending_instances(self.handle, count as _) };
    }

    pub fn pending_count(&self) -> i32 {
        unsafe { uv_pipe_pending_count(self.handle) as _ }
    }

    /// Used to receive handles over IPC pipes.
    ///
    /// First - call pending_count(), if it’s > 0 then initialize a handle of the given type,
    /// returned by pending_type() and call uv_accept(pipe, handle).
    pub fn pending_type(&self) -> crate::HandleType {
        unsafe { uv_pipe_pending_type(self.handle).into_inner() }
    }

    /// Alters pipe permissions, allowing it to be accessed from processes run by different users.
    /// Makes the pipe writable or readable by all users. Mode can be WRITABLE, READABLE or
    /// WRITABLE | READABLE. This function is blocking.
    pub fn chmod(&mut self, flags: ChmodFlags) -> crate::Result<()> {
        crate::uvret(unsafe { uv_pipe_chmod(self.handle, flags.bits()) })
    }
}

impl FromInner<*mut uv_pipe_t> for PipeHandle {
    fn from_inner(handle: *mut uv_pipe_t) -> PipeHandle {
        PipeHandle { handle }
    }
}

impl IntoInner<*mut uv_pipe_t> for PipeHandle {
    fn into_inner(self) -> *mut uv_pipe_t {
        self.handle
    }
}

impl IntoInner<*mut uv::uv_stream_t> for PipeHandle {
    fn into_inner(self) -> *mut uv::uv_stream_t {
        uv_handle!(self.handle)
    }
}

impl IntoInner<*mut uv::uv_handle_t> for PipeHandle {
    fn into_inner(self) -> *mut uv::uv_handle_t {
        uv_handle!(self.handle)
    }
}

impl From<PipeHandle> for crate::StreamHandle {
    fn from(pipe: PipeHandle) -> crate::StreamHandle {
        crate::StreamHandle::from_inner(pipe.into_inner())
    }
}

impl From<PipeHandle> for crate::Handle {
    fn from(pipe: PipeHandle) -> crate::Handle {
        crate::Handle::from_inner(pipe.into_inner())
    }
}

impl crate::StreamTrait for PipeHandle {}
impl crate::HandleTrait for PipeHandle {}

impl crate::Loop {
    /// Create and initialize a pipe handle. The ipc argument is a boolean to indicate if this pipe
    /// will be used for handle passing between processes (which may change the bytes on the wire).
    /// Only a connected pipe that will be passing the handles should have this flag set, not the
    /// listening pipe that accept() is called on.
    pub fn pipe(&self, ipc: bool) -> crate::Result<PipeHandle> {
        PipeHandle::new(self, ipc)
    }
}
