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
        cb: Option<impl FnMut(crate::ShutdownReq, i32) + 'static>,
    ) -> crate::Result<crate::ShutdownReq> {
        let req = crate::ShutdownReq::new(cb)?;
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

    /// Stop reading data from the stream. The uv_read_cb callback will no longer be called.
    ///
    /// This function is idempotent and may be safely called on a stopped stream.
    fn read_stop(&mut self) -> crate::Result<()> {
        crate::uvret(unsafe { uv_read_stop((*self).into()) })
    }

    /// Write data to stream. Buffers are written in order.
    ///
    /// Note: The memory pointed to by the buffers must remain valid until the callback gets
    /// called.
    fn write(
        &mut self,
        bufs: &[impl crate::BufTrait],
        cb: Option<impl FnMut(crate::WriteReq, i32) + 'static>,
    ) -> crate::Result<crate::WriteReq> {
        let req = crate::WriteReq::new(bufs, cb)?;
        let result = crate::uvret(unsafe {
            uv_write(
                req.into(),
                (*self).into(),
                req.bufs_ptr,
                bufs.len() as _,
                Some(crate::uv_write_cb),
            )
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Extended write function for sending handles over a pipe. The pipe must be initialized with
    /// ipc == 1.
    ///
    /// Note: send_handle must be a TCP socket or pipe, which is a server or a connection
    /// (listening or connected state). Bound sockets or pipes will be assumed to be servers.
    ///
    /// Note: The memory pointed to by the buffers must remain valid until the callback gets
    /// called.
    fn write2(
        &mut self,
        send_handle: &StreamHandle,
        bufs: &[impl crate::BufTrait],
        cb: Option<impl FnMut(crate::WriteReq, i32) + 'static>,
    ) -> crate::Result<crate::WriteReq> {
        let req = crate::WriteReq::new(bufs, cb)?;
        let result = crate::uvret(unsafe {
            uv_write2(
                req.into(),
                (*self).into(),
                req.bufs_ptr,
                bufs.len() as _,
                (*send_handle).into(),
                Some(crate::uv_write_cb),
            )
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Same as write(), but won’t queue a write request if it can’t be completed immediately.
    ///
    /// Will return number of bytes written (can be less than the supplied buffer size).
    fn try_write(&mut self, bufs: &[impl crate::BufTrait]) -> crate::Result<i32> {
        let bufs: Vec<uv::uv_buf_t> = bufs.iter().map(|b| (*(*b).into()).clone()).collect();
        let bufs_ptr = bufs.as_mut_ptr();
        let bufs_len = bufs.len();
        let bufs_capacity = bufs.capacity();
        let result = unsafe { uv_try_write((*self).into(), bufs_ptr, bufs_len as _) };

        std::mem::drop(Vec::from_raw_parts(bufs_ptr, bufs_len, bufs_capacity));

        crate::uvret(result).map(|_| result as _)
    }

    /// Returns true if the stream is readable, false otherwise.
    fn is_readable(&self) -> bool {
        unsafe { uv_is_readable((*self).into()) != 0 }
    }

    /// Returns true if the stream is writable, false otherwise.
    fn is_writable(&self) -> bool {
        unsafe { uv_is_writable((*self).into()) != 0 }
    }

    /// Enable or disable blocking mode for a stream.
    ///
    /// When blocking mode is enabled all writes complete synchronously. The interface remains
    /// unchanged otherwise, e.g. completion or failure of the operation will still be reported
    /// through a callback which is made asynchronously.
    ///
    /// Warning: Relying too much on this API is not recommended. It is likely to change
    /// significantly in the future.
    ///
    /// Currently only works on Windows for PipeHandles. On UNIX platforms, all Stream handles are
    /// supported.
    ///
    /// Also libuv currently makes no ordering guarantee when the blocking mode is changed after
    /// write requests have already been submitted. Therefore it is recommended to set the blocking
    /// mode immediately after opening or creating the stream.
    fn set_blocking(&mut self, blocking: bool) -> crate::Result<()> {
        crate::uvret(unsafe {
            uv_stream_set_blocking((*self).into(), if blocking { 1 } else { 0 })
        })
    }

    /// Returns the size of the write queue.
    fn get_write_queue_size(&self) -> usize {
        unsafe { uv_stream_get_write_queue_size((*self).into()) }
    }
}

impl StreamTrait for StreamHandle {}
impl crate::HandleTrait for StreamHandle {}
