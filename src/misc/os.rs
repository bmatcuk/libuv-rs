use crate::{FromInner, IntoInner};
use std::ffi::CStr;
use uv::{
    uv_os_free_passwd, uv_os_get_passwd, uv_os_gethostname, uv_os_getpid, uv_os_getppid,
    uv_os_getpriority, uv_os_setpriority, uv_os_uname, uv_passwd_t, uv_utsname_t,
    UV_MAXHOSTNAMESIZE,
};

/// Cross platform representation of a pid_t.
pub type Pid = i32;

/// Data type for user information.
pub struct User {
    pub username: String,
    pub uid: Option<crate::Uid>,
    pub gid: Option<crate::Gid>,
    pub shell: Option<String>,
    pub homedir: String,
}

impl FromInner<uv_passwd_t> for User {
    fn from_inner(passwd: uv_passwd_t) -> User {
        let username = unsafe { CStr::from_ptr(passwd.username) }
            .to_string_lossy()
            .into_owned();
        let homedir = unsafe { CStr::from_ptr(passwd.homedir) }
            .to_string_lossy()
            .into_owned();
        let shell = if passwd.shell.is_null() {
            None
        } else {
            Some(
                unsafe { CStr::from_ptr(passwd.shell) }
                    .to_string_lossy()
                    .into_owned(),
            )
        };
        let uid = if passwd.uid >= 0 {
            Some(passwd.uid as _)
        } else {
            None
        };
        let gid = if passwd.gid >= 0 {
            Some(passwd.gid as _)
        } else {
            None
        };
        User {
            username,
            uid,
            gid,
            shell,
            homedir,
        }
    }
}

/// Data type for operating system name and version information.
pub struct SystemInfo {
    pub sysname: String,
    pub release: String,
    pub version: String,
    pub machine: String,
}

impl FromInner<uv_utsname_t> for SystemInfo {
    fn from_inner(uname: uv_utsname_t) -> SystemInfo {
        let sysname = unsafe { CStr::from_ptr(&uname.sysname as _) }
            .to_string_lossy()
            .into_owned();
        let release = unsafe { CStr::from_ptr(&uname.release as _) }
            .to_string_lossy()
            .into_owned();
        let version = unsafe { CStr::from_ptr(&uname.version as _) }
            .to_string_lossy()
            .into_owned();
        let machine = unsafe { CStr::from_ptr(&uname.machine as _) }
            .to_string_lossy()
            .into_owned();
        SystemInfo {
            sysname,
            release,
            version,
            machine,
        }
    }
}

/// Gets a subset of the password file entry for the current effective uid (not the real uid). The
/// populated data includes the username, euid, gid, shell, and home directory. On non-Windows
/// systems, all data comes from getpwuid_r(3). On Windows, uid, gid, and shell are all set to
/// None.
pub fn get_passwd() -> crate::Result<User> {
    let mut passwd: uv_passwd_t = unsafe { std::mem::zeroed() };
    crate::uvret(unsafe { uv_os_get_passwd(&mut passwd as _) })?;

    let result = passwd.into_inner();
    unsafe { uv_os_free_passwd(&mut passwd as _) };
    Ok(result)
}

/// Returns the hostname
pub fn gethostname() -> crate::Result<String> {
    let mut size = UV_MAXHOSTNAMESIZE as u64;
    let mut buf: Vec<std::os::raw::c_uchar> = Vec::with_capacity(size as _);
    crate::uvret(unsafe { uv_os_gethostname(buf.as_mut_ptr() as _, &mut size as _) }).map(|_| {
        // size is the length of the string, *not* including the null
        unsafe { buf.set_len((size as usize) + 1) };
        unsafe { CStr::from_bytes_with_nul_unchecked(&buf) }
            .to_string_lossy()
            .into_owned()
    })
}

/// Returns the current process ID.
pub fn getpid() -> Pid {
    unsafe { uv_os_getpid() as _ }
}

/// Returns the parent process ID.
pub fn getppid() -> Pid {
    unsafe { uv_os_getppid() as _ }
}

/// Retrieves the scheduling priority of the process specified by pid. The returned value of
/// priority is between -20 (high priority) and 19 (low priority).
///
/// Note: On Windows, the returned priority will equal one of the libuv_sys2::UV_PRIORITY
/// constants.
pub fn getpriority(pid: Pid) -> crate::Result<i32> {
    let mut priority = 0i32;
    crate::uvret(unsafe { uv_os_getpriority(pid as _, &mut priority as _) }).map(|_| priority)
}

/// Sets the scheduling priority of the process specified by pid. The priority value range is
/// between -20 (high priority) and 19 (low priority). The constants UV_PRIORITY_LOW,
/// UV_PRIORITY_BELOW_NORMAL, UV_PRIORITY_NORMAL, UV_PRIORITY_ABOVE_NORMAL, UV_PRIORITY_HIGH, and
/// UV_PRIORITY_HIGHEST are also provided for convenience in libuv_sys2.
///
/// Note: On Windows, this function utilizes SetPriorityClass(). The priority argument is mapped to
/// a Windows priority class. When retrieving the process priority, the result will equal one of
/// the UV_PRIORITY constants, and not necessarily the exact value of priority.
///
/// Note: On Windows, setting PRIORITY_HIGHEST will only work for elevated user, for others it will
/// be silently reduced to PRIORITY_HIGH.
pub fn setpriority(pid: Pid, priority: i32) -> crate::Result<()> {
    crate::uvret(unsafe { uv_os_setpriority(pid as _, priority as _) })
}

/// Retrieves system information in buffer. The populated data includes the operating system name,
/// release, version, and machine. On non-Windows systems, uv_os_uname() is a thin wrapper around
/// uname(2).
pub fn uname() -> crate::Result<SystemInfo> {
    let mut buf: uv_utsname_t = unsafe { std::mem::zeroed() };
    crate::uvret(unsafe { uv_os_uname(&mut buf as _) })?;
    Ok(buf.into_inner())
}
