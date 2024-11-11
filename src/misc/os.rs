use crate::{FromInner, IntoInner};
use std::ffi::CStr;
use uv::{
    uv_available_parallelism, uv_group_t, uv_os_free_group, uv_os_free_passwd, uv_os_get_group,
    uv_os_get_passwd, uv_os_get_passwd2, uv_os_gethostname, uv_os_getpid, uv_os_getppid,
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

/// Data type for group file information.
pub struct Group {
    pub groupname: String,
    pub gid: Option<crate::Gid>,
    pub members: Vec<String>,
}

impl FromInner<uv_group_t> for Group {
    fn from_inner(group: uv_group_t) -> Group {
        let groupname = unsafe { CStr::from_ptr(group.groupname) }
            .to_string_lossy()
            .into_owned();
        let gid = if group.gid >= 0 {
            Some(group.gid as _)
        } else {
            None
        };
        let mut members = Vec::new();
        let mut members_ptr = group.members;
        unsafe {
            while let Some(member_ptr) = members_ptr.as_ref() {
                let member = CStr::from_ptr(*member_ptr).to_string_lossy().into_owned();
                members.push(member);
                members_ptr = members_ptr.offset(1);
            }
        }
        Group {
            groupname,
            gid,
            members,
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

/// Gets a subset of the password file entry for the provided uid. The populated data includes the
/// username, euid, gid, shell, and home directory. On non-Windows systems, all data comes from
/// getpwuid_r(3). On Windows, uid, gid, and shell are set to None and have no meaning.
pub fn get_passwd2(uid: crate::Uid) -> crate::Result<User> {
    let mut passwd: uv_passwd_t = unsafe { std::mem::zeroed() };
    crate::uvret(unsafe { uv_os_get_passwd2(&mut passwd as _, uid) })?;

    let result = passwd.into_inner();
    unsafe { uv_os_free_passwd(&mut passwd as _) };
    Ok(result)
}

/// Gets a subset of the group file entry for the provided uid. The populated data includes the
/// group name, gid, and members. On non-Windows systems, all data comes from getgrgid_r(3). On
/// Windows, uid and gid are set to None and have no meaning.
pub fn get_group(gid: crate::Gid) -> crate::Result<Group> {
    let mut group: uv_group_t = unsafe { std::mem::zeroed() };
    crate::uvret(unsafe { uv_os_get_group(&mut group as _, gid) })?;

    let result = group.into_inner();
    unsafe { uv_os_free_group(&mut group as _) };
    Ok(result)
}

/// Returns the hostname
pub fn gethostname() -> crate::Result<String> {
    let mut size = UV_MAXHOSTNAMESIZE as usize;
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

/// Returns an estimate of the default amount of parallelism a program should use. Always returns a
/// non-zero value.
///
/// On Linux, inspects the calling thread’s CPU affinity mask to determine if it has been pinned to
/// specific CPUs.
///
/// On Windows, the available parallelism may be underreported on systems with more than 64 logical
/// CPUs.
///
/// On other platforms, reports the number of CPUs that the operating system considers to be
/// online.
pub fn available_parallelism() -> u32 {
    unsafe { uv_available_parallelism() as _ }
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
