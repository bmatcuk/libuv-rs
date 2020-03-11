use crate::{FromInner, IntoInner};
use std::ffi::CString;
use uv::{uv_fs_poll_getpath, uv_fs_poll_init, uv_fs_poll_start, uv_fs_poll_stop, uv_fs_poll_t};

/// Additional data stored on the handle
#[derive(Default)]
pub(crate) struct FsPollDataFields {
    fs_poll_cb: Option<Box<dyn FnMut(FsPollHandle, i32, crate::Stat, crate::Stat)>>,
}

/// Callback for uv_fs_poll_start
extern "C" fn uv_fs_poll_cb(
    handle: *mut uv_fs_poll_t,
    status: std::os::raw::c_int,
    prev: *const uv::uv_stat_t,
    curr: *const uv::uv_stat_t,
) {
    let dataptr = crate::Handle::get_data(uv_handle!(handle));
    if !dataptr.is_null() {
        unsafe {
            if let super::FsPollData(d) = &mut (*dataptr).addl {
                if let Some(f) = d.fs_poll_cb.as_mut() {
                    f(
                        handle.into_inner(),
                        status as _,
                        prev.into_inner(),
                        curr.into_inner(),
                    )
                }
            }
        }
    }
}

/// FS Poll handles allow the user to monitor a given path for changes. Unlike FsEventHandle, fs
/// poll handles use stat to detect when a file has changed so they can work on file systems where
/// fs event handles can’t.
pub struct FsPollHandle {
    handle: *mut uv_fs_poll_t,
}

impl FsPollHandle {
    /// Create and initialize a new fs poll handle
    pub fn new(r#loop: &crate::Loop) -> crate::Result<FsPollHandle> {
        let layout = std::alloc::Layout::new::<uv_fs_poll_t>();
        let handle = unsafe { std::alloc::alloc(layout) as *mut uv_fs_poll_t };
        if handle.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let ret = unsafe { uv_fs_poll_init(r#loop.into_inner(), handle) };
        if ret < 0 {
            unsafe { std::alloc::dealloc(handle as _, layout) };
            return Err(crate::Error::from_inner(ret as uv::uv_errno_t));
        }

        crate::Handle::initialize_data(uv_handle!(handle), super::FsPollData(Default::default()));

        Ok(FsPollHandle { handle })
    }

    /// Check the file at path for changes every interval milliseconds.
    ///
    /// Note: For maximum portability, use multi-second intervals. Sub-second intervals will not
    /// detect all changes on many file systems.
    pub fn start(
        &mut self,
        path: &str,
        interval: u32,
        cb: Option<impl FnMut(FsPollHandle, i32, crate::Stat, crate::Stat) + 'static>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = CString::new(path)?;

        // uv_cb is either Some(fs_poll_cb) or None
        let uv_cb = cb.as_ref().map(|_| uv_fs_poll_cb as _);

        // cb is either Some(closure) or None - it is saved into data
        let cb = cb.map(|f| Box::new(f) as _);
        let dataptr = crate::Handle::get_data(uv_handle!(self.handle));
        if !dataptr.is_null() {
            if let super::FsPollData(d) = unsafe { &mut (*dataptr).addl } {
                d.fs_poll_cb = cb;
            }
        }

        crate::uvret(unsafe { uv_fs_poll_start(self.handle, uv_cb, path.as_ptr(), interval as _) })
            .map_err(|e| Box::new(e) as _)
    }

    /// Stop the handle, the callback will no longer be called.
    pub fn stop(&mut self) -> crate::Result<()> {
        crate::uvret(unsafe { uv_fs_poll_stop(self.handle) })
    }

    /// Get the path being monitored by the handle.
    pub fn getpath(&self) -> crate::Result<String> {
        // retrieve the size of the buffer we need to allocate
        let mut size = 0usize;
        unsafe { uv_fs_poll_getpath(self.handle, std::ptr::null_mut(), &mut size as _) };

        let mut buf: Vec<std::os::raw::c_uchar> = Vec::with_capacity(size);
        crate::uvret(unsafe {
            uv_fs_poll_getpath(self.handle, buf.as_mut_ptr() as _, &mut size as _)
        })
        .map(|_| {
            unsafe { CString::from_vec_unchecked(buf) }
                .to_string_lossy()
                .into_owned()
        })
    }
}

impl FromInner<*mut uv_fs_poll_t> for FsPollHandle {
    fn from_inner(handle: *mut uv_fs_poll_t) -> FsPollHandle {
        FsPollHandle { handle }
    }
}

impl IntoInner<*mut uv::uv_handle_t> for FsPollHandle {
    fn into_inner(self) -> *mut uv::uv_handle_t {
        uv_handle!(self.handle)
    }
}

impl From<FsPollHandle> for crate::Handle {
    fn from(fs_poll: FsPollHandle) -> crate::Handle {
        crate::Handle::from_inner(fs_poll.into_inner())
    }
}

impl crate::HandleTrait for FsPollHandle {}

impl crate::Loop {
    /// Create and initialize a fs poll handle
    pub fn fs_poll(&self) -> crate::Result<FsPollHandle> {
        FsPollHandle::new(self)
    }
}
