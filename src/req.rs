include!("./req_types.inc.rs");

use crate::{FromInner, IntoInner};
use std::ffi::CStr;
use uv::{
    uv_cancel, uv_req_get_data, uv_req_get_type, uv_req_set_data, uv_req_t, uv_req_type_name,
};

impl ReqType {
    /// Returns the name of the request type.
    pub fn name(&self) -> String {
        unsafe {
            CStr::from_ptr(uv_req_type_name(self.into()))
                .to_string_lossy()
                .into_owned()
        }
    }
}

impl std::fmt::Display for ReqType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name())
    }
}

/// Req is the base type for all libuv requests
pub struct Req {
    req: *mut uv_req_t,
}

impl Req {
    /// Initialize the request's data.
    pub(crate) fn initialize_data(req: *mut uv_req_t, data: crate::ReqData) {
        let data = Box::new(data);
        let ptr = Box::into_raw(data);
        unsafe { uv_req_set_data(req, ptr as _) }
    }

    /// Retrieve the request's data.
    pub(crate) fn get_data(req: *mut uv_req_t) -> *mut crate::ReqData {
        unsafe { uv_req_get_data(req) as _ }
    }

    /// Free the request's data.
    pub(crate) fn free_data(req: *mut uv_req_t) {
        let ptr = Req::get_data(req);
        std::mem::drop(unsafe { Box::from_raw(ptr) });
        unsafe { uv_req_set_data(req, std::ptr::null_mut()) };
    }
}

impl FromInner<*mut uv_req_t> for Req {
    fn from_inner(req: *mut uv_req_t) -> Req {
        Req { req }
    }
}

impl IntoInner<*mut uv_req_t> for Req {
    fn into_inner(self) -> *mut uv_req_t {
        self.req
    }
}

pub trait ReqTrait: IntoInner<*mut uv_req_t> {
    /// Cancel a pending request. Fails if the request is executing or has finished executing.
    ///
    /// Only cancellation of FsReq, GetAddrInfoReq, GetNameInfoReq and WorkReq requests is
    /// currently supported.
    ///
    /// Cancelled requests have their callbacks invoked some time in the future. It’s not safe to
    /// free the memory associated with the request until the callback is called.
    ///
    /// Here is how cancellation is reported to the callback:
    ///   * A FsReq request has its req->result field set to UV_ECANCELED.
    ///   * A WorkReq, GetAddrInfoReq or GetNameInfoReq request has its callback invoked with
    ///     status == UV_ECANCELED.
    fn cancel(&mut self) -> crate::Result<()> {
        crate::uvret(unsafe { uv_cancel((*self).into_inner()) })
    }

    /// Returns the type of the request.
    fn get_type(&self) -> ReqType {
        unsafe { uv_req_get_type((*self).into_inner()).into() }
    }
}

impl ReqTrait for Req {}
