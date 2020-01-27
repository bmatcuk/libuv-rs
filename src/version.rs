use std::ffi::CStr;
use uv::{uv_version, uv_version_string};

pub fn version() -> u32 {
    unsafe { uv_version() as _ }
}

pub fn version_string() -> String {
    unsafe { CStr::from_ptr(uv_version_string()).to_lossy_string().into_owned() }
}
