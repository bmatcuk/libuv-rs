use crate::{FromInner, HandleTrait, Inner, IntoInner, ToHandle};
use std::convert::{TryFrom, TryInto};
use uv::{
    uv_tty_get_vterm_state, uv_tty_get_winsize, uv_tty_init, uv_tty_reset_mode, uv_tty_set_mode,
    uv_tty_set_vterm_state, uv_tty_t,
};

/// TTY mode type
#[repr(u32)]
pub enum TtyMode {
    /// Initial/normal terminal mode
    Normal = uv::uv_tty_mode_t_UV_TTY_MODE_NORMAL,

    /// Raw input mode (On Windows, ENABLE_WINDOW_INPUT is also enabled)
    Raw = uv::uv_tty_mode_t_UV_TTY_MODE_RAW,

    /// Binary-safe I/O mode for IPC (Unix-only)
    IO = uv::uv_tty_mode_t_UV_TTY_MODE_IO,
}

/// Console virtual terminal mode type
#[repr(u32)]
pub enum VTermState {
    Supported = uv::uv_tty_vtermstate_t_UV_TTY_SUPPORTED,
    Unsupported = uv::uv_tty_vtermstate_t_UV_TTY_UNSUPPORTED,
}

/// TTY handles represent a stream for the console.
#[derive(Clone, Copy)]
pub struct TtyHandle {
    handle: *mut uv_tty_t,
}

