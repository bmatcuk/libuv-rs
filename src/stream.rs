use uv::{
    uv_accept, uv_is_readable, uv_is_writable, uv_listen, uv_read_start, uv_read_stop, uv_shutdown,
    uv_stream_get_write_queue_size, uv_stream_set_blocking, uv_stream_t, uv_try_write, uv_write,
    uv_write2,
};

/// Additional data to store on the handle
pub(crate) struct StreamDataFields {
    alloc_cb: Option<Box<dyn FnMut(crate::Handle, usize, crate::Buf)>>,
    connection_cb: Option<Box<dyn FnMut(StreamHandle, i32)>>,
    read_cb: Option<Box<dyn FnMut(StreamHandle, isize, crate::ReadonlyBuf)>>,
    pub(crate) addl: crate::AddlStreamData,
}

extern "C" fn uv_alloc_cb(
    handle: *mut uv::uv_handle_t,
    suggested_size: usize,
    buf: *mut uv::uv_buf_t,
) {
    let dataptr = StreamHandle::get_data(uv_handle!(handle));
    if !dataptr.is_null() {
        unsafe {
            if let Some(f) = (*dataptr).alloc_cb.as_mut() {
                f(handle.into(), suggested_size, buf.into());
            }
        }
    }
}

extern "C" fn uv_connection_cb(stream: *mut uv_stream_t, status: std::os::raw::c_int) {
    let dataptr = StreamHandle::get_data(stream);
    if !dataptr.is_null() {
        unsafe {
            if let Some(f) = (*dataptr).connection_cb.as_mut() {
                f(stream.into(), status as _);
            }
        }
    }
}

extern "C" fn uv_read_cb(stream: *mut uv_stream_t, nread: isize, buf: *const uv::uv_buf_t) {
    let dataptr = StreamHandle::get_data(stream);
    if !dataptr.is_null() {
        unsafe {
            if let Some(f) = (*dataptr).read_cb.as_mut() {
                f(stream.into(), nread, buf.into());
            }
        }
    }
}

/// Stream handles provide an abstraction of a duplex communication channel. StreamHandle is an
/// abstract type, libuv provides 3 stream implementations in the form of TcpHandle, PipeHandle and
/// TtyHandle.
pub struct StreamHandle {
    handle: *mut uv_stream_t,
}

impl StreamHandle {
    pub(crate) fn initialize_data(stream: *mut uv_stream_t, addl: crate::AddlStreamData) {
        let data = crate::StreamData(StreamDataFields {
            alloc_cb: None,
            connection_cb: None,
            read_cb: None,
            addl,
        });
        crate::Handle::initialize_data(uv_handle!(stream), data);
    }

    pub(crate) fn get_data(stream: *mut uv_stream_t) -> *mut StreamDataFields {
        if let crate::StreamData(d) = &mut (*crate::Handle::get_data(uv_handle!(stream))).addl {
            return d;
        }
        std::ptr::null_mut()
    }

    pub(crate) fn free_data(stream: *mut uv_stream_t) {
        crate::Handle::free_data(uv_handle!(stream));
    }
}

impl From<*mut uv_stream_t> for StreamHandle {
    fn from(handle: *mut uv_stream_t) -> StreamHandle {
        StreamHandle { handle }
    }
}

impl Into<*mut uv_stream_t> for StreamHandle {
    fn into(self) -> *mut uv_stream_t {
        self.handle
    }
}

impl Into<*mut uv::uv_handle_t> for StreamHandle {
    fn into(self) -> *mut uv::uv_handle_t {
        uv_handle!(self.handle)
    }
}

pub trait StreamTrait: Into<*mut uv_stream_t> {
    /// Shutdown the outgoing (write) side of a duplex stream. It waits for pending write requests
    /// to complete. The handle should refer to a initialized stream. The cb is called after
    /// shutdown is complete at which point the returned ShutdownReq is automatically destroy()'d.
    fn shutdown(
        &mut self,
        cb: Option<impl FnMut(crate::ShutdownReq, i32)>,
    ) -> crate::Result<crate::ShutdownReq> {
        let req = crate::ShutdownReq::new()?;
        let result = crate::uvret(unsafe {
            uv_shutdown(req.into(), (*self).into(), Some(crate::uv_shutdown_cb))
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    fn listen(
        &mut self,
        backlog: i32,
        cb: Option<impl FnMut(StreamHandle, i32) + 'static>,
    ) -> crate::Result<()> {
        // uv_cb is either Some(connection_cb) or None
        let uv_cb = cb.as_ref().map(|_| uv_connection_cb as _);

        // cb is either Some(closure) or None
        let cb = cb.map(|f| Box::new(f) as _);
        let dataptr = StreamHandle::get_data((*self).into());
        if !dataptr.is_null() {
            (*dataptr).connection_cb = cb;
        }

        crate::uvret(unsafe { uv_listen((*self).into(), backlog, uv_cb) })
    }

    /// This call is used in conjunction with listen() to accept incoming connections. Call this
    /// function after receiving the connection callback to accept the connection. Before calling
    /// this function the client handle must be initialized.
    ///
    /// When the connection callback is called it is guaranteed that this function will complete
    /// successfully the first time. If you attempt to use it more than once, it may fail. It is
    /// suggested to only call this function once per connection callback.
    ///
    /// Note: server and client must be handles running on the same loop.
    fn accept(&mut self, client: &mut StreamHandle) -> crate::Result<()> {
        crate::uvret(unsafe { uv_accept((*self).into(), (*client).into()) })
    }

    /// Read data from an incoming stream. The read_cb callback will be made several times until
    /// there is no more data to read or read_stop() is called.
    fn read_start(
        &mut self,
        alloc_cb: Option<impl FnMut(crate::Handle, usize, crate::Buf) + 'static>,
        read_cb: Option<impl FnMut(StreamHandle, isize, crate::ReadonlyBuf) + 'static>,
    ) -> crate::Result<()> {
        // uv_alloc_cb is either Some(alloc_cb) or None
        // uv_read_cb is either Some(read_cb) or None
        let uv_alloc_cb = alloc_cb.as_ref().map(|_| uv_alloc_cb as _);
        let uv_read_cb = read_cb.as_ref().map(|_| uv_read_cb as _);

        // alloc_cb is either Some(closure) or None
        // read_cb is either Some(closure) or None
        let alloc_cb = alloc_cb.map(|f| Box::new(f) as _);
        let read_cb = read_cb.map(|f| Box::new(f) as _);
        let dataptr = StreamHandle::get_data((*self).into());
        if !dataptr.is_null() {
            (*dataptr).alloc_cb = alloc_cb;
            (*dataptr).read_cb = read_cb;
        }

        crate::uvret(unsafe { uv_read_start((*self).into(), uv_alloc_cb, uv_read_cb) })
    }
}

impl StreamTrait for StreamHandle {}
