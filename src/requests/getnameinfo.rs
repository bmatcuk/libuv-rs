use crate::{FromInner, IntoInner};
use std::ffi::CStr;
use std::net::SocketAddr;
use uv::{uv_getnameinfo, uv_getnameinfo_t};

/// Additional data stored on the request
pub(crate) struct GetNameInfoDataFields {
    cb: Option<Box<dyn FnMut(GetNameInfoReq, i32, String, String)>>,
}

/// Callback for uv_getnameinfo
extern "C" fn uv_getnameinfo_cb(
    req: *mut uv_getnameinfo_t,
    status: i32,
    hostname: *const std::os::raw::c_char,
    service: *const std::os::raw::c_char,
) {
    let dataptr = crate::Req::get_data(uv_handle!(req));
    if !dataptr.is_null() {
        unsafe {
            if let super::GetNameInfoData(d) = *dataptr {
                if let Some(f) = d.cb.as_mut() {
                    let hostname = CStr::from_ptr(hostname).to_string_lossy().into_owned();
                    let service = CStr::from_ptr(service).to_string_lossy().into_owned();
                    f(req.into_inner(), status, hostname, service);
                }
            }
        }
    }

    // free memory
    let req = GetNameInfoReq::from_inner(req);
    req.destroy();
}

/// GetNameInfo request type
pub struct GetNameInfoReq {
    req: *mut uv_getnameinfo_t,
}

impl GetNameInfoReq {
    /// Create a new GetNameInfo request
    pub fn new(
        cb: Option<impl FnMut(GetNameInfoReq, i32, String, String) + 'static>,
    ) -> crate::Result<GetNameInfoReq> {
        let layout = std::alloc::Layout::new::<uv_getnameinfo_t>();
        let req = unsafe { std::alloc::alloc(layout) as *mut uv_getnameinfo_t };
        if req.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let cb = cb.map(|f| Box::new(f) as _);
        crate::Req::initialize_data(
            uv_handle!(req),
            super::GetNameInfoData(GetNameInfoDataFields { cb }),
        );

        Ok(GetNameInfoReq { req })
    }

    /// Returns the host result
    pub fn host(&self) -> String {
        // converting [i8] to [u8] is difficult
        let host = &(*self.req).host;
        CStr::from_ptr(host.as_ptr() as _)
            .to_string_lossy()
            .into_owned()
    }

    /// Returns the service result
    pub fn service(&self) -> String {
        let service = &(*self.req).service;
        CStr::from_ptr(service.as_ptr() as _)
            .to_string_lossy()
            .into_owned()
    }

    /// Free memory associated with this request
    pub fn destroy(&mut self) {
        if !self.req.is_null() {
            crate::Req::free_data(uv_handle!(self.req));

            let layout = std::alloc::Layout::new::<uv_getnameinfo_t>();
            unsafe { std::alloc::dealloc(self.req as _, layout) };
            self.req = std::ptr::null_mut();
        }
    }
}

impl FromInner<*mut uv_getnameinfo_t> for GetNameInfoReq {
    fn from_inner(req: *mut uv_getnameinfo_t) -> GetNameInfoReq {
        GetNameInfoReq { req }
    }
}

impl IntoInner<*mut uv_getnameinfo_t> for GetNameInfoReq {
    fn into_inner(self) -> *mut uv_getnameinfo_t {
        self.req
    }
}

impl IntoInner<*mut uv::uv_req_t> for GetNameInfoReq {
    fn into_inner(self) -> *mut uv::uv_req_t {
        uv_handle!(self.req)
    }
}

impl From<GetNameInfoReq> for crate::Req {
    fn from(req: GetNameInfoReq) -> crate::Req {
        crate::Req::from_inner(IntoInner::<*mut uv::uv_req_t>::into_inner(req))
    }
}

impl crate::ReqTrait for GetNameInfoReq {}

impl crate::Loop {
    /// Private implementation for getnameinfo()
    fn _getnameinfo(
        &self,
        addr: &SocketAddr,
        flags: u32,
        cb: Option<impl FnMut(GetNameInfoReq, i32, String, String) + 'static>,
    ) -> crate::Result<GetNameInfoReq> {
        let sockaddr: uv::sockaddr = std::mem::zeroed();
        crate::fill_sockaddr(&mut sockaddr, addr);

        let req = GetNameInfoReq::new(cb)?;
        let uv_cb = cb.as_ref().map(|_| uv_getnameinfo_cb as _);
        let result = crate::uvret(unsafe {
            uv_getnameinfo(
                self.into_inner(),
                req.into_inner(),
                uv_cb,
                &sockaddr as _,
                flags as _,
            )
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }

    /// Asynchronous getnameinfo(3).
    ///
    /// If successful, the callback will get called sometime in the future with the lookup result.
    /// Consult man -s 3 getnameinfo for more details.
    ///
    /// flags is the bitwise OR of NI_* constants
    pub fn getnameinfo(
        &self,
        addr: &SocketAddr,
        flags: u32,
        cb: impl FnMut(GetNameInfoReq, i32, String, String) + 'static,
    ) -> crate::Result<GetNameInfoReq> {
        self._getnameinfo(addr, flags, Some(cb))
    }

    /// Synchronous getnameinfo(3).
    ///
    /// If successful, will return a tuple of (host, service) Strings.
    ///
    /// flags is the bitwise OR of NI_* constants
    pub fn getnameinfo_sync(
        &self,
        addr: &SocketAddr,
        flags: u32,
    ) -> crate::Result<(String, String)> {
        self._getnameinfo(addr, flags, None::<fn(GetNameInfoReq, i32, String, String)>)
            .map(|req| {
                let res = (req.host(), req.service());
                req.destroy();
                return res;
            })
    }
}
