use crate::{FromInner, IntoInner};
use uv::{uv_idle_init, uv_idle_start, uv_idle_stop, uv_idle_t};

/// Additional data stored on the handle
#[derive(Default)]
pub(crate) struct IdleDataFields {
    idle_cb: Option<Box<dyn FnMut(IdleHandle)>>,
}

/// Callback for uv_idle_start
extern "C" fn uv_idle_cb(handle: *mut uv_idle_t) {
    let dataptr = crate::Handle::get_data(uv_handle!(handle));
    if !dataptr.is_null() {
        unsafe {
            if let crate::IdleData(d) = &mut (*dataptr).addl {
                if let Some(f) = d.idle_cb.as_mut() {
                    f(handle.into_inner())
                }
            }
        }
    }
}

/// Idle handles will run the given callback once per loop iteration, right before the uv_prepare_t
/// handles.
///
/// Note: The notable difference with prepare handles is that when there are active idle handles,
/// the loop will perform a zero timeout poll instead of blocking for i/o.
///
/// Warning: Despite the name, idle handles will get their callbacks called on every loop
/// iteration, not when the loop is actually “idle”.
pub struct IdleHandle {
    handle: *mut uv_idle_t,
}

impl IdleHandle {
    /// Create and initialize a new idle handle
    pub fn new(r#loop: &crate::Loop) -> crate::Result<IdleHandle> {
        let layout = std::alloc::Layout::new::<uv_idle_t>();
        let handle = unsafe { std::alloc::alloc(layout) as *mut uv_idle_t };
        if handle.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let ret = unsafe { uv_idle_init(r#loop.into_inner(), handle) };
        if ret < 0 {
            unsafe { std::alloc::dealloc(handle as _, layout) };
            return Err(crate::Error::from_inner(ret as uv::uv_errno_t));
        }

        crate::Handle::initialize_data(uv_handle!(handle), crate::IdleData(Default::default()));

        Ok(IdleHandle { handle })
    }

    /// Start the handle with the given callback.
    pub fn start(&mut self, cb: Option<impl FnMut(IdleHandle) + 'static>) -> crate::Result<()> {
        // uv_cb is either Some(uv_idle_cb) or None
        let uv_cb = cb.as_ref().map(|_| uv_idle_cb as _);

        // cb is either Some(closure) or None - it is saved into data
        let cb = cb.map(|f| Box::new(f) as _);
        let dataptr = crate::Handle::get_data(uv_handle!(self.handle));
        if !dataptr.is_null() {
            if let crate::IdleData(d) = unsafe { &mut (*dataptr).addl } {
                d.idle_cb = cb;
            }
        }

        crate::uvret(unsafe { uv_idle_start(self.handle, uv_cb) })
    }

    /// Stop the handle, the callback will no longer be called.
    pub fn stop(&mut self) -> crate::Result<()> {
        crate::uvret(unsafe { uv_idle_stop(self.handle) })
    }
}

impl FromInner<*mut uv_idle_t> for IdleHandle {
    fn from_inner(handle: *mut uv_idle_t) -> IdleHandle {
        IdleHandle { handle }
    }
}

impl IntoInner<*mut uv::uv_handle_t> for IdleHandle {
    fn into_inner(self) -> *mut uv::uv_handle_t {
        uv_handle!(self.handle)
    }
}

impl From<IdleHandle> for crate::Handle {
    fn from(idle: IdleHandle) -> crate::Handle {
        idle.into_inner().into_inner()
    }
}

impl crate::HandleTrait for IdleHandle {}

impl crate::Loop {
    /// Create and initialize a new idle handle
    pub fn idle(&self) -> crate::Result<IdleHandle> {
        IdleHandle::new(self)
    }
}
