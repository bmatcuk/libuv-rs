use crate::{FromInner, HandleTrait, Inner, IntoInner};
use std::convert::TryFrom;
use uv::{uv_poll_init, uv_poll_init_socket, uv_poll_start, uv_poll_stop, uv_poll_t};

bitflags! {
    /// Poll event types
    pub struct PollEvents: u32 {
        const DISCONNECT = uv::uv_poll_event_UV_DISCONNECT;
        const PRIORITIZED = uv::uv_poll_event_UV_PRIORITIZED;
        const READABLE = uv::uv_poll_event_UV_READABLE;
        const WRITABLE = uv::uv_poll_event_UV_WRITABLE;
    }
}

callbacks! {
    pub PollCB(handle: PollHandle, status: crate::Result<u32>, events: PollEvents);
}

#[derive(Default)]
pub(crate) struct PollDataFields<'a> {
    poll_cb: PollCB<'a>,
}

extern "C" fn uv_poll_cb(
    handle: *mut uv_poll_t,
    status: std::os::raw::c_int,
    events: std::os::raw::c_int,
) {
    let dataptr = crate::Handle::get_data(uv_handle!(handle));
    if !dataptr.is_null() {
        unsafe {
            if let super::PollData(d) = &mut (*dataptr).addl {
                let status = if status < 0 {
                    Err(crate::Error::from_inner(status as uv::uv_errno_t))
                } else {
                    Ok(status as _)
                };

                d.poll_cb.call(
                    handle.into_inner(),
                    status,
                    PollEvents::from_bits_truncate(events as _),
                )
            }
        }
    }
}

/// Poll handles are used to watch file descriptors for readability, writability and disconnection
/// similar to the purpose of poll(2).
///
/// The purpose of poll handles is to enable integrating external libraries that rely on the event
/// loop to signal it about the socket status changes, like c-ares or libssh2. Using PollHandle for
/// any other purpose is not recommended; TcpHandle, UdpHandle, etc. provide an implementation that
/// is faster and more scalable than what can be achieved with PollHandle, especially on Windows.
///
/// It is possible that poll handles occasionally signal that a file descriptor is readable or
/// writable even when it isn’t. The user should therefore always be prepared to handle EAGAIN or
/// equivalent when it attempts to read from or write to the fd.
///
/// It is not okay to have multiple active poll handles for the same socket, this can cause libuv
/// to busyloop or otherwise malfunction.
///
/// The user should not close a file descriptor while it is being polled by an active poll handle.
/// This can cause the handle to report an error, but it might also start polling another socket.
/// However the fd can be safely closed immediately after a call to stop() or close().
///
/// Note: On windows only sockets can be polled with poll handles. On Unix any file descriptor that
/// would be accepted by poll(2) can be used.
///
/// Note: On AIX, watching for disconnection is not supported.
#[derive(Clone, Copy)]
pub struct PollHandle {
    handle: *mut uv_poll_t,
}