impl TtyHandle {
    /// Initialize a new TTY stream with the given file descriptor. Usually the file descriptor
    /// will be:
    ///
    ///     0 = stdin
    ///     1 = stdout
    ///     2 = stderr
    ///
    /// On Unix this function will determine the path of the fd of the terminal using ttyname_r(3),
    /// open it, and use it if the passed file descriptor refers to a TTY. This lets libuv put the
    /// tty in non-blocking mode without affecting other processes that share the tty.
    ///
    /// This function is not thread safe on systems that don’t support ioctl TIOCGPTN or
    /// TIOCPTYGNAME, for instance OpenBSD and Solaris.
    ///
    /// Note: If reopening the TTY fails, libuv falls back to blocking writes.
    pub fn new(r#loop: &crate::Loop, fd: i32) -> crate::Result<TtyHandle> {
        let layout = std::alloc::Layout::new::<uv_tty_t>();
        let handle = unsafe { std::alloc::alloc(layout) as *mut uv_tty_t };
        if handle.is_null() {
            return Err(crate::Error::ENOMEM);
        }

        let ret = unsafe { uv_tty_init(r#loop.into_inner(), handle, fd, 0) };
        if ret < 0 {
            unsafe { std::alloc::dealloc(handle as _, layout) };
            return Err(crate::Error::from_inner(ret as uv::uv_errno_t));
        }

        crate::StreamHandle::initialize_data(uv_handle!(handle), super::NoAddlStreamData);

        Ok(TtyHandle { handle })
    }

    /// Set the TTY using the specified terminal mode.
    pub fn set_mode(&mut self, mode: TtyMode) -> crate::Result<()> {
        crate::uvret(unsafe { uv_tty_set_mode(self.handle, mode as _) })
    }

    /// To be called when the program exits. Resets TTY settings to default values for the next
    /// process to take over.
    ///
    /// This function is async signal-safe on Unix platforms but can fail with error code EBUSY if
    /// you call it when execution is inside uv_tty_set_mode().
    pub fn reset_mode() -> crate::Result<()> {
        crate::uvret(unsafe { uv_tty_reset_mode() })
    }

    /// Gets the current Window size.
    pub fn get_winsize(&self) -> crate::Result<(i32, i32)> {
        let mut width: std::os::raw::c_int = 0;
        let mut height: std::os::raw::c_int = 0;
        crate::uvret(unsafe { uv_tty_get_winsize(self.handle, &mut width as _, &mut height as _) })
            .map(|_| (width as _, height as _))
    }

    /// Controls whether console virtual terminal sequences are processed by libuv or console.
    /// Useful in particular for enabling ConEmu support of ANSI X3.64 and Xterm 256 colors.
    /// Otherwise Windows10 consoles are usually detected automatically.
    ///
    /// This function is only meaningful on Windows systems. On Unix it is silently ignored.
    pub fn set_vterm_state(state: VTermState) {
        unsafe { uv_tty_set_vterm_state(state as _) }
    }

    /// Get the current state of whether console virtual terminal sequences are handled by libuv or
    /// the console.
    ///
    /// This function is not implemented on Unix, where it returns UV_ENOTSUP.
    pub fn get_vterm_state() -> crate::Result<VTermState> {
        let mut state: uv::uv_tty_vtermstate_t = 0;
        crate::uvret(unsafe { uv_tty_get_vterm_state(&mut state as _) }).map(|_| match state {
            uv::uv_tty_vtermstate_t_UV_TTY_SUPPORTED => VTermState::Supported,
            _ => VTermState::Unsupported,
        })
    }
}

impl FromInner<*mut uv_tty_t> for TtyHandle {
    fn from_inner(handle: *mut uv_tty_t) -> TtyHandle {
        TtyHandle { handle }
    }
}

impl Inner<*mut uv_tty_t> for TtyHandle {
    fn inner(&self) -> *mut uv_tty_t {
        self.handle
    }
}

impl Inner<*mut uv::uv_stream_t> for TtyHandle {
    fn inner(&self) -> *mut uv::uv_stream_t {
        uv_handle!(self.handle)
    }
}

impl Inner<*mut uv::uv_handle_t> for TtyHandle {
    fn inner(&self) -> *mut uv::uv_handle_t {
        uv_handle!(self.handle)
    }
}

impl From<TtyHandle> for crate::StreamHandle {
    fn from(tty: TtyHandle) -> crate::StreamHandle {
        crate::StreamHandle::from_inner(Inner::<*mut uv::uv_stream_t>::inner(&tty))
    }
}

impl From<TtyHandle> for crate::Handle {
    fn from(tty: TtyHandle) -> crate::Handle {
        crate::Handle::from_inner(Inner::<*mut uv::uv_handle_t>::inner(&tty))
    }
}

impl crate::ToStream for TtyHandle {
    fn to_stream(&self) -> crate::StreamHandle {
        crate::StreamHandle::from_inner(Inner::<*mut uv::uv_stream_t>::inner(self))
    }
}

impl ToHandle for TtyHandle {
    fn to_handle(&self) -> crate::Handle {
        crate::Handle::from_inner(Inner::<*mut uv::uv_handle_t>::inner(self))
    }
}

impl TryFrom<crate::Handle> for TtyHandle {
    type Error = crate::ConversionError;

    fn try_from(handle: crate::Handle) -> Result<Self, Self::Error> {
        let t = handle.get_type();
        if t != crate::HandleType::TTY {
            Err(crate::ConversionError::new(t, crate::HandleType::TTY))
        } else {
            Ok((handle.inner() as *mut uv_tty_t).into_inner())
        }
    }
}

impl TryFrom<crate::StreamHandle> for TtyHandle {
    type Error = crate::ConversionError;

    fn try_from(stream: crate::StreamHandle) -> Result<Self, Self::Error> {
        stream.to_handle().try_into()
    }
}

impl crate::StreamTrait for TtyHandle {}
impl HandleTrait for TtyHandle {}

impl crate::Loop {
    /// Initialize a new TTY stream with the given file descriptor. Usually the file descriptor
    /// will be:
    ///
    ///     0 = stdin
    ///     1 = stdout
    ///     2 = stderr
    ///
    /// On Unix this function will determine the path of the fd of the terminal using ttyname_r(3),
    /// open it, and use it if the passed file descriptor refers to a TTY. This lets libuv put the
    /// tty in non-blocking mode without affecting other processes that share the tty.
    ///
    /// This function is not thread safe on systems that don’t support ioctl TIOCGPTN or
    /// TIOCPTYGNAME, for instance OpenBSD and Solaris.
    ///
    /// Note: If reopening the TTY fails, libuv falls back to blocking writes.
    pub fn tty(&self, fd: i32) -> crate::Result<TtyHandle> {
        TtyHandle::new(self, fd)
    }
}
