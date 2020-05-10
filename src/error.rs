include!("./error.inc.rs");

use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use uv::{uv_err_name, uv_strerror};

impl Error {
    /// The name of the error.
    pub fn name(&self) -> String {
        unsafe {
            CStr::from_ptr(uv_err_name(self.code() as _))
                .to_string_lossy()
                .into_owned()
        }
    }

    /// A message for the error.
    pub fn message(&self) -> String {
        unsafe {
            CStr::from_ptr(uv_strerror(self.code() as _))
                .to_string_lossy()
                .into_owned()
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name(), self.message())
    }
}

impl std::error::Error for Error {}

#[derive(Clone, Copy, Debug)]
pub struct ConversionError {
    from: crate::HandleType,
    to: crate::HandleType,
}

impl ConversionError {
    pub(crate) fn new(from: crate::HandleType, to: crate::HandleType) -> ConversionError {
        ConversionError { from, to }
    }
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cannot convert {} to {}", self.from, self.to)
    }
}

impl std::error::Error for ConversionError {}