impl PollHandle {
    /// Create and initialize a new poll handle using a file descriptor
    pub fn new(r#loop: &crate::Loop, fd: crate::File) -> crate::Result<PollHandle> {
        let layout = std::alloc::Layout::new::<uv_poll_t>();
        let handle = unsafe { std::alloc::alloc(layout) as *mut uv_poll_t };
        if handle.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let ret = unsafe { uv_poll_init(r#loop.into_inner(), handle, fd) };
        if ret < 0 {
            unsafe { std::alloc::dealloc(handle as _, layout) };
            return Err(crate::Error::from_inner(ret as uv::uv_errno_t));
        }

        crate::Handle::initialize_data(uv_handle!(handle), super::PollData(Default::default()));

        Ok(PollHandle { handle })
    }

    /// Create and initialize a new poll handle using a socket descriptor. On Unix this is
    /// identical to new(). On windows it takes a SOCKET handle.
    pub fn new_socket(r#loop: &crate::Loop, socket: crate::Socket) -> crate::Result<PollHandle> {
        let layout = std::alloc::Layout::new::<uv_poll_t>();
        let handle = unsafe { std::alloc::alloc(layout) as *mut uv_poll_t };
        if handle.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let ret = unsafe { uv_poll_init_socket(r#loop.into_inner(), handle, socket) };
        if ret < 0 {
            unsafe { std::alloc::dealloc(handle as _, layout) };
            return Err(crate::Error::from_inner(ret as uv::uv_errno_t));
        }

        crate::Handle::initialize_data(uv_handle!(handle), super::PollData(Default::default()));

        Ok(PollHandle { handle })
    }

    /// Starts polling the file descriptor. events is a bitmask made up of READABLE, WRITABLE,
    /// PRIORITIZED and DISCONNECT. As soon as an event is detected the callback will be called
    /// with status set to 0, and the detected events set on the events field.
    ///
    /// The PRIORITIZED event is used to watch for sysfs interrupts or TCP out-of-band messages.
    ///
    /// The DISCONNECT event is optional in the sense that it may not be reported and the user is
    /// free to ignore it, but it can help optimize the shutdown path because an extra read or
    /// write call might be avoided.
    ///
    /// If an error happens while polling, status will a libuv::Error. The user should not close
    /// the socket while the handle is active. If the user does that anyway, the callback may be
    /// called reporting an error status, but this is not guaranteed.
    ///
    /// Note: Calling start() on a handle that is already active is fine. Doing so will update the
    /// events mask that is being watched for.
    ///
    /// Note: Though DISCONNECT can be set, it is unsupported on AIX and as such will not be set on
    /// the events field in the callback.
    pub fn start<CB: Into<PollCB<'static>>>(
        &mut self,
        events: PollEvents,
        cb: CB,
    ) -> crate::Result<()> {
        // uv_cb is either Some(poll_cb) or None
        let cb = cb.into();
        let uv_cb = use_c_callback!(uv_poll_cb, cb);

        let dataptr = crate::Handle::get_data(uv_handle!(self.handle));
        if !dataptr.is_null() {
            if let super::PollData(d) = unsafe { &mut (*dataptr).addl } {
                d.poll_cb = cb;
            }
        }

        crate::uvret(unsafe { uv_poll_start(self.handle, events.bits() as _, uv_cb) })
    }

    /// Stop polling the file descriptor, the callback will no longer be called.
    pub fn stop(&mut self) -> crate::Result<()> {
        crate::uvret(unsafe { uv_poll_stop(self.handle) })
    }
}

impl FromInner<*mut uv_poll_t> for PollHandle {
    fn from_inner(handle: *mut uv_poll_t) -> PollHandle {
        PollHandle { handle }
    }
}

impl Inner<*mut uv::uv_handle_t> for PollHandle {
    fn inner(&self) -> *mut uv::uv_handle_t {
        uv_handle!(self.handle)
    }
}

impl From<PollHandle> for crate::Handle {
    fn from(poll: PollHandle) -> crate::Handle {
        crate::Handle::from_inner(Inner::<*mut uv::uv_handle_t>::inner(&poll))
    }
}

impl crate::ToHandle for PollHandle {
    fn to_handle(&self) -> crate::Handle {
        crate::Handle::from_inner(Inner::<*mut uv::uv_handle_t>::inner(self))
    }
}

impl TryFrom<crate::Handle> for PollHandle {
    type Error = crate::ConversionError;

    fn try_from(handle: crate::Handle) -> Result<Self, Self::Error> {
        let t = handle.get_type();
        if t != crate::HandleType::POLL {
            Err(crate::ConversionError::new(t, crate::HandleType::POLL))
        } else {
            Ok((handle.inner() as *mut uv_poll_t).into_inner())
        }
    }
}

impl HandleTrait for PollHandle {}

impl crate::Loop {
    /// Create and initialize a new poll handle using a file descriptor
    pub fn poll(&self, fd: crate::File) -> crate::Result<PollHandle> {
        PollHandle::new(self, fd)
    }

    /// Create and initialize a new poll handle using a socket descriptor. On Unix this is
    /// identical to poll(). On windows it takes a SOCKET handle.
    pub fn poll_socket(&self, socket: crate::Socket) -> crate::Result<PollHandle> {
        PollHandle::new_socket(self, socket)
    }
}
