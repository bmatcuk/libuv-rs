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

/// Calculates how much space to allocate and the alignment
fn calc_alloc_size_alignment(size: usize) -> crate::Result<(usize, usize)> {
    // this assumes layout.size() <= layout.align() - this is loosely based on the
    // experimental Rust features to create a Layout for an array
    let layout = std::alloc::Layout::new::<std::os::raw::c_char>();
    let alloc_size = layout
        .align()
        .checked_mul(size)
        .ok_or(crate::Error::ENOMEM)?;
    Ok((alloc_size, layout.align()))
}

/// Creates a Layout for the given size
fn layout(size: usize) -> crate::Result<std::alloc::Layout> {
    let (alloc_size, align) = calc_alloc_size_alignment(size)?;
    std::alloc::Layout::from_size_align(alloc_size, align).or(Err(crate::Error::ENOMEM))
}

/// Readonly buffer data type.
#[derive(Clone, Copy)]
pub struct ReadonlyBuf {
    buf: *const uv_buf_t,
}

impl ReadonlyBuf {
    /// Returns true if the internal buffer is initialized
    pub fn is_allocated(&self) -> bool {
        unsafe { !(*self.buf).base.is_null() }
    }

    /// Deallocate the internal buffer, but leave the Buf intact. Even though this is a "readonly"
    /// Buf, the internal storage can still be deallocated. This oddity is an unfortunate
    /// side-effect of the libuv API: for example, StreamHandle::read_start calls the allocate
    /// callback to create a Buf, then passes that Buf to the read callback as a ReadonlyBuf. You
    /// could run dealloc() in the read callback to deallocate the internal buffer, but then you're
    /// still leaking the Buf itself. Perhaps your allocate callback reuses Bufs?
    pub fn dealloc(&mut self) {
        unsafe {
            if self.is_allocated() {
                let len = (*self.buf).len;
                if let Ok(layout) = layout(len) {
                    std::alloc::dealloc((*self.buf).base as _, layout);
                }
            }
        }
    }

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
    fn alloc(size: usize) -> crate::Result<*mut std::os::raw::c_char> {
        let layout = layout(size)?;
        let ptr = unsafe { std::alloc::alloc(layout) as *mut std::os::raw::c_char };
        if ptr.is_null() {
            Err(crate::Error::ENOMEM)
        } else {
            Ok(ptr)
        }
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
    pub fn is_allocated(&self) -> bool {
        unsafe { !(*self.buf).base.is_null() }
    }

    /// Resizes the internal buffer
    pub fn resize(&mut self, size: usize) -> crate::Result<()> {
        let len = unsafe { (*self.buf).len };
        if len != size {
            let (alloc_size, _) = calc_alloc_size_alignment(size)?;
            let layout = layout(len)?;
            let ptr = unsafe { std::alloc::realloc((*self.buf).base as _, layout, alloc_size) };
            if ptr.is_null() {
                return Err(crate::Error::ENOMEM);
            }
            unsafe { (*self.buf).base = ptr as _ };
        }
        Ok(())
    }

    /// Deallocate the internal buffer, but leave the Buf intact.
    pub fn dealloc(&mut self) {
        unsafe {
            if self.is_allocated() {
                let len = (*self.buf).len;
                if let Ok(layout) = layout(len) {
                    std::alloc::dealloc((*self.buf).base as _, layout);
                    (*self.buf).base = std::ptr::null_mut();
                    (*self.buf).len = 0;
                }
            }
        }
    }

    /// Deallocates the internal buffer *and* the Buf
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
