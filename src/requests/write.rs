use crate::{FromInner, Inner, IntoInner};
use uv::uv_write_t;

// Additional data stored on the request
pub(crate) struct WriteDataFields {
    bufs_ptr: *mut uv::uv_buf_t,
    bufs_len: usize,
    bufs_capacity: usize,
    write_cb: Option<Box<dyn FnMut(WriteReq, i32)>>,
}

/// Callback for uv_write/uv_write2
pub(crate) extern "C" fn uv_write_cb(req: *mut uv_write_t, status: std::os::raw::c_int) {
    let dataptr = crate::Req::get_data(uv_handle!(req));
    if !dataptr.is_null() {
        unsafe {
            if let super::WriteData(d) = &mut *dataptr {
                if let Some(f) = d.write_cb.as_mut() {
                    f(req.into_inner(), status as _);
                }
            }
        }
    }

    // free memory
    let mut req = WriteReq::from_inner(req);
    req.destroy();
}

/// Write request type. Careful attention must be paid when reusing objects of this type. When a
/// stream is in non-blocking mode, write requests sent with StreamHandle::write will be queued.
/// Reusing objects at this point is undefined behaviour. It is safe to reuse the WriteReq object
/// only after the callback passed to StreamHandle::write is fired.
pub struct WriteReq {
    req: *mut uv_write_t,

    /// This is only guaranteed to be set if the WriteReq was created by new(). If it was created
    /// any other way (such as by From<*mut uv_write_t>), it will not be set.
    pub(crate) bufs_ptr: *const uv::uv_buf_t,
}

impl WriteReq {
    /// Create a new write request
    pub fn new(
        bufs: &[impl crate::BufTrait],
        cb: Option<impl FnMut(WriteReq, i32) + 'static>,
    ) -> crate::Result<WriteReq> {
        let layout = std::alloc::Layout::new::<uv_write_t>();
        let req = unsafe { std::alloc::alloc(layout) as *mut uv_write_t };
        if req.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let (bufs_ptr, bufs_len, bufs_capacity) = bufs.into_inner();
        let write_cb = cb.map(|f| Box::new(f) as _);
        crate::Req::initialize_data(
            uv_handle!(req),
            super::WriteData(WriteDataFields {
                bufs_ptr,
                bufs_len,
                bufs_capacity,
                write_cb,
            }),
        );

        Ok(WriteReq { req, bufs_ptr })
    }

    /// Deallocate the WriteReq - this is done automatically in the write callback.
    pub fn destroy(&mut self) {
        let dataptr = crate::Req::get_data(uv_handle!(self.req));
        if !dataptr.is_null() {
            if let super::WriteData(d) = unsafe { &mut *dataptr } {
                if !d.bufs_ptr.is_null() {
                    // This will destroy the Vec<uv_buf_t>, but will not actually deallocate the
                    // uv_buf_t's themselves. That's up to the user to do.
                    unsafe {
                        std::mem::drop(Vec::from_raw_parts(d.bufs_ptr, d.bufs_len, d.bufs_capacity))
                    };
                }
            }
        }

        crate::Req::free_data(uv_handle!(self.req));

        let layout = std::alloc::Layout::new::<uv_write_t>();
        unsafe { std::alloc::dealloc(self.req as _, layout) };
    }
}

impl FromInner<*mut uv_write_t> for WriteReq {
    fn from_inner(req: *mut uv_write_t) -> WriteReq {
        WriteReq {
            req,
            bufs_ptr: std::ptr::null(),
        }
    }
}

impl Inner<*mut uv_write_t> for WriteReq {
    fn inner(&self) -> *mut uv_write_t {
        self.req
    }
}

impl Inner<*mut uv::uv_req_t> for WriteReq {
    fn inner(&self) -> *mut uv::uv_req_t {
        uv_handle!(self.req)
    }
}

impl From<WriteReq> for crate::Req {
    fn from(write: WriteReq) -> crate::Req {
        crate::Req::from_inner(Inner::<*mut uv::uv_req_t>::inner(&write))
    }
}

impl crate::ToReq for WriteReq {
    fn to_req(&self) -> crate::Req {
        crate::Req::from_inner(Inner::<*mut uv::uv_req_t>::inner(self))
    }
}

impl crate::ReqTrait for WriteReq {}
