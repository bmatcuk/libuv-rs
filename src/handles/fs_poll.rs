use crate::{FromInner, HandleTrait, Inner, IntoInner};
use std::convert::TryFrom;
use std::ffi::{CStr, CString};
use uv::{uv_fs_poll_getpath, uv_fs_poll_init, uv_fs_poll_start, uv_fs_poll_stop, uv_fs_poll_t};

callbacks! {
    pub FsPollCB(
        handle: FsPollHandle,
        status: crate::Result<u32>,
        prev: crate::Stat,
        curr: crate::Stat
    );
}

/// Additional data stored on the handle
#[derive(Default)]
pub(crate) struct FsPollDataFields<'a> {
    fs_poll_cb: FsPollCB<'a>,
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
                let status = if status < 0 {
                    Err(crate::Error::from_inner(status as uv::uv_errno_t))
                } else {
                    Ok(status as _)
                };

                d.fs_poll_cb.call(
                    handle.into_inner(),
                    status,
                    prev.into_inner(),
                    curr.into_inner(),
                )
            }
        }
    }
}

/// FS Poll handles allow the user to monitor a given path for changes. Unlike FsEventHandle, fs
/// poll handles use stat to detect when a file has changed so they can work on file systems where
/// fs event handles can’t.
#[derive(Clone, Copy)]
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
    pub fn start<CB: Into<FsPollCB<'static>>>(
        &mut self,
        path: &str,
        interval: u32,
        cb: CB,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = CString::new(path)?;

        // uv_cb is either Some(fs_poll_cb) or None
        let cb = cb.into();
        let uv_cb = use_c_callback!(uv_fs_poll_cb, cb);

        // cb is either Some(closure) or None - it is saved into data
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
        let mut size = 0u64;
        let result = crate::uvret(unsafe {
            uv_fs_poll_getpath(self.handle, std::ptr::null_mut(), &mut size as _)
        });
        if let Err(e) = result {
            if e != crate::Error::ENOBUFS {
                return Err(e);
            }
        }

        // On ENOBUFS, size is the length of the required buffer, *including* the null
        let mut buf: Vec<std::os::raw::c_uchar> = Vec::with_capacity(size as _);
        crate::uvret(unsafe {
            uv_fs_poll_getpath(self.handle, buf.as_mut_ptr() as _, &mut size as _)
        })
        .map(|_| {
            // size is the length of the string, *not* including the null
            unsafe { buf.set_len((size as usize) + 1) };
            unsafe { CStr::from_bytes_with_nul_unchecked(&buf) }
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

impl Inner<*mut uv::uv_handle_t> for FsPollHandle {
    fn inner(&self) -> *mut uv::uv_handle_t {
        uv_handle!(self.handle)
    }
}

impl From<FsPollHandle> for crate::Handle {
    fn from(fs_poll: FsPollHandle) -> crate::Handle {
        crate::Handle::from_inner(Inner::<*mut uv::uv_handle_t>::inner(&fs_poll))
    }
}

impl crate::ToHandle for FsPollHandle {
    fn to_handle(&self) -> crate::Handle {
        crate::Handle::from_inner(Inner::<*mut uv::uv_handle_t>::inner(self))
    }
}

impl TryFrom<crate::Handle> for FsPollHandle {
    type Error = crate::ConversionError;

    fn try_from(handle: crate::Handle) -> Result<Self, Self::Error> {
        let t = handle.get_type();
        if t != crate::HandleType::FS_POLL {
            Err(crate::ConversionError::new(t, crate::HandleType::FS_POLL))
        } else {
            Ok((handle.inner() as *mut uv_fs_poll_t).into_inner())
        }
    }
}

impl HandleTrait for FsPollHandle {}

impl crate::Loop {
    /// Create and initialize a fs poll handle
    pub fn fs_poll(&self) -> crate::Result<FsPollHandle> {
        FsPollHandle::new(self)
    }
}
