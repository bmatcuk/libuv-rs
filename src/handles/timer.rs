use crate::{FromInner, HandleTrait, Inner, IntoInner};
use std::convert::TryFrom;
use uv::{
    uv_timer_again, uv_timer_get_repeat, uv_timer_init, uv_timer_set_repeat, uv_timer_start,
    uv_timer_stop, uv_timer_t,
};

callbacks! {
    pub TimerCB(handel: TimerHandle);
}

/// Additional data stored on the handle
#[derive(Default)]
pub(crate) struct TimerDataFields<'a> {
    timer_cb: TimerCB<'a>,
}

/// Callback for uv_timer_start
extern "C" fn uv_timer_cb(handle: *mut uv_timer_t) {
    let dataptr = crate::Handle::get_data(uv_handle!(handle));
    if !dataptr.is_null() {
        unsafe {
            if let super::TimerData(d) = &mut (*dataptr).addl {
                d.timer_cb.call(handle.into_inner());
            }
        }
    }
}

/// Timer handles are used to schedule callbacks to be called in the future.
#[derive(Clone, Copy)]
pub struct TimerHandle {
    handle: *mut uv_timer_t,
}

impl TimerHandle {
    /// Create and initialize a new timer handle
    pub fn new(r#loop: &crate::Loop) -> crate::Result<TimerHandle> {
        let layout = std::alloc::Layout::new::<uv_timer_t>();
        let handle = unsafe { std::alloc::alloc(layout) as *mut uv_timer_t };
        if handle.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let ret = unsafe { uv_timer_init(r#loop.into_inner(), handle) };
        if ret < 0 {
            unsafe { std::alloc::dealloc(handle as _, layout) };
            return Err(crate::Error::from_inner(ret as uv::uv_errno_t));
        }

        crate::Handle::initialize_data(uv_handle!(handle), super::TimerData(Default::default()));

        Ok(TimerHandle { handle })
    }

    /// Start the timer. timeout and repeat are in milliseconds.
    ///
    /// If timeout is zero, the callback fires on the next event loop iteration. If repeat is
    /// non-zero, the callback fires first after timeout milliseconds and then repeatedly after
    /// repeat milliseconds.
    ///
    /// Note: Does not update the event loop’s concept of “now”. See Loop.update_time() for more
    /// information.
    ///
    /// If the timer is already active, it is simply updated.
    pub fn start<CB: Into<TimerCB<'static>>>(
        &mut self,
        timeout: u64,
        repeat: u64,
        cb: CB,
    ) -> crate::Result<()> {
        // uv_cb is either Some(uv_timer_cb) or None
        let cb = cb.into();
        let uv_cb = use_c_callback!(uv_timer_cb, cb);

        // cb is either Some(closure) or None - it is saved into data
        let dataptr = crate::Handle::get_data(uv_handle!(self.handle));
        if !dataptr.is_null() {
            if let super::TimerData(d) = unsafe { &mut (*dataptr).addl } {
                d.timer_cb = cb;
            }
        }

        crate::uvret(unsafe { uv_timer_start(self.handle, uv_cb, timeout, repeat) })
    }

    /// Stop the timer, the callback will not be called anymore.
    pub fn stop(&mut self) -> crate::Result<()> {
        crate::uvret(unsafe { uv_timer_stop(self.handle) })
    }

    /// Stop the timer, and if it is repeating restart it using the repeat value as the timeout. If
    /// the timer has never been started before it returns EINVAL.
    pub fn again(&mut self) -> crate::Result<()> {
        crate::uvret(unsafe { uv_timer_again(self.handle) })
    }

    /// Set the repeat interval value in milliseconds. The timer will be scheduled to run on the
    /// given interval, regardless of the callback execution duration, and will follow normal timer
    /// semantics in the case of a time-slice overrun.
    ///
    /// For example, if a 50ms repeating timer first runs for 17ms, it will be scheduled to run
    /// again 33ms later. If other tasks consume more than the 33ms following the first timer
    /// callback, then the callback will run as soon as possible.
    ///
    /// Note: If the repeat value is set from a timer callback it does not immediately take effect.
    /// If the timer was non-repeating before, it will have been stopped. If it was repeating, then
    /// the old repeat value will have been used to schedule the next timeout.
    pub fn set_repeat(&mut self, repeat: u64) {
        unsafe { uv_timer_set_repeat(self.handle, repeat) };
    }

    /// Get the timer repeat value.
    pub fn get_repeat(&self) -> u64 {
        unsafe { uv_timer_get_repeat(self.handle) }
    }
}

impl FromInner<*mut uv_timer_t> for TimerHandle {
    fn from_inner(handle: *mut uv_timer_t) -> TimerHandle {
        TimerHandle { handle }
    }
}

impl Inner<*mut uv::uv_handle_t> for TimerHandle {
    fn inner(&self) -> *mut uv::uv_handle_t {
        uv_handle!(self.handle)
    }
}

impl From<TimerHandle> for crate::Handle {
    fn from(timer: TimerHandle) -> crate::Handle {
        crate::Handle::from_inner(Inner::<*mut uv::uv_handle_t>::inner(&timer))
    }
}

impl crate::ToHandle for TimerHandle {
    fn to_handle(&self) -> crate::Handle {
        crate::Handle::from_inner(Inner::<*mut uv::uv_handle_t>::inner(self))
    }
}

impl TryFrom<crate::Handle> for TimerHandle {
    type Error = crate::ConversionError;

    fn try_from(handle: crate::Handle) -> Result<Self, Self::Error> {
        let t = handle.get_type();
        if t != crate::HandleType::TIMER {
            Err(crate::ConversionError::new(t, crate::HandleType::TIMER))
        } else {
            Ok((handle.inner() as *mut uv_timer_t).into_inner())
        }
    }
}

impl HandleTrait for TimerHandle {}

impl crate::Loop {
    /// Create and initialize a new timer handle
    pub fn timer(&self) -> crate::Result<TimerHandle> {
        TimerHandle::new(self)
    }
}
