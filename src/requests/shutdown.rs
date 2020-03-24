use crate::{FromInner, Inner, IntoInner};
use uv::uv_shutdown_t;

/// Additional data stored on the request
pub(crate) struct ShutdownDataFields {
    shutdown_cb: Option<Box<dyn FnMut(ShutdownReq, i32)>>,
}

/// Callback for uv_shutdown
pub(crate) extern "C" fn uv_shutdown_cb(req: *mut uv_shutdown_t, status: std::os::raw::c_int) {
    let dataptr = crate::Req::get_data(uv_handle!(req));
    if !dataptr.is_null() {
        unsafe {
            if let super::ShutdownData(d) = *dataptr {
                if let Some(f) = d.shutdown_cb.as_mut() {
                    f(req.into_inner(), status as _);
                }
            }
        }
    }

    // free memory
    let req = ShutdownReq::from_inner(req);
    req.destroy();
}

/// Shutdown request type.
pub struct ShutdownReq {
    req: *mut uv_shutdown_t,
}

impl ShutdownReq {
    /// Create a new shutdown request
    pub fn new(cb: Option<impl FnMut(ShutdownReq, i32) + 'static>) -> crate::Result<ShutdownReq> {
        let layout = std::alloc::Layout::new::<uv_shutdown_t>();
        let req = unsafe { std::alloc::alloc(layout) as *mut uv_shutdown_t };
        if req.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let shutdown_cb = cb.map(|f| Box::new(f) as _);
        crate::Req::initialize_data(
            uv_handle!(req),
            super::ShutdownData(ShutdownDataFields { shutdown_cb }),
        );

        Ok(ShutdownReq { req })
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

impl crate::ReqTrait for ShutdownReq {}
