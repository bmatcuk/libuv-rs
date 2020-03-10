use crate::{FromInner, IntoInner};
use std::ffi::CString;
use uv::uv_stdio_container_s__bindgen_ty_1 as uv_stdio_container_data;
use uv::{
    uv_disable_stdio_inheritance, uv_kill, uv_process_get_pid, uv_process_kill,
    uv_process_options_t, uv_process_t, uv_spawn, uv_stdio_container_t,
};

/// Additional data stored on the handle
#[derive(Default)]
pub(crate) struct ProcessDataFields {
    exit_cb: Option<Box<dyn FnMut(ProcessHandle, i64, i32)>>,
}

/// Callback for uv_process_options_t.exit_cb
extern "C" fn uv_exit_cb(
    handle: *mut uv_process_t,
    exit_status: i64,
    term_signal: std::os::raw::c_int,
) {
    let dataptr = crate::Handle::get_data(uv_handle!(handle));
    if !dataptr.is_null() {
        unsafe {
            if let crate::ProcessData(d) = &mut (*dataptr).addl {
                if let Some(f) = d.exit_cb.as_mut() {
                    f(handle.into_inner(), exit_status, term_signal as _);
                }
            }
        }
    }
}

bitflags! {
    /// Flags specifying how a stdio should be transmitted to the child process.
    pub struct StdioFlags: u32 {
        const IGNORE = uv::uv_stdio_flags_UV_IGNORE;
        const CREATE_PIPE = uv::uv_stdio_flags_UV_CREATE_PIPE;
        const INHERIT_FD = uv::uv_stdio_flags_UV_INHERIT_FD;
        const INHERIT_STREAM = uv::uv_stdio_flags_UV_INHERIT_STREAM;

        /// When UV_CREATE_PIPE is specified, UV_READABLE_PIPE and UV_WRITABLE_PIPE determine the
        /// direction of flow, from the child process' perspective. Both flags may be specified to
        /// create a duplex data stream.
        const READABLE_PIPE = uv::uv_stdio_flags_UV_READABLE_PIPE;
        const WRITABLE_PIPE = uv::uv_stdio_flags_UV_WRITABLE_PIPE;

        /// Open the child pipe handle in overlapped mode on Windows. On Unix it is silently
        /// ignored.
        const OVERLAPPED_PIPE = uv::uv_stdio_flags_UV_OVERLAPPED_PIPE;
    }
}

bitflags! {
    /// Flags to be set on the flags field of ProcessOptions.
    pub struct ProcessFlags: u32 {
        /// Set the child process' user id.
        const SETUID = uv::uv_process_flags_UV_PROCESS_SETUID;

        /// Set the child process' group id.
        const SETGID = uv::uv_process_flags_UV_PROCESS_SETGID;

        /// Do not wrap any arguments in quotes, or perform any other escaping, when converting the
        /// argument list into a command line string. This option is only meaningful on Windows
        /// systems. On Unix it is silently ignored.
        const WINDOWS_VERBATIM_ARGUMENTS = uv::uv_process_flags_UV_PROCESS_WINDOWS_VERBATIM_ARGUMENTS;

        /// Spawn the child process in a detached state - this will make it a process group leader,
        /// and will effectively enable the child to keep running after the parent exits. Note that
        /// the child process will still keep the parent's event loop alive unless the parent
        /// process calls uv_unref() on the child's process handle.
        const DETACHED = uv::uv_process_flags_UV_PROCESS_DETACHED;

        /// Hide the subprocess window that would normally be created. This option is only
        /// meaningful on Windows systems. On Unix it is silently ignored.
        const WINDOWS_HIDE = uv::uv_process_flags_UV_PROCESS_WINDOWS_HIDE;

        /// Hide the subprocess console window that would normally be created. This option is only
        /// meaningful on Windows systems. On Unix it is silently ignored.
        const WINDOWS_HIDE_CONSOLE = uv::uv_process_flags_UV_PROCESS_WINDOWS_HIDE_CONSOLE;

        /// Hide the subprocess GUI window that would normally be created. This option is only
        /// meaningful on Windows systems. On Unix it is silently ignored.
        const WINDOWS_HIDE_GUI = uv::uv_process_flags_UV_PROCESS_WINDOWS_HIDE_GUI;
    }
}

pub enum StdioType {
    Stream(crate::StreamHandle),
    Fd(i32),
}

