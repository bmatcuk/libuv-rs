use crate::{FromInner, Inner, IntoInner};
use std::ffi::CStr;
use std::net::SocketAddr;
use uv::{uv_getnameinfo, uv_getnameinfo_t};

callbacks! {
    pub GetNameInfoCB(
        req: GetNameInfoReq,
        status: crate::Result<u32>,
        hostname: String,
        service: String
    );
}

/// Additional data stored on the request
pub(crate) struct GetNameInfoDataFields<'a> {
    cb: GetNameInfoCB<'a>,
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
            if let super::GetNameInfoData(d) = &mut *dataptr {
                let hostname = CStr::from_ptr(hostname).to_string_lossy().into_owned();
                let service = CStr::from_ptr(service).to_string_lossy().into_owned();
                let status = if status < 0 {
                    Err(crate::Error::from_inner(status as uv::uv_errno_t))
                } else {
                    Ok(status as _)
                };
                d.cb.call(req.into_inner(), status, hostname, service);
            }
        }
    }

    // free memory
    let mut req = GetNameInfoReq::from_inner(req);
    req.destroy();
}

/// GetNameInfo request type
#[derive(Clone, Copy)]
pub struct GetNameInfoReq {
    req: *mut uv_getnameinfo_t,
}

impl GetNameInfoReq {
    /// Create a new GetNameInfo request
    pub fn new<CB: Into<GetNameInfoCB<'static>>>(cb: CB) -> crate::Result<GetNameInfoReq> {
        let layout = std::alloc::Layout::new::<uv_getnameinfo_t>();
        let req = unsafe { std::alloc::alloc(layout) as *mut uv_getnameinfo_t };
        if req.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let cb = cb.into();
        crate::Req::initialize_data(
            uv_handle!(req),
            super::GetNameInfoData(GetNameInfoDataFields { cb }),
        );

        Ok(GetNameInfoReq { req })
    }

    /// Loop that started this getnameinfo request and where completion will be reported.
    pub fn r#loop(&self) -> crate::Loop {
        unsafe { (*self.req).loop_ }.into_inner()
    }

    /// Returns the host result
    pub fn host(&self) -> String {
        unsafe {
            // converting [i8] to [u8] is difficult
            let host = &(*self.req).host;
            CStr::from_ptr(host.as_ptr() as _)
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Returns the service result
    pub fn service(&self) -> String {
        unsafe {
            let service = &(*self.req).service;
            CStr::from_ptr(service.as_ptr() as _)
                .to_string_lossy()
                .into_owned()
        }
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

impl Inner<*mut uv_getnameinfo_t> for GetNameInfoReq {
    fn inner(&self) -> *mut uv_getnameinfo_t {
        self.req
    }
}

impl Inner<*mut uv::uv_req_t> for GetNameInfoReq {
    fn inner(&self) -> *mut uv::uv_req_t {
        uv_handle!(self.req)
    }
}

impl From<GetNameInfoReq> for crate::Req {
    fn from(req: GetNameInfoReq) -> crate::Req {
        crate::Req::from_inner(Inner::<*mut uv::uv_req_t>::inner(&req))
    }
}

impl crate::ToReq for GetNameInfoReq {
    fn to_req(&self) -> crate::Req {
        crate::Req::from_inner(Inner::<*mut uv::uv_req_t>::inner(self))
    }
}

impl crate::ReqTrait for GetNameInfoReq {}

impl crate::Loop {
    /// Private implementation for getnameinfo()
    fn _getnameinfo<CB: Into<GetNameInfoCB<'static>>>(
        &self,
        addr: &SocketAddr,
        flags: u32,
        cb: CB,
    ) -> Result<GetNameInfoReq, Box<dyn std::error::Error>> {
        let mut sockaddr: uv::sockaddr = unsafe { std::mem::zeroed() };
        crate::fill_sockaddr(&mut sockaddr, addr)?;

        let cb = cb.into();
        let uv_cb = use_c_callback!(uv_getnameinfo_cb, cb);
        let mut req = GetNameInfoReq::new(cb)?;
        let result = crate::uvret(unsafe {
            uv_getnameinfo(
                self.into_inner(),
                req.inner(),
                uv_cb,
                &sockaddr as _,
                flags as _,
            )
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req).map_err(|e| Box::new(e) as _)
    }

    /// Asynchronous getnameinfo(3).
    ///
    /// If successful, the callback will get called sometime in the future with the lookup result.
    /// Consult man -s 3 getnameinfo for more details.
    ///
    /// flags is the bitwise OR of NI_* constants
    pub fn getnameinfo<CB: Into<GetNameInfoCB<'static>>>(
        &self,
        addr: &SocketAddr,
        flags: u32,
        cb: CB,
    ) -> Result<GetNameInfoReq, Box<dyn std::error::Error>> {
        self._getnameinfo(addr, flags, cb)
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
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        self._getnameinfo(addr, flags, ()).map(|mut req| {
            let res = (req.host(), req.service());
            req.destroy();
            return res;
        })
    }
}
