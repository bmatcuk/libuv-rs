include!("./req_types.inc.rs");

use crate::{FromInner, Inner, IntoInner};
use std::ffi::CStr;
use uv::{
    uv_cancel, uv_req_get_data, uv_req_get_type, uv_req_set_data, uv_req_t, uv_req_type_name,
};

impl ReqType {
    /// Returns the name of the request type.
    pub fn name(&self) -> String {
        unsafe {
            CStr::from_ptr(uv_req_type_name(self.into_inner()))
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
#[derive(Clone, Copy)]
pub struct Req {
    req: *mut uv_req_t,
}

impl Req {
    /// Initialize the request's data.
    pub(crate) fn initialize_data(req: *mut uv_req_t, data: super::ReqData) {
        let ptr = Box::into_raw(Box::new(data));
        unsafe { uv_req_set_data(req, ptr as _) }
    }

    /// Retrieve the request's data.
    pub(crate) fn get_data(req: *mut uv_req_t) -> *mut super::ReqData {
        unsafe { uv_req_get_data(req) as _ }
    }

    /// Free the request's data.
    pub(crate) fn free_data(req: *mut uv_req_t) {
        let ptr = Req::get_data(req);
        std::mem::drop(unsafe { Box::from_raw(ptr) });
        unsafe { uv_req_set_data(req, std::ptr::null_mut()) };
    }
}

pub trait ToReq {
    fn to_req(&self) -> Req;
}

impl FromInner<*mut uv_req_t> for Req {
    fn from_inner(req: *mut uv_req_t) -> Req {
        Req { req }
    }
}

impl Inner<*mut uv_req_t> for Req {
    fn inner(&self) -> *mut uv_req_t {
        self.req
    }
}

impl ToReq for Req {
    fn to_req(&self) -> Req {
        Req { req: self.req }
    }
}

pub trait ReqTrait: ToReq {
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
        crate::uvret(unsafe { uv_cancel(self.to_req().inner()) })
    }

    /// Returns the type of the request.
    fn get_type(&self) -> ReqType {
        unsafe { uv_req_get_type(self.to_req().inner()).into_inner() }
    }
}

impl ReqTrait for Req {}
