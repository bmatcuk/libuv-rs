use uv::{uv_timer_t, uv_timer_init, uv_timer_start, uv_timer_stop, uv_timer_again, uv_timer_set_repeat, uv_timer_get_repeat};

/// Timer handles are used to schedule callbacks to be called in the future.
pub struct TimerHandle {
    handle: *mut uv_timer_t,
    should_drop: bool,
}

impl TimerHandle {
    /// Create and initialize a new timer handle
    pub fn new(r#loop: &crate::Loop) -> crate::Result<TimerHandle> {
        let layout = std::alloc::Layout::new::<uv_timer_t>();
        let handle = unsafe { std::alloc::alloc(layout) as *mut uv_timer_t };
        if handle.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let ret = unsafe { uv_timer_init(r#loop.into(), handle) };
        if ret < 0 {
            return Err(crate::Error::from(ret as uv::uv_errno_t));
        }

        Ok(TimerHandle {
            handle,
            should_drop: true,
        })
    }
}

impl From<*mut uv_timer_t> for TimerHandle {
    fn from(handle: *mut uv_timer_t) -> TimerHandle {
        TimerHandle { handle, should_drop: false }
    }
}

impl Drop for TimerHandle {
    fn drop(&mut self) {
        if self.should_drop {
            if !self.handle.is_null() {
                let layout = std::alloc::Layout::new::<uv_timer_t>();
                unsafe { std::alloc::dealloc(self.handle as _, layout) };
            }
            self.handle = std::ptr::null_mut();
        }
    }
}

impl crate::Loop {
    /// Create and initialize a new timer handle
    pub fn timer(&self) -> crate::Result<TimerHandle> {
        TimerHandle::new(self)
    }
}