impl IntoInner<uv_stdio_container_data> for StdioType {
    fn into_inner(self) -> uv_stdio_container_data {
        match self {
            StdioType::Stream(s) => uv_stdio_container_data { stream: s.into_inner() },
            StdioType::Fd(fd) => uv_stdio_container_data { fd },
        }
    }
}

/// Container for each stdio handle or fd passed to a child process.
pub struct StdioContainer {
    flags: StdioFlags,
    data: StdioType,
}

/// Options for spawning the process (passed to spawn()).
pub struct ProcessOptions<'a> {
    /// Called after the process exits.
    exit_cb: Option<Box<dyn FnMut(ProcessHandle, i64, i32)>>,

    /// Path to program to execute.
    file: &'a str,

    /// Command line arguments. args[0] should be the path to the program. On Windows this uses
    /// CreateProcess which concatenates the arguments into a string this can cause some strange
    /// errors. See the note at windows_verbatim_arguments.
    args: &'a [&'a str],

    /// This will be set as the environ variable in the subprocess. If this is None then the
    /// parents environ will be used.
    env: Option<&'a [&'a str]>,

    /// If Some() this represents a directory the subprocess should execute in. Stands for current
    /// working directory.
    cwd: Option<&'a str>,

    /// Various flags that control how spawn() behaves. See the definition of `ProcessFlags`.
    flags: ProcessFlags,

    /// The `stdio` field points to an array of StdioContainer structs that describe the file
    /// descriptors that will be made available to the child process. The convention is that
    /// stdio[0] points to stdin, fd 1 is used for stdout, and fd 2 is stderr.
    ///
    /// Note that on windows file descriptors greater than 2 are available to the child process
    /// only if the child processes uses the MSVCRT runtime.
    stdio: &'a [StdioContainer],

    /// Libuv can change the child process' user/group id. This happens only when the appropriate
    /// bits are set in the flags fields. This is not supported on windows; spawn() will fail and
    /// set the error to ENOTSUP.
    uid: u32,

    /// Libuv can change the child process' user/group id. This happens only when the appropriate
    /// bits are set in the flags fields. This is not supported on windows; spawn() will fail and
    /// set the error to ENOTSUP.
    gid: u32,
}

/// Process handles will spawn a new process and allow the user to control it and establish
/// communication channels with it using streams.
pub struct ProcessHandle {
    handle: *mut uv_process_t,
}

impl ProcessHandle {
    /// Create a new process handle
    pub fn new() -> crate::Result<ProcessHandle> {
        let layout = std::alloc::Layout::new::<uv_process_t>();
        let handle = unsafe { std::alloc::alloc(layout) as *mut uv_process_t };
        if handle.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        crate::Handle::initialize_data(uv_handle!(handle), crate::ProcessData(Default::default()));

        Ok(ProcessHandle { handle })
    }

    /// Disables inheritance for file descriptors / handles that this process inherited from its
    /// parent. The effect is that child processes spawned by this process don’t accidentally
    /// inherit these handles.
    ///
    /// It is recommended to call this function as early in your program as possible, before the
    /// inherited file descriptors can be closed or duplicated.
    ///
    /// Note: This function works on a best-effort basis: there is no guarantee that libuv can
    /// discover all file descriptors that were inherited. In general it does a better job on
    /// Windows than it does on Unix.
    pub fn disable_stdio_inheritance() {
        unsafe { uv_disable_stdio_inheritance() };
    }

