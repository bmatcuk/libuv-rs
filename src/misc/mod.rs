use crate::{FromInner, IntoInner};
use std::ffi::{CStr, CString};
use uv::{
    uv_clock_gettime, uv_cpu_info, uv_cpu_info_t, uv_cpumask_size, uv_free_cpu_info,
    uv_get_available_memory, uv_get_constrained_memory, uv_get_free_memory, uv_get_process_title,
    uv_get_total_memory, uv_getrusage, uv_gettimeofday, uv_hrtime, uv_library_shutdown, uv_loadavg,
    uv_resident_set_memory, uv_rusage_t, uv_set_process_title, uv_setup_args, uv_sleep,
    uv_timespec64_t, uv_timeval64_t, uv_timeval_t, uv_uptime,
};

pub mod os;
pub use os::*;

/// Data type for storing times.
pub struct TimeVal {
    pub sec: i64,
    pub usec: i64,
}

impl FromInner<uv_timeval_t> for TimeVal {
    fn from_inner(tv: uv_timeval_t) -> TimeVal {
        TimeVal {
            sec: tv.tv_sec as _,
            usec: tv.tv_usec as _,
        }
    }
}

impl FromInner<uv_timeval64_t> for TimeVal {
    fn from_inner(tv: uv_timeval64_t) -> TimeVal {
        TimeVal {
            sec: tv.tv_sec,
            usec: tv.tv_usec as _,
        }
    }
}

/// Data type for resource usage results.
pub struct ResourceUsage {
    /// user CPU time used
    pub usertime: TimeVal,

    /// system CPU time used
    pub systime: TimeVal,

    /// maximum resident set size
    pub maxrss: u64,

    /// integral shared memory size (no Windows support)
    pub ixrss: u64,

    /// integral unshared data size (no Windows support)
    pub idrss: u64,

    /// integral unshared stack size (no Windows support)
    pub isrss: u64,

    /// page reclaims (soft page faults) (no Windows support)
    pub minflt: u64,

    /// page faults (hard page faults)
    pub majflt: u64,

    /// swaps (no Windows support)
    pub nswap: u64,

    /// block input operations
    pub inblock: u64,

    /// block output operations
    pub oublock: u64,

    /// IPC messages sent (no windows support)
    pub msgsnd: u64,

    /// IPC messages received (no Windows support)
    pub msgrcv: u64,

    /// signals received (no Windows support)
    pub nsignals: u64,

    /// voluntary context switches (no Windows support)
    pub nvcsw: u64,

    /// involuntary context switches (no Windows support)
    pub nivcsw: u64,
}

impl FromInner<uv_rusage_t> for ResourceUsage {
    fn from_inner(usage: uv_rusage_t) -> ResourceUsage {
        ResourceUsage {
            usertime: usage.ru_utime.into_inner(),
            systime: usage.ru_stime.into_inner(),
            maxrss: usage.ru_maxrss,
            ixrss: usage.ru_ixrss,
            idrss: usage.ru_idrss,
            isrss: usage.ru_isrss,
            minflt: usage.ru_minflt,
            majflt: usage.ru_majflt,
            nswap: usage.ru_nswap,
            inblock: usage.ru_inblock,
            oublock: usage.ru_oublock,
            msgsnd: usage.ru_msgsnd,
            msgrcv: usage.ru_msgrcv,
            nsignals: usage.ru_nsignals,
            nvcsw: usage.ru_nvcsw,
            nivcsw: usage.ru_nivcsw,
        }
    }
}

/// Data type for CPU information.
pub struct CpuInfo {
    pub model: String,
    pub speed: i32,
    pub user_time: u64,
    pub nice_time: u64,
    pub sys_time: u64,
    pub idle_time: u64,
    pub irq_time: u64,
}

impl FromInner<&uv_cpu_info_t> for CpuInfo {
    fn from_inner(cpu: &uv_cpu_info_t) -> CpuInfo {
        let model = unsafe { CStr::from_ptr(cpu.model) }
            .to_string_lossy()
            .into_owned();
        CpuInfo {
            model,
            speed: cpu.speed,
            user_time: cpu.cpu_times.user,
            nice_time: cpu.cpu_times.nice,
            sys_time: cpu.cpu_times.sys,
            idle_time: cpu.cpu_times.idle,
            irq_time: cpu.cpu_times.irq,
        }
    }
}

/// Clock source for clock_gettime().
#[repr(u32)]
pub enum ClockId {
    Monotonic = uv::uv_clock_id_UV_CLOCK_MONOTONIC as _,
    Realtime = uv::uv_clock_id_UV_CLOCK_REALTIME as _,
}

