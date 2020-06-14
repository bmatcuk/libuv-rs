use crate::{FromInner, Inner, IntoInner};
use uv::uv_shutdown_t;

callbacks! {
    pub ShutdownCB(req: ShutdownReq, status: crate::Result<u32>);
}

/// Additional data stored on the request
pub(crate) struct ShutdownDataFields<'a> {
    shutdown_cb: ShutdownCB<'a>,
}

/// Callback for uv_shutdown
pub(crate) extern "C" fn uv_shutdown_cb(req: *mut uv_shutdown_t, status: std::os::raw::c_int) {
    let dataptr = crate::Req::get_data(uv_handle!(req));
    if !dataptr.is_null() {
        unsafe {
            if let super::ShutdownData(d) = &mut *dataptr {
                let status = if status < 0 {
                    Err(crate::Error::from_inner(status as uv::uv_errno_t))
                } else {
                    Ok(status as _)
                };
                d.shutdown_cb.call(req.into_inner(), status);
            }
        }
    }

    // free memory
    let mut req = ShutdownReq::from_inner(req);
    req.destroy();
}

/// Shutdown request type.
#[derive(Clone, Copy)]
pub struct ShutdownReq {
    req: *mut uv_shutdown_t,
}

impl ShutdownReq {
    /// Create a new shutdown request
    pub fn new<CB: Into<ShutdownCB<'static>>>(cb: CB) -> crate::Result<ShutdownReq> {
        let layout = std::alloc::Layout::new::<uv_shutdown_t>();
        let req = unsafe { std::alloc::alloc(layout) as *mut uv_shutdown_t };
        if req.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let shutdown_cb = cb.into();
        crate::Req::initialize_data(
            uv_handle!(req),
            super::ShutdownData(ShutdownDataFields { shutdown_cb }),
        );

        Ok(ShutdownReq { req })
    }

    /// The stream where this connection request is running
    pub fn handle(&self) -> crate::StreamHandle {
        unsafe { (*self.req).handle }.into_inner()
    }

    /// Deallocate the shutdown request - called automatically in the shudown callback
    pub fn destroy(&mut self) {
        crate::Req::free_data(uv_handle!(self.req));

        let layout = std::alloc::Layout::new::<uv_shutdown_t>();
        unsafe { std::alloc::dealloc(self.req as _, layout) };
    }
}

impl FromInner<*mut uv_shutdown_t> for ShutdownReq {
    fn from_inner(req: *mut uv_shutdown_t) -> ShutdownReq {
        ShutdownReq { req }
    }
}

impl Inner<*mut uv_shutdown_t> for ShutdownReq {
    fn inner(&self) -> *mut uv_shutdown_t {
        self.req
    }
}

impl Inner<*mut uv::uv_req_t> for ShutdownReq {
    fn inner(&self) -> *mut uv::uv_req_t {
        uv_handle!(self.req)
    }
}

impl From<ShutdownReq> for crate::Req {
    fn from(shutdown: ShutdownReq) -> crate::Req {
        crate::Req::from_inner(Inner::<*mut uv::uv_req_t>::inner(&shutdown))
    }
}

impl crate::ToReq for ShutdownReq {
    fn to_req(&self) -> crate::Req {
        crate::Req::from_inner(Inner::<*mut uv::uv_req_t>::inner(self))
    }
}

impl crate::ReqTrait for ShutdownReq {}
