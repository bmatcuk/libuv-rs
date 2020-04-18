use crate::{FromInner, Inner, IntoInner};
use uv::uv_connect_t;

/// Additional data stored on the request
pub(crate) struct ConnectDataFields {
    connect_cb: Option<Box<dyn FnMut(ConnectReq, crate::Result<u32>)>>,
}

/// Callback for uv_tcp_connect
pub(crate) extern "C" fn uv_connect_cb(req: *mut uv_connect_t, status: std::os::raw::c_int) {
    let dataptr = crate::Req::get_data(uv_handle!(req));
    if !dataptr.is_null() {
        unsafe {
            if let super::ConnectData(d) = &mut *dataptr {
                if let Some(f) = d.connect_cb.as_mut() {
                    let status = if status < 0 {
                        Err(crate::Error::from_inner(status as uv::uv_errno_t))
                    } else {
                        Ok(status as _)
                    };
                    f(req.into_inner(), status);
                }
            }
        }
    }

    // free memory
    let mut req = ConnectReq::from_inner(req);
    req.destroy();
}

/// Connect request type
pub struct ConnectReq {
    req: *mut uv_connect_t,
}

impl ConnectReq {
    /// Create a new connect request
    pub fn new(
        cb: Option<impl FnMut(ConnectReq, crate::Result<u32>) + 'static>,
    ) -> crate::Result<ConnectReq> {
        let layout = std::alloc::Layout::new::<uv_connect_t>();
        let req = unsafe { std::alloc::alloc(layout) as *mut uv_connect_t };
        if req.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let connect_cb = cb.map(|f| Box::new(f) as _);
        crate::Req::initialize_data(
            uv_handle!(req),
            super::ConnectData(ConnectDataFields { connect_cb }),
        );

        Ok(ConnectReq { req })
    }

    /// The stream where this connection request is running
    pub fn handle(&self) -> crate::StreamHandle {
        unsafe { (*self.req).handle }.into_inner()
    }

    pub fn destroy(&mut self) {
        crate::Req::free_data(uv_handle!(self.req));

        let layout = std::alloc::Layout::new::<uv_connect_t>();
        unsafe { std::alloc::dealloc(self.req as _, layout) };
    }
}

impl FromInner<*mut uv_connect_t> for ConnectReq {
    fn from_inner(req: *mut uv_connect_t) -> ConnectReq {
        ConnectReq { req }
    }
}

impl Inner<*mut uv_connect_t> for ConnectReq {
    fn inner(&self) -> *mut uv_connect_t {
        self.req
    }
}

impl Inner<*mut uv::uv_req_t> for ConnectReq {
    fn inner(&self) -> *mut uv::uv_req_t {
        uv_handle!(self.req)
    }
}

impl From<ConnectReq> for crate::Req {
    fn from(connect: ConnectReq) -> crate::Req {
        crate::Req::from_inner(Inner::<*mut uv::uv_req_t>::inner(&connect))
    }
}

impl crate::ToReq for ConnectReq {
    fn to_req(&self) -> crate::Req {
        crate::Req::from_inner(Inner::<*mut uv::uv_req_t>::inner(self))
    }
}

impl crate::ReqTrait for ConnectReq {}
