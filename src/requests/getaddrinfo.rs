use crate::{FromInner, Inner, IntoInner};
use std::ffi::CString;
use uv::{addrinfo, uv_freeaddrinfo, uv_getaddrinfo, uv_getaddrinfo_t};

callbacks! {
    pub GetAddrInfoCB(
        req: GetAddrInfoReq,
        status: crate::Result<u32>,
        res: Vec<crate::AddrInfo>
    );
}

/// Additional data stored on the request
pub(crate) struct GetAddrInfoDataFields<'a> {
    cb: GetAddrInfoCB<'a>,
}

/// Callback for uv_getaddrinfo
extern "C" fn uv_getaddrinfo_cb(req: *mut uv_getaddrinfo_t, status: i32, res: *mut addrinfo) {
    let dataptr = crate::Req::get_data(uv_handle!(req));
    if !dataptr.is_null() {
        unsafe {
            if let super::GetAddrInfoData(d) = &mut *dataptr {
                let status = if status < 0 {
                    Err(crate::Error::from_inner(status as uv::uv_errno_t))
                } else {
                    Ok(status as _)
                };
                let res = res.into_inner();
                d.cb.call(req.into_inner(), status, res);
            }
        }
    }

    // free memory
    let mut req = GetAddrInfoReq::from_inner(req);
    req.destroy();

    unsafe { uv_freeaddrinfo(res) };
}

/// GetAddrInfo request type
#[derive(Clone, Copy)]
pub struct GetAddrInfoReq {
    req: *mut uv_getaddrinfo_t,
}

impl GetAddrInfoReq {
    /// Create a new GetAddrInfo request
    pub fn new<CB: Into<GetAddrInfoCB<'static>>>(cb: CB) -> crate::Result<GetAddrInfoReq> {
        let layout = std::alloc::Layout::new::<uv_getaddrinfo_t>();
        let req = unsafe { std::alloc::alloc(layout) as *mut uv_getaddrinfo_t };
        if req.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let cb = cb.into();
        crate::Req::initialize_data(
            uv_handle!(req),
            super::GetAddrInfoData(GetAddrInfoDataFields { cb }),
        );

        Ok(GetAddrInfoReq { req })
    }

    /// Frees memory associated with this request
    pub fn destroy(&mut self) {
        if !self.req.is_null() {
            // do I need this?
            // unsafe { uv_freeaddrinfo((*self.req).addrinfo) };
            crate::Req::free_data(uv_handle!(self.req));

            let layout = std::alloc::Layout::new::<uv_getaddrinfo_t>();
            unsafe { std::alloc::dealloc(self.req as _, layout) };
            self.req = std::ptr::null_mut();
        }
    }

    /// Retrieve an iterator of AddrInfo responses
    pub fn addrinfos(self) -> Vec<crate::AddrInfo> {
        let ai = unsafe { (*self.req).addrinfo };
        ai.into_inner()
    }

    /// The loop
    pub fn r#loop(&self) -> crate::Loop {
        unsafe { (*self.req).loop_.into_inner() }
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

impl crate::ToReq for GetAddrInfoReq {
    fn to_req(&self) -> crate::Req {
        crate::Req::from_inner(Inner::<*mut uv::uv_req_t>::inner(self))
    }
}

impl crate::ReqTrait for GetAddrInfoReq {}

impl crate::Loop {
    /// Private implementation for getaddrinfo()
    fn _getaddrinfo<CB: Into<GetAddrInfoCB<'static>>>(
        &self,
        node: Option<&str>,
        service: Option<&str>,
        hints: Option<crate::AddrInfo>,
        cb: CB,
    ) -> Result<GetAddrInfoReq, Box<dyn std::error::Error>> {
        let cb = cb.into();
        let uv_cb = use_c_callback!(uv_getaddrinfo_cb, cb);
        let node = node.map(CString::new).transpose()?;
        let service = service.map(CString::new).transpose()?;
        let mut req = GetAddrInfoReq::new(cb)?;
        let hints = hints.map(|h| h.into_inner());
        let result = crate::uvret(unsafe {
            uv_getaddrinfo(
                self.into_inner(),
                req.inner(),
                uv_cb,
                if let Some(node) = node.as_ref() {
                    node.as_ptr()
                } else {
                    std::ptr::null()
                },
                if let Some(service) = service.as_ref() {
                    service.as_ptr()
                } else {
                    std::ptr::null()
                },
                if let Some(hints) = hints.as_ref() {
                    hints as _
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
    pub fn getaddrinfo<CB: Into<GetAddrInfoCB<'static>>>(
        &self,
        node: Option<&str>,
        service: Option<&str>,
        hints: Option<crate::AddrInfo>,
        cb: CB,
    ) -> Result<GetAddrInfoReq, Box<dyn std::error::Error>> {
        self._getaddrinfo(node, service, hints, cb)
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
    ) -> Result<Vec<crate::AddrInfo>, Box<dyn std::error::Error>> {
        self._getaddrinfo(node, service, hints, ())
            .map(|req| req.addrinfos())
    }
}
