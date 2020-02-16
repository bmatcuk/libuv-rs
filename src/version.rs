use std::ffi::CStr;
use uv::{uv_version, uv_version_string};

/// Returns the libuv version packed into a single integer. 8 bits are used for each component,
/// with the patch number stored in the 8 least significant bits. E.g. for libuv 1.2.3 this would
/// be 0x010203.
pub fn version() -> u32 {
    unsafe { uv_version() as _ }
}

/// Returns the libuv version number as a string. For non-release versions the version suffix is
/// included.
pub fn version_string() -> String {
    unsafe { CStr::from_ptr(uv_version_string()).to_string_lossy().into_owned() }
}
