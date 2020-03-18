use crate::{FromInner, IntoInner};
use uv::uv_udp_send_t;

/// Additional data stored on the request
pub(crate) struct UdpSendDataFields {
    bufs_ptr: *mut uv::uv_buf_t,
    bufs_len: usize,
    bufs_capacity: usize,
    udp_send_cb: Option<Box<dyn FnMut(UdpSendReq, i32)>>,
}

/// Callback for uv_udp_send
pub(crate) extern "C" fn uv_udp_send_cb(req: *mut uv_udp_send_t, status: std::os::raw::c_int) {
    let dataptr = crate::Req::get_data(uv_handle!(req));
    if !dataptr.is_null() {
        unsafe {
            if let super::UdpSendData(d) = *dataptr {
                if let Some(f) = d.udp_send_cb.as_mut() {
                    f(req.into_inner(), status as _);
                }
            }
        }
    }

    // free memory
    let req = UdpSendReq::from_inner(req);
    req.destroy();
}

pub struct UdpSendReq {
    req: *mut uv_udp_send_t,

    /// This is only guaranteed to be set if the UdpSendReq was created by new(). If it was created
    /// any other way (such as by From<*mut uv_udp_send_t>), it will not be set.
    pub(crate) bufs_ptr: *const uv::uv_buf_t,
}

impl UdpSendReq {
    /// Create a new udp send request
    pub fn new(
        bufs: &[impl crate::BufTrait],
        cb: Option<impl FnMut(UdpSendReq, i32) + 'static>,
    ) -> crate::Result<UdpSendReq> {
        let layout = std::alloc::Layout::new::<uv_udp_send_t>();
        let req = unsafe { std::alloc::alloc(layout) as *mut uv_udp_send_t };
        if req.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let (bufs_ptr, bufs_len, bufs_capacity) = bufs.into_inner();
        let udp_send_cb = cb.map(|f| Box::new(f) as _);
        crate::Req::initialize_data(
            uv_handle!(req),
            super::UdpSendData(UdpSendDataFields {
                bufs_ptr,
                bufs_len,
                bufs_capacity,
                udp_send_cb,
            }),
        );

        Ok(UdpSendReq { req, bufs_ptr })
    }

    pub fn destroy(&mut self) {
        let dataptr = crate::Req::get_data(uv_handle!(self.req));
        if !dataptr.is_null() {
            if let super::UdpSendData(d) = unsafe { *dataptr } {
                if !d.bufs_ptr.is_null() {
                    // This will destroy the Vec<uv_buf_t>, but will not actually deallocate the
                    // uv_buf_t's themselves. That's up to the user to do.
                    std::mem::drop(Vec::from_raw_parts(d.bufs_ptr, d.bufs_len, d.bufs_capacity));
                }
            }
        }

        crate::Req::free_data(uv_handle!(self.req));

        let layout = std::alloc::Layout::new::<uv_udp_send_t>();
        unsafe { std::alloc::dealloc(self.req as _, layout) }
    }
}

impl FromInner<*mut uv_udp_send_t> for UdpSendReq {
    fn from_inner(req: *mut uv_udp_send_t) -> UdpSendReq {
        UdpSendReq {
            req,
            bufs_ptr: std::ptr::null(),
        }
    }
}

impl IntoInner<*mut uv_udp_send_t> for UdpSendReq {
    fn into_inner(self) -> *mut uv_udp_send_t {
        self.req
    }
}

impl IntoInner<*mut uv::uv_req_t> for UdpSendReq {
    fn into_inner(self) -> *mut uv::uv_req_t {
        uv_handle!(self.req)
    }
}

impl From<UdpSendReq> for crate::Req {
    fn from(udp_send: UdpSendReq) -> crate::Req {
        crate::Req::from_inner(IntoInner::<*mut uv::uv_req_t>::into_inner(udp_send))
    }
}

impl crate::ReqTrait for UdpSendReq {}
