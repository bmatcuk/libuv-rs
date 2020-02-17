use uv::{
    uv_close, uv_fileno, uv_handle_get_data, uv_handle_get_loop, uv_handle_get_type,
    uv_handle_set_data, uv_handle_t, uv_handle_type_name, uv_has_ref, uv_is_active, uv_is_closing,
    uv_recv_buffer_size, uv_ref, uv_send_buffer_size, uv_unref,
};

/// Handle is the base type for all libuv handle types.
pub struct Handle {
    handle: *mut uv_handle_t,
}

impl Handle {
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
    pub fn is_active(&self) -> bool {
        unsafe { uv_is_active(self.handle) != 0 }
    }

    /// Returns non-zero if the handle is closing or closed, zero otherwise.
    ///
    /// Note: This function should only be used between the initialization of the handle and the
    /// arrival of the close callback.
    pub fn is_closing(&self) -> bool {
        unsafe { uv_is_closing(self.handle) != 0 }
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
    pub fn close(&mut self) {
        unsafe { uv_close(self.handle, None) };
    }
}

impl From<*mut uv_handle_t> for Handle {
    fn from(handle: *mut uv_handle_t) -> Handle {
        Handle { handle }
    }
}
