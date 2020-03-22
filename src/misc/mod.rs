use crate::{FromInner, IntoInner};
use std::ffi::{CStr, CString};
use uv::{
    uv_cpu_info, uv_cpu_info_t, uv_free_cpu_info, uv_get_constrained_memory, uv_get_free_memory,
    uv_get_process_title, uv_get_total_memory, uv_getrusage, uv_gettimeofday, uv_hrtime,
    uv_loadavg, uv_print_active_handles, uv_print_all_handles, uv_random, uv_resident_set_memory,
    uv_set_process_title, uv_setup_args, uv_sleep, uv_uptime,
};

pub mod os;
pub use os::*;

/// Data type for CPU information.
pub struct CpuInfo {
    model: String,
    speed: i32,
    user_time: u64,
    nice_time: u64,
    sys_time: u64,
    idle_time: u64,
    irq_time: u64,
}

impl FromInner<&uv_cpu_info_t> for CpuInfo {
    fn from_inner(cpu: &uv_cpu_info_t) -> CpuInfo {
        let model = CStr::from_ptr(cpu.model).to_string_lossy().into_owned();
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

/// Gets information about the CPUs on the system.
pub fn cpu_info() -> crate::Result<Vec<CpuInfo>> {
    let mut infos: *mut uv_cpu_info_t = std::mem::zeroed();
    let mut count: std::os::raw::c_int = 0;
    crate::uvret(unsafe { uv_cpu_info(&mut infos as _, &mut count as _) })?;

    let result = unsafe { std::slice::from_raw_parts(infos, count as _) }
        .iter()
        .map(|info| info.into_inner())
        .collect();
    unsafe { uv_free_cpu_info(infos, count as _) };
    Ok(result)
}

/// Gets the amount of memory available to the process (in bytes) based on limits imposed by the
/// OS. If there is no such constraint, or the constraint is unknown, 0 is returned. Note that it
/// is not unusual for this value to be less than or greater than uv_get_total_memory().
///
/// Note: This function currently only returns a non-zero value on Linux, based on cgroups if it is
/// present.
pub fn get_constrained_memory() -> u64 {
    unsafe { uv_get_constrained_memory() }
}

/// Gets memory information (in bytes).
pub fn get_free_memory() -> u64 {
    unsafe { uv_get_free_memory() }
}

/// Store the program arguments. Required for getting / setting the process title. Libuv may take
/// ownership of the memory that argv points to. This function should be called exactly once, at
/// program start-up.
pub fn setup_args() -> Result<Vec<String>, std::ffi::NulError> {
    // Get arguments, transform into CStrings and then into raw bytes
    let args = std::env::args()
        .map(|s| CString::new(s).map(|s| s.into_bytes_with_nul()))
        .collect::<Result<Vec<_>, std::ffi::NulError>>()?;
    let argsptr: Vec<*mut std::os::raw::c_char> =
        args.iter().map(|s| s.as_mut_ptr() as _).collect();
    let argc = args.len();

    // rebuild args from the return value
    let args = unsafe { uv_setup_args(argc as _, argsptr.as_mut_ptr()) };
    let args = unsafe { std::slice::from_raw_parts(args, argc) };
    Ok(args
        .iter()
        .map(|arg| CStr::from_ptr(*arg).to_string_lossy().into_owned())
        .collect())
}

/// Gets the title of the current process. You must call setup_args before calling this function.
pub fn get_process_title() -> crate::Result<String> {
    let mut size = 16usize;
    let mut buf: Vec<std::os::raw::c_char> = vec![];
    loop {
        // title didn't fit in old size - double our allocation and try again
        size *= 2;
        buf.reserve(size - buf.len());

        let result = crate::uvret(unsafe { uv_get_process_title(buf.as_mut_ptr() as _, size) });
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
