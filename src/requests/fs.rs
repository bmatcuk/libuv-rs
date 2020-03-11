use crate::{FromInner, IntoInner};
use uv::{uv_fs_req_cleanup, uv_fs_t};

/// Additional data stored on the request
pub(crate) struct FsDataFields {
    fs_cb: Option<Box<dyn FnMut(FsReq)>>,
}

/// Callback for various fs functions
pub(crate) extern "C" fn uv_fs_cb(req: *mut uv_fs_t) {
    let dataptr = crate::Req::get_data(uv_handle!(req));
    if !dataptr.is_null() {
        unsafe {
            if let super::FsData(d) = *dataptr {
                if let Some(f) = d.fs_cb.as_mut() {
                    f(req.into_inner());
                }
            }
        }
    }

    // free memory
    let req = FsReq::from_inner(req);
    req.destroy();
}

/// File system request type.
pub struct FsReq {
    req: *mut uv_fs_t,
}

impl FsReq {
    /// Create a new fs request
    pub fn new(cb: Option<impl FnMut(FsReq) + 'static>) -> crate::Result<FsReq> {
        let layout = std::alloc::Layout::new::<uv_fs_t>();
        let req = unsafe { std::alloc::alloc(layout) as *mut uv_fs_t };
        if req.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let fs_cb = cb.map(|f| Box::new(f) as _);
        crate::Req::initialize_data(uv_handle!(req), super::FsData(FsDataFields { fs_cb }));

        Ok(FsReq { req })
    }

    pub fn destroy(&mut self) {
        crate::Req::free_data(uv_handle!(self.req));
        unsafe { uv_fs_req_cleanup(self.req) };

        let layout = std::alloc::Layout::new::<uv_fs_t>();
        unsafe { std::alloc::dealloc(self.req as _, layout) };
    }
}

impl FromInner<*mut uv_fs_t> for FsReq {
    fn from_inner(req: *mut uv_fs_t) -> FsReq {
        FsReq { req }
    }
}

impl IntoInner<*mut uv_fs_t> for FsReq {
    fn into_inner(self) -> *mut uv_fs_t {
        self.req
    }
}

impl IntoInner<*mut uv::uv_req_t> for FsReq {
    fn into_inner(self) -> *mut uv::uv_req_t {
        uv_handle!(self.req)
    }
}

impl From<FsReq> for crate::Req {
    fn from(fs: FsReq) -> crate::Req {
        crate::Req::from_inner(fs.into_inner())
    }
}

impl crate::ReqTrait for FsReq {}
