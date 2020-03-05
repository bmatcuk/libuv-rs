use uv::uv_shutdown_t;

/// Additional data stored on the request
#[derive(Default)]
pub(crate) struct ShutdownDataFields {
    shutdown_cb: Option<Box<dyn FnMut(ShutdownReq, i32)>>,
}

/// Callback for uv_shutdown
pub(crate) extern "C" fn uv_shutdown_cb(req: *mut uv_shutdown_t, status: std::os::raw::c_int) {
    let dataptr = crate::Req::get_data(uv_handle!(req));
    if !dataptr.is_null() {
        unsafe {
            if let crate::ShutdownData(d) = *dataptr {
                if let Some(f) = d.shutdown_cb.as_mut() {
                    f(req.into(), status as _);
                }
            }
        }
    }

    // free memory
    let req = ShutdownReq::from(req);
    req.destroy();
}

/// Shutdown request type.
pub struct ShutdownReq {
    req: *mut uv_shutdown_t,
}

impl ShutdownReq {
    /// Create a new shutdown request
    pub fn new() -> crate::Result<ShutdownReq> {
        let layout = std::alloc::Layout::new::<uv_shutdown_t>();
        let req = unsafe { std::alloc::alloc(layout) as *mut uv_shutdown_t };
        if req.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        crate::Req::initialize_data(uv_handle!(req), crate::ShutdownData(Default::default()));

        Ok(ShutdownReq { req })
    }

    /// Deallocate the shutdown request - called automatically in the shudown callback
    pub fn destroy(&mut self) {
        crate::Req::free_data(uv_handle!(self.req));

        let layout = std::alloc::Layout::new::<uv_shutdown_t>();
        unsafe { std::alloc::dealloc(self.req as _, layout) };
    }
}

impl From<*mut uv_shutdown_t> for ShutdownReq {
    fn from(req: *mut uv_shutdown_t) -> ShutdownReq {
        ShutdownReq { req }
    }
}

impl Into<*mut uv_shutdown_t> for ShutdownReq {
    fn into(self) -> *mut uv_shutdown_t {
        self.req
    }
}

impl Into<*mut uv::uv_req_t> for ShutdownReq {
    fn into(self) -> *mut uv::uv_req_t {
        uv_handle!(self.req)
    }
}

impl crate::ReqTrait for ShutdownReq {}