/// Y2K38-safe data type for storing times with nanosecond resolution.
pub struct TimeSpec64 {
    pub sec: i64,
    pub nsec: i32,
}

impl FromInner<uv_timespec64_t> for TimeSpec64 {
    fn from_inner(timespec: uv_timespec64_t) -> TimeSpec64 {
        TimeSpec64 {
            sec: timespec.tv_sec,
            nsec: timespec.tv_nsec,
        }
    }
}

/// Store the program arguments. Required for getting / setting the process title or the executable
/// path. Libuv may take ownership of the memory that argv points to. This function should be
/// called exactly once, at program start-up.
pub fn setup_args() -> Result<Vec<String>, std::ffi::NulError> {
    // Get arguments, transform into CStrings and then into raw bytes
    let mut args = std::env::args()
        .map(|s| CString::new(s).map(|s| s.into_bytes_with_nul()))
        .collect::<Result<Vec<_>, std::ffi::NulError>>()?;
    let mut argsptr: Vec<*mut std::os::raw::c_char> =
        args.iter_mut().map(|s| s.as_mut_ptr() as _).collect();
    let argc = args.len();

    // rebuild args from the return value
    let args = unsafe { uv_setup_args(argc as _, argsptr.as_mut_ptr()) };
    let args = unsafe { std::slice::from_raw_parts(args, argc) };
    Ok(args
        .iter()
        .map(|arg| {
            unsafe { CStr::from_ptr(*arg) }
                .to_string_lossy()
                .into_owned()
        })
        .collect())
}

/// Release any global state that libuv is holding onto. Libuv will normally do so automatically
/// when it is unloaded but it can be instructed to perform cleanup manually.
///
/// Warning: Only call shutdown() once.
///
/// Warning: Don’t call shutdown() when there are still event loops or I/O requests active.
///
/// Warning: Don’t call libuv functions after calling shutdown().
pub fn shutdown() {
    unsafe { uv_library_shutdown() };
}

/// Gets the title of the current process. You must call setup_args before calling this function on
/// Unix and AIX systems. If setup_args has not been called on systems that require it, then
/// ENOBUFS is returned.
///
/// Note On BSD systems, setup_args is needed for getting the initial process title. The process
/// title returned will be an empty string until either setup_args or set_process_title is called.
///
/// This function is thread-safe on all supported platforms.
///
/// Returns an error if setup_args is needed but hasn’t been called.
pub fn get_process_title() -> crate::Result<String> {
    let mut size = 16usize;
    let mut buf: Vec<std::os::raw::c_char> = vec![];
    loop {
        // title didn't fit in old size - double our allocation and try again
        size *= 2;
        buf.reserve(size - buf.len());

        let result =
            crate::uvret(unsafe { uv_get_process_title(buf.as_mut_ptr() as _, size as _) });
        if let Err(e) = result {
            if e != crate::Error::ENOBUFS {
                return Err(e);
            }
        } else {
            break;
        }
    }

    Ok(unsafe { CStr::from_ptr(buf.as_ptr()) }
        .to_string_lossy()
        .into_owned())
}

/// Sets the current process title. You must call setup_args before calling this function on Unix
/// and AIX systems. If setup_args has not been called on systems that require it, then ENOBUFS is
/// returned. On platforms with a fixed size buffer for the process title the contents of title
/// will be copied to the buffer and truncated if larger than the available space. Other platforms
/// will return ENOMEM if they cannot allocate enough space to duplicate the contents of title.
///
/// This function is thread-safe on all supported platforms.
///
/// Returns an error if setup_args is needed but hasn’t been called.
pub fn set_process_title(title: &str) -> Result<(), Box<dyn std::error::Error>> {
    let title = CString::new(title)?;
    crate::uvret(unsafe { uv_set_process_title(title.as_ptr()) }).map_err(|e| Box::new(e) as _)
}

/// Gets the resident set size (RSS) for the current process.
pub fn resident_set_memory() -> crate::Result<usize> {
    let mut rss = 0usize;
    crate::uvret(unsafe { uv_resident_set_memory(&mut rss as _) }).map(|_| rss as _)
}

/// Gets the current system uptime. Depending on the system full or fractional seconds are
/// returned.
pub fn uptime() -> crate::Result<f64> {
    let mut uptime = 0f64;
    crate::uvret(unsafe { uv_uptime(&mut uptime as _) }).map(|_| uptime)
}

