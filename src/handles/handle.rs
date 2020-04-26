include!("./handle_types.inc.rs");

use crate::{FromInner, Inner, IntoInner};
use std::alloc::Layout;
use std::ffi::CStr;
use uv::{
    uv_close, uv_handle_get_data, uv_handle_get_loop, uv_handle_get_type, uv_handle_set_data,
    uv_handle_t, uv_handle_type_name, uv_has_ref, uv_is_active, uv_is_closing, uv_recv_buffer_size,
    uv_ref, uv_send_buffer_size, uv_unref,
};

impl HandleType {
    /// Returns the name of the handle type.
    pub fn name(&self) -> String {
        unsafe {
            CStr::from_ptr(uv_handle_type_name(self.into_inner()))
                .to_string_lossy()
                .into_owned()
        }
    }
}

impl std::fmt::Display for HandleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name())
    }
}

impl IntoInner<Option<Layout>> for HandleType {
    fn into_inner(self) -> Option<Layout> {
        match self {
            HandleType::ASYNC => Some(Layout::new::<uv::uv_async_t>()),
            HandleType::CHECK => Some(Layout::new::<uv::uv_check_t>()),
            HandleType::FS_EVENT => Some(Layout::new::<uv::uv_fs_event_t>()),
            HandleType::FS_POLL => Some(Layout::new::<uv::uv_fs_poll_t>()),
            HandleType::HANDLE => Some(Layout::new::<uv::uv_handle_t>()),
            HandleType::IDLE => Some(Layout::new::<uv::uv_idle_t>()),
            HandleType::NAMED_PIPE => Some(Layout::new::<uv::uv_pipe_t>()),
            HandleType::POLL => Some(Layout::new::<uv::uv_poll_t>()),
            HandleType::PREPARE => Some(Layout::new::<uv::uv_prepare_t>()),
            HandleType::PROCESS => Some(Layout::new::<uv::uv_process_t>()),
            HandleType::SIGNAL => Some(Layout::new::<uv::uv_signal_t>()),
            HandleType::STREAM => Some(Layout::new::<uv::uv_stream_t>()),
            HandleType::TCP => Some(Layout::new::<uv::uv_tcp_t>()),
            HandleType::TIMER => Some(Layout::new::<uv::uv_timer_t>()),
            HandleType::TTY => Some(Layout::new::<uv::uv_tty_t>()),
            HandleType::UDP => Some(Layout::new::<uv::uv_udp_t>()),
            _ => None,
        }
    }
}

callbacks! {
    pub(crate) CloseCB(handle: crate::Handle);
}

/// Data that we need to track with the handle.
pub(crate) struct HandleData<'a> {
    pub(crate) close_cb: CloseCB<'a>,
    pub(crate) addl: super::AddlHandleData<'a>,
}

/// Callback for uv_close
pub(crate) extern "C" fn uv_close_cb(handle: *mut uv_handle_t) {
    let dataptr = Handle::get_data(handle);
    if !dataptr.is_null() {
        unsafe {
            (*dataptr).close_cb.call(handle.into_inner());
        }
    }

    // free memory
    Handle::free_data(handle);

    let handle_obj: Handle = handle.into_inner();
    let layout: Option<Layout> = handle_obj.get_type().into_inner();
    if let Some(layout) = layout {
        unsafe { std::alloc::dealloc(handle as _, layout) };
    }
}

/// Handle is the base type for all libuv handle types.
#[derive(Clone, Copy)]
pub struct Handle {
    handle: *mut uv_handle_t,
}

impl Handle {
    /// Initialize the handle's data.
    pub(crate) fn initialize_data(handle: *mut uv_handle_t, addl: super::AddlHandleData) {
        let data: Box<HandleData> = Box::new(HandleData {
            close_cb: ().into(),
            addl,
        });
        let ptr = Box::into_raw(data);
        unsafe { uv_handle_set_data(handle, ptr as _) };
    }

    /// Retrieve the handle's data.
    pub(crate) fn get_data<'a>(handle: *mut uv_handle_t) -> *mut HandleData<'a> {
        unsafe { uv_handle_get_data(handle) as _ }
    }

    /// Free the handle's data.
    pub(crate) fn free_data(handle: *mut uv_handle_t) {
        let ptr = Handle::get_data(handle);
        std::mem::drop(unsafe { Box::from_raw(ptr) });
        unsafe { uv_handle_set_data(handle, std::ptr::null_mut()) };
    }
}

pub trait ToHandle {
    fn to_handle(&self) -> Handle;
}

impl ToHandle for Handle {
    fn to_handle(&self) -> Handle {
        Handle {
            handle: self.handle,
        }
    }
}

impl FromInner<*mut uv_handle_t> for Handle {
    fn from_inner(handle: *mut uv_handle_t) -> Handle {
        Handle { handle }
    }
}

