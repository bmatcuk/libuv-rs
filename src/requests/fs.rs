use crate::{FromInner, Inner, IntoInner};
use std::ffi::CStr;
use uv::{
    uv_fs_get_path, uv_fs_get_ptr, uv_fs_get_result, uv_fs_get_statbuf, uv_fs_get_type,
    uv_fs_req_cleanup, uv_fs_t,
};

callbacks! {
    pub FsCB(req: FsReq);
}

/// Additional data stored on the request
pub(crate) struct FsDataFields<'a> {
    fs_cb: FsCB<'a>,
}

/// Callback for various fs functions
pub(crate) extern "C" fn uv_fs_cb(req: *mut uv_fs_t) {
    let dataptr = crate::Req::get_data(uv_handle!(req));
    if !dataptr.is_null() {
        unsafe {
            if let super::FsData(d) = &mut *dataptr {
                d.fs_cb.call(req.into_inner());
            }
        }
    }

    // free memory
    let mut req = FsReq::from_inner(req);
    req.destroy();
}

/// File system request type.
#[derive(Clone, Copy)]
pub struct FsReq {
    req: *mut uv_fs_t,
}

impl FsReq {
    /// Create a new fs request
    pub fn new<CB: Into<FsCB<'static>>>(cb: CB) -> crate::Result<FsReq> {
        let layout = std::alloc::Layout::new::<uv_fs_t>();
        let req = unsafe { std::alloc::alloc(layout) as *mut uv_fs_t };
        if req.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let fs_cb = cb.into();
        crate::Req::initialize_data(uv_handle!(req), super::FsData(FsDataFields { fs_cb }));

        Ok(FsReq { req })
    }

    /// Type of request that was made
    pub fn request_type(&self) -> crate::FsType {
        unsafe { uv_fs_get_type(self.req).into_inner() }
    }

    /// The loop that ran this request
    pub fn r#loop(&self) -> crate::Loop {
        unsafe { (*self.req).loop_.into_inner() }
    }

    /// Returns the result from the request
    pub fn result(&self) -> crate::Result<usize> {
        let result = unsafe { uv_fs_get_result(self.req) };
        if result < 0 {
            Err(crate::Error::from_inner(result as uv::uv_errno_t))
        } else {
            Ok(result as _)
        }
    }

    /// Returns the file stats
    pub fn stat(&self) -> crate::Stat {
        unsafe { uv_fs_get_statbuf(self.req) as *const uv::uv_stat_t }.into_inner()
    }

    /// If this request is from opendir() or readdir(), return the Dir struct
    pub fn dir(&self) -> Option<crate::Dir> {
        match self.request_type() {
            crate::FsType::OPENDIR | crate::FsType::READDIR => {
                let ptr: *mut uv::uv_dir_t = unsafe { uv_fs_get_ptr(self.req) } as _;
                Some(ptr.into_inner())
            }
            _ => None,
        }
    }

    /// If this request is from fs_statfs(), return the StatFs struct
    pub fn statfs(&self) -> Option<crate::StatFs> {
        match self.request_type() {
            crate::FsType::STATFS => {
                let ptr: *mut uv::uv_statfs_t = unsafe { uv_fs_get_ptr(self.req) } as _;
                Some(ptr.into_inner())
            }
            _ => None,
        }
    }

    /// If this request is from fs_readlink() or fs_realpath(), return the path
    pub fn real_path(&self) -> Option<String> {
        match self.request_type() {
            crate::FsType::READLINK | crate::FsType::REALPATH => {
                let ptr: *const i8 = unsafe { uv_fs_get_ptr(self.req) } as _;
                Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
            }
            _ => None,
        }
    }

    /// Returns the path of this file
    pub fn path(&self) -> String {
        unsafe {
            let path = uv_fs_get_path(self.req);
            CStr::from_ptr(path).to_string_lossy().into_owned()
        }
    }

    /// Free up memory associated with this request. If you are using one of the async fs_*
    /// functions, this will be called automatically after the callback runs.
    pub fn destroy(&mut self) {
        if !self.req.is_null() {
            crate::Req::free_data(uv_handle!(self.req));
            unsafe { uv_fs_req_cleanup(self.req) };

            let layout = std::alloc::Layout::new::<uv_fs_t>();
            unsafe { std::alloc::dealloc(self.req as _, layout) };
            self.req = std::ptr::null_mut();
        }
    }
}

impl FromInner<*mut uv_fs_t> for FsReq {
    fn from_inner(req: *mut uv_fs_t) -> FsReq {
        FsReq { req }
    }
}

impl Inner<*mut uv_fs_t> for FsReq {
    fn inner(&self) -> *mut uv_fs_t {
        self.req
    }
}

impl Inner<*mut uv::uv_req_t> for FsReq {
    fn inner(&self) -> *mut uv::uv_req_t {
        uv_handle!(self.req)
    }
}

impl From<FsReq> for crate::Req {
    fn from(fs: FsReq) -> crate::Req {
        crate::Req::from_inner(Inner::<*mut uv::uv_req_t>::inner(&fs))
    }
}

impl crate::ToReq for FsReq {
    fn to_req(&self) -> crate::Req {
        crate::Req::from_inner(Inner::<*mut uv::uv_req_t>::inner(self))
    }
}

impl crate::ReqTrait for FsReq {}
