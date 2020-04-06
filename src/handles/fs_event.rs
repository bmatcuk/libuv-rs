use crate::{FromInner, Inner, IntoInner};
use std::borrow::Cow;
use std::ffi::{CStr, CString};
use uv::{
    uv_fs_event_getpath, uv_fs_event_init, uv_fs_event_start, uv_fs_event_stop, uv_fs_event_t,
};

bitflags! {
    /// Flags for FsEventHandle::start()
    pub struct FsEventFlags: u32 {
        /// By default, if the fs event watcher is given a directory name, we will watch for all
        /// events in that directory. This flags overrides this behavior and makes fs_event report
        /// only changes to the directory entry itself. This flag does not affect individual files
        /// watched.
        ///
        /// This flag is currently not implemented yet on any backend.
        const WATCHENTRY = uv::uv_fs_event_flags_UV_FS_EVENT_WATCH_ENTRY;

        /// By default FsEventHandle will try to use a kernel interface such as inotify or kqueue
        /// to detect events. This may not work on remote file systems such as NFS mounts. This
        /// flag makes fs_event fall back to calling stat() on a regular interval.
        ///
        /// This flag is currently not implemented yet on any backend.
        const STAT = uv::uv_fs_event_flags_UV_FS_EVENT_STAT;

        /// By default, event watcher, when watching directory, is not registering (is ignoring)
        /// changes in its subdirectories.
        ///
        /// This flag will override this behaviour on platforms that support it.
        const RECURSIVE = uv::uv_fs_event_flags_UV_FS_EVENT_RECURSIVE;
    }
}

bitflags! {
    /// Event that caused the FsEventHandle callback to be called.
    pub struct FsEvent: u32 {
        /// File has been renamed
        const RENAME = uv::uv_fs_event_UV_RENAME;

        /// File has changed
        const CHANGE = uv::uv_fs_event_UV_CHANGE;
    }
}

/// Additional data stored on the handle
#[derive(Default)]
pub(crate) struct FsEventDataFields {
    fs_event_cb:
        Option<Box<dyn FnMut(FsEventHandle, Option<Cow<str>>, FsEvent, crate::Result<i32>)>>,
}

/// Callback for uv_fs_event_start
extern "C" fn uv_fs_event_cb(
    handle: *mut uv_fs_event_t,
    filename: *const std::os::raw::c_char,
    events: std::os::raw::c_int,
    status: std::os::raw::c_int,
) {
    let dataptr = crate::Handle::get_data(uv_handle!(handle));
    if !dataptr.is_null() {
        unsafe {
            if let super::FsEventData(d) = &mut (*dataptr).addl {
                if let Some(f) = d.fs_event_cb.as_mut() {
                    let filename = if filename.is_null() {
                        None
                    } else {
                        Some(CStr::from_ptr(filename).to_string_lossy())
                    };

                    let status = if status < 0 {
                        Err(crate::Error::from_inner(status as uv::uv_errno_t))
                    } else {
                        Ok(status)
                    };

                    f(handle.into_inner(), filename, FsEvent::from_bits_truncate(events as _), status);
                }
            }
        }
    }
}

/// FS Event handles allow the user to monitor a given path for changes, for example, if the file
/// was renamed or there was a generic change in it. This handle uses the best backend for the job
/// on each platform.
///
/// Note: For AIX, the non default IBM bos.ahafs package has to be installed. The AIX Event
/// Infrastructure file system (ahafs) has some limitations:
///
/// * ahafs tracks monitoring per process and is not thread safe. A separate process must be
///   spawned for each monitor for the same event.
/// * Events for file modification (writing to a file) are not received if only the containing
///   folder is watched.
///
/// See documentation for more details.
///
/// The z/OS file system events monitoring infrastructure does not notify of file creation/deletion
/// within a directory that is being monitored. See the IBM Knowledge centre for more details.
pub struct FsEventHandle {
    handle: *mut uv_fs_event_t,
}

