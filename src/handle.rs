include!("./handle_types.inc.rs");

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
            CStr::from_ptr(uv_handle_type_name(self.into()))
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

/// Data that we need to track with the handle.
pub(crate) struct HandleData {
    close_cb: Option<Box<dyn FnMut(crate::Handle)>>,
    pub(crate) addl: crate::AddlHandleData,
}

/// Callback for uv_close
extern "C" fn close_cb(handle: *mut uv_handle_t) {
    let dataptr = Handle::get_data(handle);
    if !dataptr.is_null() {
        unsafe {
            if let Some(f) = (*dataptr).close_cb.as_mut() {
                f(handle.into());
            }
        }
    }
}

/// Handle is the base type for all libuv handle types.
pub struct Handle {
    handle: *mut uv_handle_t,
}

impl Handle {
    /// Initialize the handle's data.
    pub(crate) fn initialize_data(handle: *mut uv_handle_t, addl: crate::AddlHandleData) {
        let data: Box<HandleData> = Box::new(HandleData {
            close_cb: None,
            addl,
        });
        let ptr = Box::into_raw(data);
        unsafe { uv_handle_set_data(handle, ptr as _) };
    }

    /// Retrieve the handle's data.
    pub(crate) fn get_data(handle: *mut uv_handle_t) -> *mut HandleData {
        unsafe { uv_handle_get_data(handle) as _ }
    }

    /// Free the handle's data.
    pub(crate) fn free_data(handle: *mut uv_handle_t) {
        let ptr = Handle::get_data(handle);
        std::mem::drop(unsafe { Box::from_raw(ptr) });
        unsafe { uv_handle_set_data(handle, std::ptr::null_mut()) };
    }
}

impl From<*mut uv_handle_t> for Handle {
    fn from(handle: *mut uv_handle_t) -> Handle {
        Handle { handle }
    }
}

impl Into<*mut uv_handle_t> for Handle {
    fn into(self) -> *mut uv_handle_t {
        self.handle
    }
}

pub trait HandleTrait: Into<*mut uv_handle_t> {
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
        unsafe { uv_is_active((*self).into()) != 0 }
    }

    /// Returns non-zero if the handle is closing or closed, zero otherwise.
    ///
    /// Note: This function should only be used between the initialization of the handle and the
    /// arrival of the close callback.
    fn is_closing(&self) -> bool {
        unsafe { uv_is_closing((*self).into()) != 0 }
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
    fn close(&mut self, cb: Option<(impl FnMut(Handle) + 'static)>) {
        let handle = (*self).into();

        // uv_cb is either Some(close_cb) or None
        let uv_cb = cb.as_ref().map(|_| close_cb as _);

        // cb is either Some(closure) or None - it is saved into data
        let cb = cb.map(|f| Box::new(f) as _);
        let dataptr = Handle::get_data(handle);
        if !dataptr.is_null() {
            unsafe { (*dataptr).close_cb = cb };
        }

        unsafe { uv_close(handle, uv_cb) };
    }

    /// Reference the given handle. References are idempotent, that is, if a handle is already
    /// referenced calling this function again will have no effect.
    fn r#ref(&mut self) {
        unsafe { uv_ref((*self).into()) };
    }

    /// Un-reference the given handle. References are idempotent, that is, if a handle is not
    /// referenced calling this function again will have no effect.
    fn unref(&mut self) {
        unsafe { uv_unref((*self).into()) };
    }

    /// Returns true if the handle referenced, zero otherwise.
    fn has_ref(&self) -> bool {
        unsafe { uv_has_ref((*self).into()) != 0 }
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
        crate::uvret(unsafe { uv_send_buffer_size((*self).into(), &mut v as _) })?;
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
        crate::uvret(unsafe { uv_recv_buffer_size((*self).into(), &mut v as _) })?;
        Ok(v)
    }

    /// Returns the Loop associated with this handle.
    fn get_loop(&self) -> crate::Loop {
        unsafe { uv_handle_get_loop((*self).into()).into() }
    }

    /// Returns the type of the handle.
    fn get_type(&self) -> HandleType {
        unsafe { uv_handle_get_type((*self).into()).into() }
    }
}

impl HandleTrait for Handle {}
