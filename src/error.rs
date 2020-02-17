include!("./error.inc.rs");

use std::ffi::CStr;
use uv::{uv_err_name, uv_strerror};

impl Error {
    /// The name of the error.
    pub fn name(&self) -> String {
        unsafe { CStr::from_ptr(uv_err_name(self.code() as _)).to_string_lossy().into_owned() }
    }

    /// A message for the error.
    pub fn message(&self) -> String {
        unsafe { CStr::from_ptr(uv_strerror(self.code() as _)).to_string_lossy().into_owned() }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name(), self.message())
    }
}

impl std::error::Error for Error {}
