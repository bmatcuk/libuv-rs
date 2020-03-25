use crate::{FromInner, Inner, IntoInner};
use std::ffi::{CStr, CString};
use uv::{uv_dlclose, uv_dlerror, uv_dlopen, uv_dlsym, uv_lib_t};

/// Returns an error from DLib::open() or DLib::sym()
#[derive(Debug)]
pub struct DLError(String);

impl DLError {
    /// Construct a new DLError
    fn new(lib: &DLib) -> DLError {
        let ptr = unsafe { uv_dlerror(lib.inner()) };
        DLError(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
    }

    /// Retrieve the error message
    pub fn message(&self) -> String {
        self.0.clone()
    }
}

impl std::fmt::Display for DLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DLError {}

/// Shared library data type.
pub struct DLib {
    lib: *mut uv_lib_t,
}

impl DLib {
    /// Construct a new DLib
    fn new() -> crate::Result<DLib> {
        let layout = std::alloc::Layout::new::<uv_lib_t>();
        let lib = unsafe { std::alloc::alloc(layout) as *mut uv_lib_t };
        if lib.is_null() {
            return Err(crate::Error::ENOMEM);
        }
        Ok(DLib { lib })
    }

    /// Opens a shared library. The filename is in utf-8.
    pub fn open(filename: &str) -> Result<DLib, Box<dyn std::error::Error>> {
        let filename = CString::new(filename)?;
        let lib = DLib::new()?;
        let libptr = lib.inner();
        let result = unsafe {
            uv_dlopen(filename.as_ptr(), libptr)
        };
        if result < 0 {
            Err(Box::new(DLError::new(&lib)))
        } else {
            Ok(lib)
        }
    }

    /// Close the shared library.
    pub fn close(self) {
        unsafe { uv_dlclose(self.lib.into_inner()) };
    }

    /// Retrieves a data pointer from a dynamic library. It is legal for a symbol to map to NULL.
    /// Returns a DLError if the symbol was not found.
    pub fn sym<T>(&self, name: &str) -> Result<*mut T, Box<dyn std::error::Error>> {
        let name = CString::new(name)?;
        let mut ptr: *mut std::os::raw::c_void = std::ptr::null_mut();
        let result = unsafe {
            uv_dlsym((*self).inner(), name.as_ptr(), &mut ptr)
        };
        if result < 0 {
            Err(Box::new(DLError::new(self)))
        } else {
            Ok(ptr as _)
        }
    }
}

impl Drop for DLib {
    fn drop(&mut self) {
        let layout = std::alloc::Layout::new::<uv_lib_t>();
        unsafe { std::alloc::dealloc(self.lib as _, layout) };
    }
}

impl FromInner<*mut uv_lib_t> for DLib {
    fn from_inner(lib: *mut uv_lib_t) -> DLib {
        DLib { lib }
    }
}

impl Inner<*mut uv_lib_t> for DLib {
    fn inner(&self) -> *mut uv_lib_t {
        self.lib
    }
}

impl Inner<*const uv_lib_t> for DLib {
    fn inner(&self) -> *const uv_lib_t {
        self.lib as _
    }
}
