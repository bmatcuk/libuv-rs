use std::borrow::Cow;
use std::ffi::{CStr, CString, NulError};
use uv::{uv_buf_t, uv_buf_init};

/// When trying to convert an empty Buf to a string.
#[derive(Debug)]
pub struct EmptyBufError;

impl std::fmt::Display for EmptyBufError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("The Buf is empty.")
    }
}

impl std::error::Error for EmptyBufError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// Readonly buffer data type.
pub struct ReadonlyBuf {
    buf: *const uv_buf_t
}

impl From<*const uv_buf_t> for ReadonlyBuf {
    fn from(buf: *const uv_buf_t) -> ReadonlyBuf {
        ReadonlyBuf { buf }
    }
}

impl Into<*const uv_buf_t> for ReadonlyBuf {
    fn into(self) -> *const uv_buf_t {
        self.buf
    }
}

/// Buffer data type.
pub struct Buf {
    buf: *mut uv_buf_t
}

impl Buf {
    /// Create a new Buf with the given string
    pub fn new(s: &str) -> Result<Buf, NulError> {
        let s = CString::new(s)?;
        let buf = Box::new(unsafe { uv_buf_init(s.into_raw(), s.as_bytes().len() as _) });
        Ok(Box::into_raw(buf).into())
    }

    /// Deallocate the string inside the Buf, but leave the Buf intact.
    pub fn dealloc_string(&mut self) {
        if !(*self.buf).base.is_null() {
            std::mem::drop(CString::from_raw((*self.buf).base));
            (*self.buf).base = std::ptr::null_mut();
            (*self.buf).len = 0;
        }
    }

    /// Deallocates the string inside the Buf, *and* deallocs the Buf itself
    pub fn dealloc(&mut self) {
        self.dealloc_string();
        std::mem::drop(Box::from_raw(self.buf));
    }
}

impl From<*mut uv_buf_t> for Buf {
    fn from(buf: *mut uv_buf_t) -> Buf {
        Buf { buf }
    }
}

impl Into<*mut uv_buf_t> for Buf {
    fn into(self) -> *mut uv_buf_t {
        self.buf
    }
}

impl Into<*const uv_buf_t> for Buf {
    fn into(self) -> *const uv_buf_t {
        self.buf
    }
}

impl From<Buf> for ReadonlyBuf {
    fn from(buf: Buf) -> ReadonlyBuf {
        ReadonlyBuf { buf: buf.buf }
    }
}

impl std::convert::TryFrom<&str> for Buf {
    type Error = NulError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Buf::new(s)
    }
}

pub trait BufTrait: Into<*const uv_buf_t> {
    /// Convert the Buf to a CStr. Returns an error if the Buf is empty.
    fn as_c_str(&self) -> Result<&'_ CStr, EmptyBufError> {
        let ptr: *const uv_buf_t = (*self).into();
        if (*ptr).base.is_null() {
            Err(EmptyBufError)
        } else {
            Ok(CStr::from_ptr((*ptr).base))
        }
    }

    /// Convert the Buf to a string. Returns an error if the Buf is empty.
    fn to_string_lossy(&self) -> Result<Cow<'_, str>, EmptyBufError> {
        let cstr: &CStr = self.as_c_str()?;
        Ok(cstr.to_string_lossy())
    }
}

impl BufTrait for ReadonlyBuf {}
impl BufTrait for Buf {}

impl<'a> std::convert::TryInto<&'a CStr> for ReadonlyBuf {
    type Error = EmptyBufError;

    fn try_into(self) -> Result<&'a CStr, EmptyBufError> {
        self.as_c_str()
    }
}

impl<'a> std::convert::TryInto<&'a CStr> for Buf {
    type Error = EmptyBufError;

    fn try_into(self) -> Result<&'a CStr, EmptyBufError> {
        self.as_c_str()
    }
}

impl<'a> std::convert::TryInto<Cow<'a, str>> for Buf {
    type Error = EmptyBufError;

    fn try_into(self) -> Result<Cow<'a, str>, EmptyBufError> {
        self.to_string_lossy()
    }
}

impl<'a> std::convert::TryInto<Cow<'a, str>> for ReadonlyBuf {
    type Error = EmptyBufError;

    fn try_into(self) -> Result<Cow<'a, str>, EmptyBufError> {
        self.to_string_lossy()
    }
}
