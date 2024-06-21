use crate::{FromInner, HandleTrait, IntoInner};
use uv::{
    uv_backend_fd, uv_backend_timeout, uv_default_loop, uv_handle_t, uv_loop_alive, uv_loop_close,
    uv_loop_configure, uv_loop_delete, uv_loop_fork, uv_loop_get_data, uv_loop_init, uv_loop_new,
    uv_loop_option_UV_LOOP_BLOCK_SIGNAL, uv_loop_option_UV_METRICS_IDLE_TIME, uv_loop_set_data,
    uv_loop_t, uv_metrics_idle_time, uv_metrics_info, uv_metrics_t, uv_now, uv_run, uv_run_mode,
    uv_run_mode_UV_RUN_DEFAULT, uv_run_mode_UV_RUN_NOWAIT, uv_run_mode_UV_RUN_ONCE, uv_stop,
    uv_update_time, uv_walk,
};

/// Mode used to run the loop.
pub enum RunMode {
    /// Runs the event loop until there are no more active and referenced handles or requests.
    /// Returns non-zero if uv_stop() was called and there are still active handles or requests.
    /// Returns zero in all other cases.
    Default,

    /// Poll for i/o once. Note that this function blocks if there are no pending callbacks.
    /// Returns zero when done (no active handles or requests left), or non-zero if more callbacks
    /// are expected (meaning you should run the event loop again sometime in the future).
    Once,

    /// Poll for i/o once but don’t block if there are no pending callbacks. Returns zero if done
    /// (no active handles or requests left), or non-zero if more callbacks are expected (meaning
    /// you should run the event loop again sometime in the future).
    NoWait,
}

impl IntoInner<uv_run_mode> for RunMode {
    fn into_inner(self) -> uv_run_mode {
        match self {
            RunMode::Default => uv_run_mode_UV_RUN_DEFAULT,
            RunMode::Once => uv_run_mode_UV_RUN_ONCE,
            RunMode::NoWait => uv_run_mode_UV_RUN_NOWAIT,
        }
    }
}

/// The struct that contains event loop metrics. It is recommended to retrieve these metrics in a
/// PrepareCB in order to make sure there are no inconsistencies with the metrics counters.
pub struct Metrics {
    /// Number of event loop iterations.
    pub loop_count: u64,

    /// Number of events that have been processed by the event handler.
    pub events: u64,

    /// Number of events that were waiting to be processed when the event provider was called.
    pub events_waiting: u64,
}

impl FromInner<uv_metrics_t> for Metrics {
    fn from_inner(metrics: uv_metrics_t) -> Metrics {
        Metrics {
            loop_count: metrics.loop_count,
            events: metrics.events,
            events_waiting: metrics.events_waiting,
        }
    }
}

/// Data that we need to track with the loop.
#[derive(Default)]
pub(crate) struct LoopData {
    walk_cb: Option<Box<dyn FnMut(crate::Handle)>>,
}

/// Callback for uv_walk
extern "C" fn uv_walk_cb(handle: *mut uv_handle_t, _: *mut ::std::os::raw::c_void) {
    let handle: crate::Handle = handle.into_inner();
    let r#loop = handle.get_loop();
    let dataptr = r#loop.get_data();
    if !dataptr.is_null() {
        unsafe {
            if let Some(f) = (*dataptr).walk_cb.as_mut() {
                f(handle);
            }
        }
    }
}

/// The event loop is the central part of libuv’s functionality. It takes care of polling for i/o
/// and scheduling callbacks to be run based on different sources of events.
pub struct Loop {
    handle: *mut uv_loop_t,
    should_drop: bool,
}

impl Loop {
    /// Creates a new Loop.
    pub fn new() -> crate::Result<Loop> {
        let handle = unsafe { uv_loop_new() };
        if handle.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let ret = unsafe { uv_loop_init(handle) };
        if ret < 0 {
            unsafe { uv_loop_delete(handle) };
            return Err(crate::Error::from_inner(ret as uv::uv_errno_t));
        }

        let mut r#loop = Loop {
            handle,
            should_drop: true,
        };
        r#loop.initialize_data();

        Ok(r#loop)
    }

    // connect to an existing libuv instance
    pub unsafe fn from_external(handle: *mut uv_loop_t) -> Self {
        Self {
            handle,
            should_drop: false,
        }
    }

