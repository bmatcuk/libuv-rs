use crate::{FromInner, Inner, IntoInner};
use std::borrow::Cow;
use std::ffi::CStr;
use std::ops::{
    Bound, Index, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo,
    RangeToInclusive,
};
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
    /// could run dealloc() in the read callback to deallocate the internal buffer - the allocate
    /// callback takes ownership of the actual Buf struct, so you don't need to worry about that.
    pub fn dealloc(&mut self) {
        unsafe {
            if self.is_allocated() {
                let len = (*self.buf).len as _;
                if let Ok(layout) = layout(len) {
                    std::alloc::dealloc((*self.buf).base as _, layout);
                }
            }
        }
    }

    /// Convert the Buf to a CStr. Returns an error if the Buf is empty. Data contained within the
    /// ReadonlyBuf must be null-terminated or this will fail!
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

    /// Convert the Buf to a string. Returns an error if the Buf is empty. Data contained within
    /// the ReadonlyBuf must be null-terminated or this will fail!
    pub fn to_string_lossy(&self) -> Result<Cow<'_, str>, EmptyBufError> {
        let cstr: &CStr = self.as_c_str()?;
        Ok(cstr.to_string_lossy())
    }

    /// Convert data in the Buf to a &str. Returns an error if the Buf is empty or the data is not
    /// valid utf8. Data does _not_ need to be null-terminated because only the first `len` bytes
    /// will be used to create the string.
    pub fn to_str(&self, len: usize) -> Result<&str, Box<dyn std::error::Error>> {
        let ptr: *const uv_buf_t = self.inner();
        unsafe {
            if (*ptr).base.is_null() {
                Err(Box::new(EmptyBufError))
            } else {
                Ok(std::str::from_utf8(std::slice::from_raw_parts(
                    (*ptr).base as _,
                    len,
                ))?)
            }
        }
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

impl Index<usize> for ReadonlyBuf {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        let len = if self.is_allocated() {
            unsafe { (*self.buf).len }
        } else {
            0
        };
        if len <= (index as _) {
            panic!("index {} out of range for Buf of length {}", index, len);
        }
        unsafe { &*((*self.buf).base.add(index) as *const u8) }
    }
}

/// Utility function to implement all of the Index<RangeX> traits. Unfortunately, I cannot just
/// impl Index<I: RangeBounds<usize>> for ReadonlyBuf because that precludes an implementation for
/// usize alone.
fn range_from_readonlybuf<I>(buf: &ReadonlyBuf, index: I) -> &[u8]
where
    I: RangeBounds<usize>,
{
    let len = if buf.is_allocated() {
        unsafe { (*buf.buf).len as usize }
    } else {
        0
    };

    let start = match index.start_bound() {
        Bound::Included(i) => *i,
        Bound::Excluded(i) => *i + 1,
        Bound::Unbounded => 0,
    };
    let end = match index.end_bound() {
        Bound::Included(i) => *i + 1,
        Bound::Excluded(i) => *i,
        Bound::Unbounded => len,
    };

    if start > end {
        panic!("Buf index starts at {} but ends at {}", start, end);
    }

    if len <= end {
        panic!("index {} out of range for Buf of length {}", end, len);
    }

    unsafe { std::slice::from_raw_parts((*buf.buf).base.add(start) as *const u8, end - start) }
}

impl Index<Range<usize>> for ReadonlyBuf {
    type Output = [u8];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        range_from_readonlybuf(self, index)
    }
}

impl Index<RangeFrom<usize>> for ReadonlyBuf {
    type Output = [u8];

    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        range_from_readonlybuf(self, index)
    }
}

impl Index<RangeFull> for ReadonlyBuf {
    type Output = [u8];

    fn index(&self, index: RangeFull) -> &Self::Output {
        range_from_readonlybuf(self, index)
    }
}

impl Index<RangeInclusive<usize>> for ReadonlyBuf {
    type Output = [u8];

    fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
        range_from_readonlybuf(self, index)
    }
}

impl Index<RangeTo<usize>> for ReadonlyBuf {
    type Output = [u8];

    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        range_from_readonlybuf(self, index)
    }
}

impl Index<RangeToInclusive<usize>> for ReadonlyBuf {
    type Output = [u8];

    fn index(&self, index: RangeToInclusive<usize>) -> &Self::Output {
        range_from_readonlybuf(self, index)
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
        Buf::new_from_bytes(s.as_bytes())
    }