/// Gets the resource usage measures for the current process.
///
/// Note: On Windows not all fields are set, the unsupported fields are filled with zeroes. See
/// ResourceUsage for more details.
pub fn getrusage() -> crate::Result<ResourceUsage> {
    let mut usage: uv_rusage_t = unsafe { std::mem::zeroed() };
    crate::uvret(unsafe { uv_getrusage(&mut usage as _) }).map(|_| usage.into_inner())
}

/// Gets information about the CPUs on the system.
pub fn cpu_info() -> crate::Result<Vec<CpuInfo>> {
    let mut infos: *mut uv_cpu_info_t = unsafe { std::mem::zeroed() };
    let mut count: std::os::raw::c_int = 0;
    crate::uvret(unsafe { uv_cpu_info(&mut infos as _, &mut count as _) })?;

    let result = unsafe { std::slice::from_raw_parts(infos, count as _) }
        .iter()
        .map(|info| info.into_inner())
        .collect();
    unsafe { uv_free_cpu_info(infos, count as _) };
    Ok(result)
}

/// Returns the maximum size of the mask used for process/thread affinities, or ENOTSUP if
/// affinities are not supported on the current platform.
pub fn cpumask_size() -> i32 {
    unsafe { uv_cpumask_size() }
}

/// Gets the load average. See: https://en.wikipedia.org/wiki/Load_(computing)
///
/// Note: Returns [0,0,0] on Windows (i.e., it’s not implemented).
pub fn loadavg() -> [f64; 3] {
    let mut avg = [0f64; 3];
    unsafe { uv_loadavg(avg.as_mut_ptr()) };
    return avg;
}

/// Gets the amount of free memory available in the system, as reported by the kernel (in bytes).
/// Returns 0 when unknown.
pub fn get_free_memory() -> u64 {
    unsafe { uv_get_free_memory() }
}

/// Gets the total amount of physical memory in the system (in bytes). Returns 0 when unknown.
pub fn get_total_memory() -> u64 {
    unsafe { uv_get_total_memory() }
}

/// Gets the total amount of memory available to the process (in bytes) based on limits imposed by
/// the OS. If there is no such constraint, or the constraint is unknown, 0 is returned. If there
/// is a constraining mechanism, but there is no constraint set, `UINT64_MAX` is returned. Note
/// that it is not unusual for this value to be less than or greater than get_total_memory().
///
/// Note: This function currently only returns a non-zero value on Linux, based on cgroups if it is
/// present, and on z/OS based on RLIMIT_MEMLIMIT.
pub fn get_constrained_memory() -> u64 {
    unsafe { uv_get_constrained_memory() }
}

/// Gets the amount of free memory that is still available to the process (in bytes). This differs
/// from get_free_memory() in that it takes into account any limits imposed by the OS. If there is
/// no such constraint, or the constraint is unknown, the amount returned will be identical to
/// get_free_memory().
///
/// Note: This function currently only returns a value that is different from what
/// get_free_memory() reports on Linux, based on cgroups if it is present.
pub fn get_available_memory() -> u64 {
    unsafe { uv_get_available_memory() }
}

/// Returns the current high-resolution timestamp. This is expressed in nanoseconds. It is relative
/// to an arbitrary time in the past. It is not related to the time of day and therefore not
/// subject to clock drift. The primary use is for measuring performance between intervals.
///
/// Note: Not every platform can support nanosecond resolution; however, this value will always be
/// in nanoseconds.
pub fn hrtime() -> u64 {
    unsafe { uv_hrtime() }
}

/// Obtain the current system time from a high-resolution real-time or monotonic clock source.
///
/// The real-time clock counts from the UNIX epoch (1970-01-01) and is subject to time adjustments;
/// it can jump back in time.
///
/// The monotonic clock counts from an arbitrary point in the past and never jumps back in time.
pub fn clock_gettime(clock_id: ClockId) -> crate::Result<TimeSpec64> {
    let mut timespec: uv_timespec64_t = unsafe { std::mem::zeroed() };
    crate::uvret(unsafe { uv_clock_gettime(clock_id as _, &mut timespec as _) })
        .map(|_| timespec.into_inner())
}

/// Cross-platform implementation of gettimeofday(2). The timezone argument to gettimeofday() is
/// not supported, as it is considered obsolete.
pub fn gettimeofday() -> crate::Result<TimeVal> {
    let mut tv: uv_timeval64_t = unsafe { std::mem::zeroed() };
    crate::uvret(unsafe { uv_gettimeofday(&mut tv as _) }).map(|_| tv.into_inner())
}

/// Causes the calling thread to sleep for msec milliseconds.
pub fn sleep(msec: u32) {
    unsafe { uv_sleep(msec) };
}