    /// Returns the initialized default loop.
    ///
    /// This function is just a convenient way for having a global loop throughout an application,
    /// the default loop is in no way different than the ones initialized with new(). As such, the
    /// default loop can (and should) be closed with close() so the resources associated with it
    /// are freed.
    ///
    /// Warning: This function is not thread safe.
    pub fn default() -> crate::Result<Loop> {
        let handle = unsafe { uv_default_loop() };
        if handle.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let mut r#loop = Loop {
            handle,
            should_drop: false,
        };
        if r#loop.get_data().is_null() {
            r#loop.initialize_data();
        }

        Ok(r#loop)
    }

    /// Initialize the loop's data.
    fn initialize_data(&mut self) {
        let data: Box<LoopData> = Box::new(Default::default());
        let ptr = Box::into_raw(data);
        unsafe { uv_loop_set_data(self.handle, ptr as _) };
    }

    /// Retrieve the loop's data.
    fn get_data(&self) -> *mut LoopData {
        unsafe { uv_loop_get_data(self.handle) as _ }
    }

    /// Free the loop's data.
    fn free_data(&mut self) {
        let ptr = self.get_data();
        std::mem::drop(unsafe { Box::from_raw(ptr) });
        unsafe { uv_loop_set_data(self.handle, std::ptr::null_mut()) };
    }

    /// Block a signal when polling for new events. The second argument to configure() is the
    /// signal number.
    ///
    /// This operation is currently only implemented for SIGPROF signals, to suppress unnecessary
    /// wakeups when using a sampling profiler. Requesting other signals will fail with UV_EINVAL.
    pub fn block_signal(&mut self, signum: i32) -> crate::Result<()> {
        crate::uvret(unsafe {
            uv_loop_configure(self.handle, uv_loop_option_UV_LOOP_BLOCK_SIGNAL, signum)
        })
    }

    /// Accumulate the amount of idle time the event loop spends in the event provider.
    ///
    /// This option is necessary to use metrics_idle_time().
    pub fn accumulate_idle_time(&mut self) -> crate::Result<()> {
        crate::uvret(unsafe { uv_loop_configure(self.handle, uv_loop_option_UV_METRICS_IDLE_TIME) })
    }

    /// Releases all internal loop resources. Call this function only when the loop has finished
    /// executing and all open handles and requests have been closed, or it will return
    /// Error::EBUSY.  After this function returns, the user can free the memory allocated for the
    /// loop.
    pub fn close(&mut self) -> crate::Result<()> {
        crate::uvret(unsafe { uv_loop_close(self.handle) })
    }

    /// This function runs the event loop. It will act differently depending on the specified mode.
    /// run() is not reentrant. It must not be called from a callback.
    pub fn run(&mut self, mode: RunMode) -> crate::Result<i32> {
        let ret = unsafe { uv_run(self.handle, mode.into_inner()) };
        if ret < 0 {
            Err(crate::Error::from_inner(ret as uv::uv_errno_t))
        } else {
            Ok(ret)
        }
    }

    /// Returns true if there are referenced active handles, active requests or closing handles in
    /// the loop.
    pub fn is_alive(&self) -> bool {
        unsafe { uv_loop_alive(self.handle) != 0 }
    }

    /// Retrieve the amount of time the event loop has been idle in the kernel’s event provider
    /// (e.g. epoll_wait). The call is thread safe.
    ///
    /// The return value is the accumulated time spent idle in the kernel’s event provider starting
    /// from when the loop was configured to collect the idle time.
    ///
    /// Note The event loop will not begin accumulating the event provider’s idle time until
    /// calling accumulate_idle_time().
    pub fn idle_time(&self) -> u64 {
        unsafe { uv_metrics_idle_time(self.handle) }
    }

    /// The current set of event loop metrics.
    pub fn metrics_info(&self) -> crate::Result<Metrics> {
        let mut metrics: uv_metrics_t = unsafe { std::mem::zeroed() };
        crate::uvret(unsafe { uv_metrics_info(self.handle, &mut metrics as _) })
            .map(|_| metrics.into_inner())
    }

    /// Stop the event loop, causing run() to end as soon as possible. This will happen not sooner
    /// than the next loop iteration. If this function was called before blocking for i/o, the loop
    /// won’t block for i/o on this iteration.
    pub fn stop(&mut self) {
        unsafe { uv_stop(self.handle) };
    }

    /// Get backend file descriptor. Only kqueue, epoll and event ports are supported.
    ///
    /// This can be used in conjunction with run(NoWait) to poll in one thread and run the event
    /// loop’s callbacks in another see test/test-embed.c for an example.
    ///
    /// Note: Embedding a kqueue fd in another kqueue pollset doesn’t work on all platforms. It’s
    /// not an error to add the fd but it never generates events.
    pub fn backend_fd(&self) -> i32 {
        unsafe { uv_backend_fd(self.handle) as _ }
    }

    /// Get the poll timeout. The return value is in milliseconds, or -1 for no timeout.
    pub fn backend_timeout(&self) -> i32 {
        unsafe { uv_backend_timeout(self.handle) as _ }
    }

    /// Return the current timestamp in milliseconds. The timestamp is cached at the start of the
    /// event loop tick, see update_time() for details and rationale.
    ///
    /// The timestamp increases monotonically from some arbitrary point in time. Don’t make
    /// assumptions about the starting point, you will only get disappointed.
    pub fn now(&self) -> u64 {
        unsafe { uv_now(self.handle) }
    }

    /// Update the event loop’s concept of “now”. Libuv caches the current time at the start of the
    /// event loop tick in order to reduce the number of time-related system calls.
    ///
    /// You won’t normally need to call this function unless you have callbacks that block the
    /// event loop for longer periods of time, where “longer” is somewhat subjective but probably
    /// on the order of a millisecond or more.
    pub fn update_time(&mut self) {
        unsafe { uv_update_time(self.handle) }
    }

    /// Walk the list of handles.
    pub fn walk(&self, cb: impl FnMut(crate::Handle) + 'static) {
        let cb = Box::new(cb);
        let dataptr = self.get_data();
        if !dataptr.is_null() {
            unsafe { (*dataptr).walk_cb = Some(cb) };
        }

        unsafe { uv_walk(self.handle, Some(uv_walk_cb), std::ptr::null_mut()) };
    }

    /// Reinitialize any kernel state necessary in the child process after a fork(2) system call.
    ///
    /// Previously started watchers will continue to be started in the child process.
    ///
    /// It is necessary to explicitly call this function on every event loop created in the parent
    /// process that you plan to continue to use in the child, including the default loop (even if
    /// you don’t continue to use it in the parent). This function must be called before calling
    /// run() or any other API function using the loop in the child. Failure to do so will result
    /// in undefined behaviour, possibly including duplicate events delivered to both parent and
    /// child or aborting the child process.
    ///
    /// When possible, it is preferred to create a new loop in the child process instead of reusing
    /// a loop created in the parent. New loops created in the child process after the fork should
    /// not use this function.
    ///
    /// This function is not implemented on Windows, where it returns UV_ENOSYS.
    ///
    /// Caution: This function is experimental. It may contain bugs, and is subject to change or
    /// removal. API and ABI stability is not guaranteed.
    ///
    /// Note: On Mac OS X, if directory FS event handles were in use in the parent process for any
    /// event loop, the child process will no longer be able to use the most efficient FSEvent
    /// implementation. Instead, uses of directory FS event handles in the child will fall back to
    /// the same implementation used for files and on other kqueue-based systems.
    ///
    /// Caution: On AIX and SunOS, FS event handles that were already started in the parent process
    /// at the time of forking will not deliver events in the child process; they must be closed
    /// and restarted. On all other platforms, they will continue to work normally without any
    /// further intervention.
    pub fn fork(&mut self) -> crate::Result<()> {
        crate::uvret(unsafe { uv_loop_fork(self.handle) })
    }
}

impl Clone for Loop {
    fn clone(&self) -> Self {
        Loop {
            handle: self.handle,
            should_drop: false,
        }
    }
}

impl FromInner<*mut uv_loop_t> for Loop {
    fn from_inner(handle: *mut uv_loop_t) -> Loop {
        Loop {
            handle,
            should_drop: false,
        }
    }
}

impl IntoInner<*mut uv_loop_t> for &Loop {
    fn into_inner(self) -> *mut uv_loop_t {
        self.handle
    }
}

impl Drop for Loop {
    fn drop(&mut self) {
        if self.should_drop {
            if !self.handle.is_null() {
                self.free_data();
                unsafe { uv_loop_delete(self.handle) };
            }
            self.handle = std::ptr::null_mut();
        }
    }
}
