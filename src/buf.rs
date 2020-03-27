use crate::{FromInner, Inner, IntoInner};
use std::borrow::Cow;
use std::ffi::{CStr, CString, NulError};
use uv::{uv_buf_init, uv_buf_t};

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
#[derive(Clone)]
pub struct ReadonlyBuf {
    buf: *const uv_buf_t,
}

impl ReadonlyBuf {
    /// Convert the Buf to a CStr. Returns an error if the Buf is empty.
    pub fn as_c_str(&self) -> Result<&'_ CStr, EmptyBufError> {
        let ptr: *const uv_buf_t = self.inner();
        unsafe {
            if (*ptr).base.is_null() {
                Err(EmptyBufError)
            } else {
                Ok(CStr::from_ptr((*ptr).base))
            }
        }
    }

    /// Convert the Buf to a string. Returns an error if the Buf is empty.
    pub fn to_string_lossy(&self) -> Result<Cow<'_, str>, EmptyBufError> {
        let cstr: &CStr = self.as_c_str()?;
        Ok(cstr.to_string_lossy())
    }
}

impl FromInner<*const uv_buf_t> for ReadonlyBuf {
    fn from_inner(buf: *const uv_buf_t) -> ReadonlyBuf {
        ReadonlyBuf { buf }
    }
}

impl Inner<*const uv_buf_t> for ReadonlyBuf {
    fn inner(&self) -> *const uv_buf_t {
        self.buf
    }
}

/// Buffer data type.
#[derive(Clone)]
pub struct Buf {
    buf: *mut uv_buf_t,
}

impl Buf {
    /// Create a new Buf with the given string
    pub fn new(s: &str) -> Result<Buf, NulError> {
        let s = CString::new(s)?;
        let len = s.as_bytes().len();
        let buf = Box::new(unsafe { uv_buf_init(s.into_raw(), len as _) });
        Ok(Box::into_raw(buf).into_inner())
    }

    /// Deallocate the string inside the Buf, but leave the Buf intact.
    pub fn dealloc_string(&mut self) {
        unsafe {
            if !(*self.buf).base.is_null() {
                std::mem::drop(CString::from_raw((*self.buf).base));
                (*self.buf).base = std::ptr::null_mut();
                (*self.buf).len = 0;
            }
        }
    }

    /// Deallocates the string inside the Buf, *and* deallocs the Buf itself
    pub fn dealloc(&mut self) {
        self.dealloc_string();
        std::mem::drop(unsafe { Box::from_raw(self.buf) });
    }
}

impl FromInner<*mut uv_buf_t> for Buf {
    fn from_inner(buf: *mut uv_buf_t) -> Buf {
        Buf { buf }
    }
}

impl Inner<*mut uv_buf_t> for Buf {
    fn inner(&self) -> *mut uv_buf_t {
        self.buf
    }
}

impl Inner<*const uv_buf_t> for Buf {
    fn inner(&self) -> *const uv_buf_t {
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

impl<T> FromInner<&[T]> for (*mut uv_buf_t, usize, usize)
where
    T: Inner<*const uv_buf_t>
{
    fn from_inner(bufs: &[T]) -> (*mut uv_buf_t, usize, usize) {
        // Buf/ReadonlyBuf objects contain pointers to uv_buf_t objects on the heap. However,
        // functions like uv_write, uv_udf_send, etc expect an array of uv_buf_t objects, *not* an
        // array of pointers. So, we need to create a Vec of copies of the data from the
        // dereferenced pointers.
        let mut bufs: Vec<uv::uv_buf_t> = bufs.iter().map(|b| unsafe { *b.inner() }.clone()).collect();
        let bufs_ptr = bufs.as_mut_ptr();
        let bufs_len = bufs.len();
        let bufs_capacity = bufs.capacity();
        (bufs_ptr, bufs_len, bufs_capacity)
    }
}
