use crate::{FromInner, IntoInner};
use uv::{uv_queue_work, uv_work_t};

/// Additional data stored on the request
pub(crate) struct WorkDataFields {
    work_cb: Option<Box<dyn FnMut(WorkReq)>>,
    after_work_cb: Option<Box<dyn FnMut(WorkReq, i32)>>,
}

/// Callback for uv_queue_work
extern "C" fn uv_work_cb(req: *mut uv_work_t) {
    let dataptr = crate::Req::get_data(uv_handle!(req));
    if !dataptr.is_null() {
        unsafe {
            if let super::WorkData(d) = *dataptr {
                if let Some(f) = d.work_cb.as_mut() {
                    f(req.into_inner());
                }
            }
        }
    }
}

extern "C" fn uv_after_work_cb(req: *mut uv_work_t, status: i32) {
    let dataptr = crate::Req::get_data(uv_handle!(req));
    if !dataptr.is_null() {
        unsafe {
            if let super::WorkData(d) = *dataptr {
                if let Some(f) = d.after_work_cb.as_mut() {
                    f(req.into_inner(), status);
                }
            }
        }
    }

    // free memory
    let req = WorkReq::from_inner(req);
    req.destroy();
}

/// Work request type.
pub struct WorkReq {
    req: *mut uv_work_t,
}

impl WorkReq {
    /// Create a new work request
    pub fn new(
        work_cb: Option<impl FnMut(WorkReq) + 'static>,
        after_work_cb: Option<impl FnMut(WorkReq, i32) + 'static>,
    ) -> crate::Result<WorkReq> {
        let layout = std::alloc::Layout::new::<uv_work_t>();
        let req = unsafe { std::alloc::alloc(layout) as *mut uv_work_t };
        if req.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let work_cb = work_cb.map(|f| Box::new(f) as _);
        let after_work_cb = after_work_cb.map(|f| Box::new(f) as _);
        crate::Req::initialize_data(
            uv_handle!(req),
            super::WorkData(WorkDataFields {
                work_cb,
                after_work_cb,
            }),
        );

        Ok(WorkReq { req })
    }

    pub fn destroy(&mut self) {
        crate::Req::free_data(uv_handle!(self.req));

        let layout = std::alloc::Layout::new::<uv_work_t>();
        unsafe { std::alloc::dealloc(self.req as _, layout) };
    }
}

impl FromInner<*mut uv_work_t> for WorkReq {
    fn from_inner(req: *mut uv_work_t) -> WorkReq {
        WorkReq { req }
    }
}

impl IntoInner<*mut uv_work_t> for WorkReq {
    fn into_inner(self) -> *mut uv_work_t {
        self.req
    }
}

impl IntoInner<*mut uv::uv_req_t> for WorkReq {
    fn into_inner(self) -> *mut uv::uv_req_t {
        uv_handle!(self.req)
    }
}

impl From<WorkReq> for crate::Req {
    fn from(work: WorkReq) -> crate::Req {
        crate::Req::from_inner(work.into_inner())
    }
}

impl crate::ReqTrait for WorkReq {}

impl crate::Loop {
    /// Initializes a work request which will run the given work_cb in a thread from the
    /// threadpool. Once work_cb is completed, after_work_cb will be called on the loop thread.
    ///
    /// This request can be cancelled with Req::cancel().
    pub fn queue_work(
        &self,
        work_cb: Option<impl FnMut(WorkReq) + 'static>,
        after_work_cb: Option<impl FnMut(WorkReq, i32) + 'static>,
    ) -> crate::Result<WorkReq> {
        let req = WorkReq::new(work_cb, after_work_cb)?;
        let uv_work_cb = work_cb.as_ref().map(|_| uv_work_cb as _);
        let uv_after_work_cb = Some(uv_after_work_cb as _);
        let result = crate::uvret(unsafe {
            uv_queue_work(
                self.into_inner(),
                req.into_inner(),
                uv_work_cb,
                uv_after_work_cb,
            )
        });
        if result.is_err() {
            req.destroy();
        }
        result.map(|_| req)
    }
}
