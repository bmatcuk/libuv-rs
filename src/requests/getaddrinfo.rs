use crate::{FromInner, Inner, IntoInner};
use std::ffi::CString;
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

    /// Frees memory associated with this request
    pub fn destroy(&mut self) {
        if !self.req.is_null() {
            unsafe { uv_freeaddrinfo((*self.req).addrinfo) };
            crate::Req::free_data(uv_handle!(self.req));

            let layout = std::alloc::Layout::new::<uv_getaddrinfo_t>();
            unsafe { std::alloc::dealloc(self.req as _, layout) };
            self.req = std::ptr::null_mut();
        }
    }

    /// Retrieve an iterator of AddrInfo responses
    pub fn addrinfos(&self) -> AddrInfoIter {
        AddrInfoIter {
            req: *self,
            ai: (*self.req).addrinfo,
        }
    }
}

impl FromInner<*mut uv_getaddrinfo_t> for GetAddrInfoReq {
    fn from_inner(req: *mut uv_getaddrinfo_t) -> GetAddrInfoReq {
        GetAddrInfoReq { req }
    }
}

impl Inner<*mut uv_getaddrinfo_t> for GetAddrInfoReq {
    fn inner(&self) -> *mut uv_getaddrinfo_t {
        self.req
    }
}

impl Inner<*mut uv::uv_req_t> for GetAddrInfoReq {
    fn inner(&self) -> *mut uv::uv_req_t {
        uv_handle!(self.req)
    }
}

impl From<GetAddrInfoReq> for crate::Req {
    fn from(req: GetAddrInfoReq) -> crate::Req {
        crate::Req::from_inner(Inner::<*mut uv::uv_req_t>::inner(&req))
    }
}

impl crate::ReqTrait for GetAddrInfoReq {}

pub struct AddrInfoIter {
    req: GetAddrInfoReq,
    ai: *mut addrinfo,
}

impl Iterator for AddrInfoIter {
    type Item = crate::AddrInfo;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ai.is_null() {
            return None;
        }

        let ai = self.ai.into_inner();
        self.ai = (*self.ai).ai_next;
        Some(ai)
    }
}

impl std::iter::FusedIterator for AddrInfoIter {}

impl Drop for AddrInfoIter {
    fn drop(&mut self) {
        self.req.destroy();
    }
}

impl crate::Loop {
    /// Private implementation for getaddrinfo()
    fn _getaddrinfo(
        &self,
        node: Option<&str>,
        service: Option<&str>,
        hints: Option<crate::AddrInfo>,
        cb: Option<impl FnMut(GetAddrInfoReq, i32, crate::AddrInfo) + 'static>,
    ) -> Result<GetAddrInfoReq, Box<dyn std::error::Error>> {
        let node = node.map(CString::new).transpose()?;
        let service = service.map(CString::new).transpose()?;
        let req = GetAddrInfoReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| uv_getaddrinfo_cb as _);
        let hints = hints.map(|h| h.into_inner());
        let result = crate::uvret(unsafe {
            uv_getaddrinfo(
                self.into_inner(),
                req.inner(),
                uv_cb,
                if let Some(node) = node {
                    node.as_ptr()
                } else {
                    std::ptr::null()
                },
                if let Some(service) = service {
                    service.as_ptr()
                } else {
                    std::ptr::null()
                },
                if let Some(hints) = hints {
                    &hints as _
                } else {
                    std::ptr::null()
                },
            )
        })
        .map_err(|e| Box::new(e) as _);
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Asynchronous getaddrinfo(3).
    ///
    /// Either node or service may be None but not both.
    ///
    /// hints is a AddrInfo with additional address type constraints, or None. Consult man -s 3
    /// getaddrinfo for more details.
    pub fn getaddrinfo(
        &self,
        node: Option<&str>,
        service: Option<&str>,
        hints: Option<crate::AddrInfo>,
        cb: impl FnMut(GetAddrInfoReq, i32, crate::AddrInfo) + 'static,
    ) -> Result<GetAddrInfoReq, Box<dyn std::error::Error>> {
        self._getaddrinfo(node, service, hints, Some(cb))
    }

    /// Synchronous getaddrinfo(3).
    ///
    /// Either node or service may be None but not both.
    ///
    /// hints is a AddrInfo with additional address type constraints, or None. Consult man -s 3
    /// getaddrinfo for more details.
    ///
    /// Returns an iterator over resulting AddrInfo structs.
    pub fn getaddrinfo_sync(
        &self,
        node: Option<&str>,
        service: Option<&str>,
        hints: Option<crate::AddrInfo>,
    ) -> Result<AddrInfoIter, Box<dyn std::error::Error>> {
        self._getaddrinfo(
            node,
            service,
            hints,
            None::<fn(GetAddrInfoReq, i32, crate::AddrInfo)>,
        )
        .map(|req| req.addrinfos())
    }
}
