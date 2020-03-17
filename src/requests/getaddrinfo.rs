use crate::{FromInner, IntoInner};
use uv::{addrinfo, uv_freeaddrinfo, uv_getaddrinfo, uv_getaddrinfo_t};

/// Additional data stored on the request
pub(crate) struct GetAddrInfoDataFields {
    cb: Option<Box<dyn FnMut(GetAddrInfoReq, i32, crate::AddrInfo)>>,
}

/// Callback for uv_getaddrinfo
extern "C" fn uv_getaddrinfo_cb(req: *mut uv_getaddrinfo_t, status: i32, res: *mut addrinfo) {
    let dataptr = crate::Req::get_data(uv_handle!(req));
    if !dataptr.is_null() {
        unsafe {
            if let super::GetAddrInfoData(d) = *dataptr {
                if let Some(f) = d.cb.as_mut() {
                    f(req.into_inner(), status, res.into_inner());
                }
            }
        }
    }

    // free memory
    let req = GetAddrInfoReq::from_inner(req);
    req.destroy();
}

/// GetAddrInfo request type
pub struct GetAddrInfoReq {
    req: *mut uv_getaddrinfo_t,
}

impl GetAddrInfoReq {
    /// Create a new GetAddrInfo request
    pub fn new(
        cb: Option<impl FnMut(GetAddrInfoReq, i32, crate::AddrInfo) + 'static>,
    ) -> crate::Result<GetAddrInfoReq> {
        let layout = std::alloc::Layout::new::<uv_getaddrinfo_t>();
        let req = unsafe { std::alloc::alloc(layout) as *mut uv_getaddrinfo_t };
        if req.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let cb = cb.map(|f| Box::new(f) as _);
        crate::Req::initialize_data(
            uv_handle!(req),
            super::GetAddrInfoData(GetAddrInfoDataFields { cb }),
        );

        Ok(GetAddrInfoReq { req })
    }

    pub fn destroy(&mut self) {
        unsafe { uv_freeaddrinfo((*self.req).addrinfo) };
        crate::Req::free_data(uv_handle!(self.req));

        let layout = std::alloc::Layout::new::<uv_getaddrinfo_t>();
        unsafe { std::alloc::dealloc(self.req as _, layout) };
    }
}

impl FromInner<*mut uv_getaddrinfo_t> for GetAddrInfoReq {
    fn from_inner(req: *mut uv_getaddrinfo_t) -> GetAddrInfoReq {
        GetAddrInfoReq { req }
    }
}

impl IntoInner<*mut uv_getaddrinfo_t> for GetAddrInfoReq {
    fn into_inner(self) -> *mut uv_getaddrinfo_t {
        self.req
    }
}

impl IntoInner<*mut uv::uv_req_t> for GetAddrInfoReq {
    fn into_inner(self) -> *mut uv::uv_req_t {
        uv_handle!(self.req)
    }
}

impl From<GetAddrInfoReq> for crate::Req {
    fn from(req: GetAddrInfoReq) -> crate::Req {
        crate::Req::from_inner(req.into_inner())
    }
}

impl crate::ReqTrait for GetAddrInfoReq {}

impl crate::Loop {}
