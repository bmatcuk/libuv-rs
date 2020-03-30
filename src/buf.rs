use crate::{FromInner, Inner, IntoInner};
use std::borrow::Cow;
use std::ffi::{CStr, CString};
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
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
pub struct Buf {
    buf: *mut uv_buf_t,
}

impl Buf {
    fn alloc(size: usize) -> Result<*mut std::os::raw::c_char, Box<dyn std::error::Error>> {
        // this assumes layout.size() <= layout.align() - this is loosely based on the
        // experimental Rust features to create a Layout for an array
        let layout = std::alloc::Layout::new::<std::os::raw::c_char>();
        let alloc_size = layout
            .align()
            .checked_mul(size)
            .ok_or(crate::Error::ENOMEM)?;
        let layout = std::alloc::Layout::from_size_align(alloc_size, layout.align())?;
        Ok(unsafe { std::alloc::alloc(layout) as _ })
    }

    /// Create a new Buf with the given string
    pub fn new(s: &str) -> Result<Buf, Box<dyn std::error::Error>> {
        let bytes = s.as_bytes().len();
        let len = bytes + 1;
        let base = Buf::alloc(len)?;
        unsafe {
            base.copy_from_nonoverlapping(s.as_ptr() as _, bytes);
            base.add(bytes).write(0);
        }

        let buf = Box::new(unsafe { uv_buf_init(base, len as _) });
        Ok(Box::into_raw(buf).into_inner())
    }

    /// Create a Buf with the given capacity - the memory is not initialized
    pub fn with_capacity(size: usize) -> Result<Buf, Box<dyn std::error::Error>> {
        let base = Buf::alloc(size)?;
        let buf = Box::new(unsafe { uv_buf_init(base, size as _) });
        Ok(Box::into_raw(buf).into_inner())
    }

    /// Returns true if the internal buffer is initialized
    fn is_allocated(&self) -> bool {
        unsafe { !(*self.buf).base.is_null() }
    }

    /// Resizes the internal buffer - if the new size is smaller, no allocation takes place.
    /// Otherwise, a new buffer is initialized and the data from the old buffer is copied.
    pub fn resize(&mut self, size: usize) -> Result<(), Box<dyn std::error::Error>> {
        let len = unsafe { (*self.buf).len };
        if len > size {
            self.truncate(size);
        } else if len < size {
            let base = Buf::alloc(size)?;
            if self.is_allocated() {
                unsafe { base.copy_from_nonoverlapping((*self.buf).base, len) };
            }

            let buf = Box::new(unsafe { uv_buf_init(base, size as _) });
            self.destroy();
            self.buf = Box::into_raw(buf);
        }
        Ok(())
    }

    /// Truncate the length of the buffer
    pub fn truncate(&mut self, size: usize) {
        unsafe {
            assert!(
                size <= (*self.buf).len,
                "new size ({}) must be <= current size ({})",
                size,
                (*self.buf).len
            );
            (*self.buf).len = size;
        }
    }

    /// Deallocate the string inside the Buf, but leave the Buf intact.
    fn dealloc(&mut self) {
        unsafe {
            if self.is_allocated() {
                std::mem::drop(CString::from_raw((*self.buf).base));
                (*self.buf).base = std::ptr::null_mut();
                (*self.buf).len = 0;
            }
        }
    }

    /// Deallocates the Buf
    pub fn destroy(&mut self) {
        self.dealloc();
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
    type Error = Box<dyn std::error::Error>;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Buf::new(s)
    }
}

pub trait BufTrait {
    fn readonly(&self) -> ReadonlyBuf;
}

impl BufTrait for ReadonlyBuf {
    fn readonly(&self) -> ReadonlyBuf {
        ReadonlyBuf { buf: self.buf }
    }
}

impl BufTrait for Buf {
    fn readonly(&self) -> ReadonlyBuf {
        ReadonlyBuf { buf: self.buf }
    }
}

impl<T> FromInner<&[T]> for (*mut uv_buf_t, usize, usize)
where
    T: BufTrait,
{
    fn from_inner(bufs: &[T]) -> (*mut uv_buf_t, usize, usize) {
        // Buf/ReadonlyBuf objects contain pointers to uv_buf_t objects on the heap. However,
        // functions like uv_write, uv_udf_send, etc expect an array of uv_buf_t objects, *not* an
        // array of pointers. So, we need to create a Vec of copies of the data from the
        // dereferenced pointers.
        let mut bufs: Vec<uv::uv_buf_t> = bufs
            .iter()
            .map(|b| unsafe { *b.readonly().inner() }.clone())
            .collect();
        let bufs_ptr = bufs.as_mut_ptr();
        let bufs_len = bufs.len();
        let bufs_capacity = bufs.capacity();
        (bufs_ptr, bufs_len, bufs_capacity)
    }
}