    /// Initializes the process handle and starts the process.
    ///
    /// Possible reasons for failing to spawn would include (but not be limited to) the file to
    /// execute not existing, not having permissions to use the setuid or setgid specified, or not
    /// having enough memory to allocate for the new process.
    pub fn spawn(
        &mut self,
        r#loop: &crate::Loop,
        options: ProcessOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dataptr = crate::Handle::get_data(uv_handle!(self.handle));
        if !dataptr.is_null() {
            if let crate::ProcessData(d) = unsafe { &mut (*dataptr).addl } {
                d.exit_cb = options.exit_cb;
            }
        }

        // CString will ensure we have a terminating null
        let file = CString::new(options.file)?;

        // For args, libuv-sys is expecting a "*mut *mut c_char". The only way to get a "*mut
        // c_char" from a CString is via CString::into_raw() which will "leak" the memory from
        // rust. We'll need to make sure to reclaim that memory later so it'll be GC'd. So, first
        // we need to convert all of the arguments to CStrings for the null-termination. Then we
        // need to grab a *mut pointer to the data using CString::into_raw() which will "leak" the
        // CStrings out of rust. Then we need to add a final null pointer to the end (the C code
        // requires it so it can find the end of the array) and collect it all into a Vec.
        let args = options
            .args
            .iter()
            .map(|a| CString::new(*a).map(|s| s.into_raw()))
            .chain(std::iter::once(Ok(std::ptr::null_mut())))
            .collect::<Result<Vec<*mut std::os::raw::c_char>, std::ffi::NulError>>()?;

        // env is similar to args except that it is Option'al.
        let env = options
            .env
            .map(|env| {
                env.iter()
                    .map(|e| CString::new(*e).map(|s| s.into_raw()))
                    .chain(std::iter::once(Ok(std::ptr::null_mut())))
                    .collect::<Result<Vec<*mut std::os::raw::c_char>, std::ffi::NulError>>()
            })
            .transpose()?;

        // cwd is like file except it's Option'al
        let cwd = options.cwd.map(|cwd| CString::new(cwd)).transpose()?;

        // stdio is an array of uv_stdio_container_t objects
        let stdio = options
            .stdio
            .iter()
            .map(|stdio| uv_stdio_container_t {
                flags: stdio.flags.bits(),
                data: stdio.data.into_inner(),
            })
            .collect::<Vec<uv_stdio_container_t>>();

        let options = uv_process_options_t {
            exit_cb: options.exit_cb.map(|_| uv_exit_cb as _),
            file: file.as_ptr(),
            args: args.as_mut_ptr(),
            env: env.map_or(std::ptr::null_mut(), |e| e.as_mut_ptr()),
            cwd: cwd.map_or(std::ptr::null(), |s| s.as_ptr()),
            flags: options.flags.bits(),
            stdio_count: options.stdio.len() as _,
            stdio: stdio.as_mut_ptr(),
            uid: options.uid,
            gid: options.gid,
        };

        let result = crate::uvret(unsafe {
            uv_spawn(r#loop.into_inner(), self.handle, &options as *const _)
        })
        .map_err(|e| Box::new(e) as _);

        // reclaim data so it'll be freed - I'm pretty sure it's safe to free options here. Under
        // the hood, libuv is calling fork and execvp. The fork should copy the address space into
        // the new process, so freeing it here shouldn't affect that. Then execvp is going to
        // replace the address space, so we don't need to worry about leaking the copy.

        // For args, we don't need the last element because it's a null pointer.
        let args: Vec<CString> = args
            .iter()
            .take(args.len() - 1)
            .map(|a| CString::from_raw(*a))
            .collect();

        // env is the same as args except it's Option'al
        let env: Option<Vec<CString>> = env.map(|env| {
            env.iter()
                .take(env.len() - 1)
                .map(|e| CString::from_raw(*e))
                .collect()
        });

        result
    }

    /// The PID of the spawned process. It’s set after calling spawn().
    fn pid(&self) -> i32 {
        unsafe { uv_process_get_pid(self.handle) as _ }
    }

    /// Sends the specified signal to the given process handle. Check the documentation on
    /// SignalHandle for signal support, specially on Windows.
    fn kill(&mut self, signum: i32) -> crate::Result<()> {
        crate::uvret(unsafe { uv_process_kill(self.handle, signum) })
    }

    /// Sends the specified signal to the given PID. Check the documentation on SignalHandle for
    /// signal support, specially on Windows.
    fn kill_pid(pid: i32, signum: i32) -> crate::Result<()> {
        crate::uvret(unsafe { uv_kill(pid, signum) })
    }
}

impl FromInner<*mut uv_process_t> for ProcessHandle {
    fn from_inner(handle: *mut uv_process_t) -> ProcessHandle {
        ProcessHandle { handle }
    }
}

impl IntoInner<*mut uv::uv_handle_t> for ProcessHandle {
    fn into_inner(self) -> *mut uv::uv_handle_t {
        uv_handle!(self.handle)
    }
}

impl From<ProcessHandle> for crate::Handle {
    fn from(process: ProcessHandle) -> crate::Handle {
        process.into_inner().into_inner()
    }
}

impl crate::HandleTrait for ProcessHandle {}

impl crate::Loop {
    /// Create a new process handle and spawn the process
    pub fn spawn_process(
        &self,
        options: ProcessOptions,
    ) -> Result<ProcessHandle, Box<dyn std::error::Error>> {
        let process = ProcessHandle::new()?;
        process.spawn(self, options)?;
        Ok(process)
    }
}