    /// Create a new Buf from the given byte slice
    pub fn new_from_bytes(bytes: &[u8]) -> Result<Buf, Box<dyn std::error::Error>> {
        let len = bytes.len();
        let buflen = len + 1;
        let base = Buf::alloc(buflen)?;
        unsafe {
            base.copy_from_nonoverlapping(bytes.as_ptr() as _, len);
            base.add(len).write(0);
        }

        let buf = Box::new(unsafe { uv_buf_init(base, buflen as _) });
        Ok(Box::into_raw(buf).into_inner())
    }

    /// Create a Buf with the given capacity - the memory is not initialized
    pub fn with_capacity(size: usize) -> crate::Result<Buf> {
        let base = Buf::alloc(size)?;
        let buf = Box::new(unsafe { uv_buf_init(base, size as _) });
        Ok(Box::into_raw(buf).into_inner())
    }

    /// Create a duplicate of this Buf - if the optional size parameter is None, the new Buf will
    /// have the same size as the existing Buf. Otherwise, the new Buf will have the specified size
    /// and data up to that size, or the size of the original buf, whichever is lower, will be
    /// copied.
    pub fn new_from(other: &impl BufTrait, size: Option<usize>) -> crate::Result<Self> {
        let other = other.readonly();
        if !other.is_allocated() {
            if let Some(s) = size {
                return Buf::with_capacity(s);
            }
            return Ok(Buf {
                buf: std::ptr::null_mut(),
            });
        }

        let len = if let Some(s) = size {
            s
        } else {
            unsafe { (*other.buf).len as _ }
        };

        let mut buf = Buf::with_capacity(len)?;
        buf.copy_from(&other)?;
        Ok(buf)
    }

    /// Returns true if the internal buffer is initialized
    pub fn is_allocated(&self) -> bool {
        unsafe { !(*self.buf).base.is_null() }
    }

    /// Resizes the internal buffer
    pub fn resize(&mut self, size: usize) -> crate::Result<()> {
        if self.is_allocated() {
            let len = unsafe { (*self.buf).len as _ };
            if len != size {
                let (alloc_size, _) = calc_alloc_size_alignment(size)?;
                let layout = layout(len)?;
                let ptr = unsafe { std::alloc::realloc((*self.buf).base as _, layout, alloc_size) };
                if ptr.is_null() {
                    return Err(crate::Error::ENOMEM);
                }
                unsafe {
                    (*self.buf).base = ptr as _;
                    (*self.buf).len = alloc_size as _;
                }
            }
        } else {
            let base = Buf::alloc(size)?;
            unsafe {
                (*self.buf).base = base as _;
                (*self.buf).len = size as _;
            }
        }
        Ok(())
    }

    /// Copies the data from a Buf to this one.
    pub fn copy_from(&mut self, other: &impl BufTrait) -> crate::Result<()> {
        let other = other.readonly();
        if !other.is_allocated() {
            return Ok(());
        }

        let other_len = unsafe { (*other.buf).len as _ };
        if !self.is_allocated() {
            self.resize(other_len)?;
        }

        let my_len = unsafe { (*self.buf).len as usize };
        let len = my_len.min(other_len);
        unsafe {
            (*self.buf)
                .base
                .copy_from_nonoverlapping((*other.buf).base, len)
        };

        Ok(())
    }

    /// Deallocate the internal buffer, but leave the Buf intact.
    pub fn dealloc(&mut self) {
        unsafe {
            if self.is_allocated() {
                let len = (*self.buf).len as _;
                if let Ok(layout) = layout(len) {
                    std::alloc::dealloc((*self.buf).base as _, layout);
                    (*self.buf).base = std::ptr::null_mut();
                    (*self.buf).len = 0;
                }
            }
        }
    }

    /// Deallocates the Buf struct, leaving the internal buffer alone. This is used by alloc_cb.
    pub(crate) fn destroy_container(&mut self) {
        std::mem::drop(unsafe { Box::from_raw(self.buf) });
    }

    /// Deallocates the internal buffer *and* the Buf
    pub fn destroy(&mut self) {
        self.dealloc();
        self.destroy_container();
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
        let mut bufs: std::mem::ManuallyDrop<Vec<uv::uv_buf_t>> = std::mem::ManuallyDrop::new(
            bufs.iter()
                .map(|b| unsafe { *b.readonly().inner() }.clone())
                .collect(),
        );
        let bufs_ptr = bufs.as_mut_ptr();
        let bufs_len = bufs.len();
        let bufs_capacity = bufs.capacity();
        (bufs_ptr, bufs_len, bufs_capacity)
    }
}