impl Inner<*mut uv_handle_t> for Handle {
    fn inner(&self) -> *mut uv_handle_t {
        self.handle
    }
}

pub trait HandleTrait: ToHandle {
    /// Returns non-zero if the handle is active, zero if it’s inactive. What “active” means
    /// depends on the type of handle:
    ///   * An AsyncHandle is always active and cannot be deactivated, except by closing it with
    ///     close().
    ///   * A PipeHandle, TcpHandle, UdpHandle, etc. - basically any handle that deals with i/o -
    ///     is active when it is doing something that involves i/o, like reading, writing,
    ///     connecting, accepting new connections, etc.
    ///   * A CheckHandle, IdleHandle, TimerHandle, etc. is active when it has been started with a
    ///     call to start().
    ///
    /// Rule of thumb: if a handle start() function, then it’s active from the moment that function
    /// is called. Likewise, stop() deactivates the handle again.
    fn is_active(&self) -> bool {
        unsafe { uv_is_active(self.to_handle().inner()) != 0 }
    }

    /// Returns non-zero if the handle is closing or closed, zero otherwise.
    ///
    /// Note: This function should only be used between the initialization of the handle and the
    /// arrival of the close callback.
    fn is_closing(&self) -> bool {
        unsafe { uv_is_closing(self.to_handle().inner()) != 0 }
    }

    /// Request handle to be closed. close_cb will be called asynchronously after this call. This
    /// MUST be called on each handle before memory is released. Moreover, the memory can only be
    /// released in close_cb or after it has returned.
    ///
    /// Handles that wrap file descriptors are closed immediately but close_cb will still be
    /// deferred to the next iteration of the event loop. It gives you a chance to free up any
    /// resources associated with the handle.
    ///
    /// In-progress requests, like ConnectRequest or WriteRequest, are cancelled and have their
    /// callbacks called asynchronously with status=UV_ECANCELED.
    fn close<CB: Into<CloseCB<'static>>>(&mut self, cb: CB) {
        let handle = self.to_handle().inner();

        // cb is either Some(closure) or None - it is saved into data
        let cb = cb.into();
        let dataptr = Handle::get_data(handle);
        if !dataptr.is_null() {
            unsafe { (*dataptr).close_cb = cb };
        }

        unsafe { uv_close(handle, Some(uv_close_cb)) };
    }

    /// Reference the given handle. References are idempotent, that is, if a handle is already
    /// referenced calling this function again will have no effect.
    fn r#ref(&mut self) {
        unsafe { uv_ref(self.to_handle().inner()) };
    }

    /// Un-reference the given handle. References are idempotent, that is, if a handle is not
    /// referenced calling this function again will have no effect.
    fn unref(&mut self) {
        unsafe { uv_unref(self.to_handle().inner()) };
    }

    /// Returns true if the handle referenced, zero otherwise.
    fn has_ref(&self) -> bool {
        unsafe { uv_has_ref(self.to_handle().inner()) != 0 }
    }

    /// Gets or sets the size of the send buffer that the operating system uses for the socket.
    ///
    /// If value == 0, then it will return the current send buffer size. If value > 0 then it will
    /// use value to set the new send buffer size and return that.
    ///
    /// This function works for TCP, pipe and UDP handles on Unix and for TCP and UDP handles on
    /// Windows.
    ///
    /// Note: Linux will set double the size and return double the size of the original set value.
    fn send_buffer_size(&mut self, value: i32) -> crate::Result<i32> {
        let mut v = value;
        crate::uvret(unsafe { uv_send_buffer_size(self.to_handle().inner(), &mut v as _) })?;
        Ok(v)
    }

    /// Gets or sets the size of the receive buffer that the operating system uses for the socket.
    ///
    /// If value == 0, then it will return the current receive buffer size. If value > 0 then it
    /// will use value to set the new receive buffer size and return that.
    ///
    /// This function works for TCP, pipe and UDP handles on Unix and for TCP and UDP handles on
    /// Windows.
    ///
    /// Note: Linux will set double the size and return double the size of the original set value.
    fn recv_buffer_size(&mut self, value: i32) -> crate::Result<i32> {
        let mut v = value;
        crate::uvret(unsafe { uv_recv_buffer_size(self.to_handle().inner(), &mut v as _) })?;
        Ok(v)
    }

    /// Returns the Loop associated with this handle.
    fn get_loop(&self) -> crate::Loop {
        unsafe { uv_handle_get_loop(self.to_handle().inner()).into_inner() }
    }

    /// Returns the type of the handle.
    fn get_type(&self) -> HandleType {
        unsafe { uv_handle_get_type(self.to_handle().inner()).into_inner() }
    }
}

impl HandleTrait for Handle {}