impl FsEventHandle {
    /// Create and initialize a fs event handle
    pub fn new(r#loop: &crate::Loop) -> crate::Result<FsEventHandle> {
        let layout = std::alloc::Layout::new::<uv_fs_event_t>();
        let handle = unsafe { std::alloc::alloc(layout) as *mut uv_fs_event_t };
        if handle.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let ret = unsafe { uv_fs_event_init(r#loop.into_inner(), handle) };
        if ret < 0 {
            unsafe { std::alloc::dealloc(handle as _, layout) };
            return Err(crate::Error::from_inner(ret as uv::uv_errno_t));
        }

        crate::Handle::initialize_data(uv_handle!(handle), super::FsEventData(Default::default()));

        Ok(FsEventHandle { handle })
    }

    /// Start the handle with the given callback, which will watch the specified path for changes.
    ///
    /// Note: Currently the only supported flag is RECURSIVE and only on OSX and Windows.
    pub fn start(
        &mut self,
        path: &str,
        flags: FsEventFlags,
        cb: Option<
            impl FnMut(FsEventHandle, Option<Cow<str>>, FsEvent, crate::Result<i32>) + 'static,
        >,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = CString::new(path)?;

        // uv_cb is either Some(fs_event_cb) or None
        let uv_cb = cb.as_ref().map(|_| uv_fs_event_cb as _);

        // cb is either Some(closure) or None - it is saved into data
        let cb = cb.map(|f| Box::new(f) as _);
        let dataptr = crate::Handle::get_data(uv_handle!(self.handle));
        if !dataptr.is_null() {
            if let super::FsEventData(d) = unsafe { &mut (*dataptr).addl } {
                d.fs_event_cb = cb;
            }
        }

        crate::uvret(unsafe { uv_fs_event_start(self.handle, uv_cb, path.as_ptr(), flags.bits()) })
            .map_err(|e| Box::new(e) as _)
    }

    /// Stop the handle, the callback will no longer be called.
    pub fn stop(&mut self) -> crate::Result<()> {
        crate::uvret(unsafe { uv_fs_event_stop(self.handle) })
    }

    /// Get the path being monitored by the handle.
    pub fn getpath(&self) -> crate::Result<String> {
        // retrieve the size of the buffer we need to allocate
        let mut size = 0usize;
        let result = crate::uvret(unsafe {
            uv_fs_event_getpath(self.handle, std::ptr::null_mut(), &mut size as _)
        });
        if let Err(e) = result {
            if e != crate::Error::ENOBUFS {
                return Err(e);
            }
        }

        // On ENOBUFS, size is the length of the required buffer, *including* the null
        let mut buf: Vec<std::os::raw::c_uchar> = Vec::with_capacity(size);
        crate::uvret(unsafe {
            uv_fs_event_getpath(self.handle, buf.as_mut_ptr() as _, &mut size as _)
        })
        .map(|_| {
            // size is the length of the string, *not* including the null
            unsafe { buf.set_len(size + 1) };
            unsafe { CStr::from_bytes_with_nul_unchecked(&buf) }
                .to_string_lossy()
                .into_owned()
        })
    }
}

impl FromInner<*mut uv_fs_event_t> for FsEventHandle {
    fn from_inner(handle: *mut uv_fs_event_t) -> FsEventHandle {
        FsEventHandle { handle }
    }
}

impl Inner<*mut uv::uv_handle_t> for FsEventHandle {
    fn inner(&self) -> *mut uv::uv_handle_t {
        uv_handle!(self.handle)
    }
}

impl From<FsEventHandle> for crate::Handle {
    fn from(fs_event: FsEventHandle) -> crate::Handle {
        crate::Handle::from_inner(Inner::<*mut uv::uv_handle_t>::inner(&fs_event))
    }
}

impl crate::ToHandle for FsEventHandle {
    fn to_handle(&self) -> crate::Handle {
        crate::Handle::from_inner(Inner::<*mut uv::uv_handle_t>::inner(self))
    }
}

impl crate::HandleTrait for FsEventHandle {}

impl crate::Loop {
    /// Create and initialize a fs event handle
    pub fn fs_event(&self) -> crate::Result<FsEventHandle> {
        FsEventHandle::new(self)
    }
}
