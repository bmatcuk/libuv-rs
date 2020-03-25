use crate::{FromInner, Inner, IntoInner};
use std::ffi::CStr;
use uv::{
    uv_fs_get_path, uv_fs_get_ptr, uv_fs_get_result, uv_fs_get_statbuf, uv_fs_get_type,
    uv_fs_req_cleanup, uv_fs_t,
};

/// Additional data stored on the request
pub(crate) struct FsDataFields {
    fs_cb: Option<Box<dyn FnMut(FsReq)>>,
}

/// Callback for various fs functions
pub(crate) extern "C" fn uv_fs_cb(req: *mut uv_fs_t) {
    let dataptr = crate::Req::get_data(uv_handle!(req));
    if !dataptr.is_null() {
        unsafe {
            if let super::FsData(d) = &mut *dataptr {
                if let Some(f) = d.fs_cb.as_mut() {
                    f(req.into_inner());
                }
            }
        }
    }

    // free memory
    let mut req = FsReq::from_inner(req);
    req.destroy();
}

/// File system request type.
pub struct FsReq {
    req: *mut uv_fs_t,
}

impl FsReq {
    /// Create a new fs request
    pub fn new(cb: Option<impl FnMut(FsReq) + 'static>) -> crate::Result<FsReq> {
        let layout = std::alloc::Layout::new::<uv_fs_t>();
        let req = unsafe { std::alloc::alloc(layout) as *mut uv_fs_t };
        if req.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let fs_cb = cb.map(|f| Box::new(f) as _);
        crate::Req::initialize_data(uv_handle!(req), super::FsData(FsDataFields { fs_cb }));

        Ok(FsReq { req })
    }

    /// Type of request that was made
    pub fn request_type(&self) -> crate::FsType {
        unsafe { uv_fs_get_type(self.req).into_inner() }
    }

    /// Returns the result from the request
    pub fn result(&self) -> isize {
        unsafe { uv_fs_get_result(self.req) }
    }

    /// Returns the file handle from the request
    pub fn file(&self) -> crate::File {
        unsafe { (*self.req).file }
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

impl crate::ReqTrait for FsReq {}
