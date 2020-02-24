use uv::{uv_async_init, uv_async_send, uv_async_t};

/// Additional data stored on the handle
#[derive(Default)]
pub(crate) struct AsyncDataFields {
    async_cb: Option<Box<dyn FnMut(AsyncHandle)>>,
}

/// Callback for uv_async_init
extern "C" fn async_cb(handle: *mut uv_async_t) {
    let dataptr = crate::Handle::get_data(uv_handle!(handle));
    if !dataptr.is_null() {
        unsafe {
            if let crate::AsyncData(d) = &mut (*dataptr).addl {
                if let Some(f) = d.async_cb.as_mut() {
                    f(handle.into());
                }
            }
        }
    }
}

/// Async handles allow the user to “wakeup” the event loop and get a callback called from another
/// thread.
pub struct AsyncHandle {
    handle: *mut uv_async_t,
    should_drop: bool,
}

impl AsyncHandle {
    /// Create and initialize a new async handle
    pub fn new(
        r#loop: &crate::Loop,
        cb: Option<impl FnMut(AsyncHandle) + 'static>,
    ) -> crate::Result<AsyncHandle> {
        let layout = std::alloc::Layout::new::<uv_async_t>();
        let handle = unsafe { std::alloc::alloc(layout) as *mut uv_async_t };
        if handle.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        // uv_cb is either Some(async_cb) or None
        let uv_cb = cb.as_ref().map(|_| async_cb as _);

        // async_cb is either Some(closure) or None - it is saved into data
        let async_cb = cb.map(|f| Box::new(f) as _);
        let data = AsyncDataFields { async_cb };
        crate::Handle::initialize_data(uv_handle!(handle), crate::AsyncData(data));

        let ret = unsafe { uv_async_init(r#loop.into(), handle, uv_cb) };
        if ret < 0 {
            crate::Handle::free_data(uv_handle!(handle));
            unsafe { std::alloc::dealloc(handle as _, layout) };
            return Err(crate::Error::from(ret as uv::uv_errno_t));
        }

        Ok(AsyncHandle {
            handle,
            should_drop: true,
        })
    }

    /// Wake up the event loop and call the async handle’s callback.
    ///
    /// Note: It’s safe to call this function from any thread. The callback will be called on the
    /// loop thread.
    ///
    /// Note: uv_async_send() is async-signal-safe. It’s safe to call this function from a signal
    /// handler.
    ///
    /// Warning: libuv will coalesce calls to send(), that is, not every call to it will yield an
    /// execution of the callback. For example: if send() is called 5 times in a row before the
    /// callback is called, the callback will only be called once. If send() is called again after
    /// the callback was called, it will be called again.
    pub fn send(&mut self) -> crate::Result<()> {
        crate::uvret(unsafe { uv_async_send(self.handle) })
    }
}

impl From<*mut uv_async_t> for AsyncHandle {
    fn from(handle: *mut uv_async_t) -> AsyncHandle {
        AsyncHandle {
            handle,
            should_drop: false,
        }
    }
}

impl From<AsyncHandle> for crate::Handle {
    fn from(r#async: AsyncHandle) -> crate::Handle {
        (r#async.handle as *mut uv::uv_handle_t).into()
    }
}

impl Into<*mut uv::uv_handle_t> for AsyncHandle {
    fn into(self) -> *mut uv::uv_handle_t {
        uv_handle!(self.handle)
    }
}

impl crate::HandleTrait for AsyncHandle {}

impl Drop for AsyncHandle {
    fn drop(&mut self) {
        if self.should_drop {
            if !self.handle.is_null() {
                crate::Handle::free_data(uv_handle!(self.handle));

                let layout = std::alloc::Layout::new::<uv_async_t>();
                unsafe { std::alloc::dealloc(self.handle as _, layout) };
            }
            self.handle = std::ptr::null_mut();
        }
    }
}

impl crate::Loop {
    /// Create and initialize a new async handle
    pub fn r#async(
        &self,
        cb: Option<impl FnMut(AsyncHandle) + 'static>,
    ) -> crate::Result<AsyncHandle> {
        AsyncHandle::new(self, cb)
    }
}
