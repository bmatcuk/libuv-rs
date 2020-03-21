use crate::IntoInner;
use std::ffi::{CStr, CString};
use uv::{uv_chdir, uv_cwd, uv_exepath, uv_guess_handle};

/// Changes the current working directory.
pub fn chdir(dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let dir = CString::new(dir)?;
    crate::uvret(unsafe { uv_chdir(dir.as_ptr()) }).map_err(|e| Box::new(e) as _)
}

/// Gets the current working directory.
pub fn cwd() -> crate::Result<String> {
    let mut size = 0usize;
    unsafe { uv_cwd(std::ptr::null_mut(), &mut size as _) };

    let mut buf: Vec<std::os::raw::c_uchar> = Vec::with_capacity(size);
    crate::uvret(unsafe {
        uv_cwd(buf.as_mut_ptr() as _, &mut size as _)
    })
    .map(|_| {
        unsafe { buf.set_len(size) };
        unsafe { CStr::from_bytes_with_nul_unchecked(&buf) }
            .to_string_lossy()
            .into_owned()
    })
}

/// Gets the executable path.
pub fn execpath() -> crate::Result<String> {
    let mut allocated = 32usize;
    let mut size = allocated - 1;
    let mut buf: Vec<std::os::raw::c_uchar> = vec![];
    while size == allocated - 1 {
        // path didn't fit in old size - double our allocation and try again
        allocated *= 2;
        size = allocated;
        buf.reserve(size - buf.len());

        // after uv_exepath, size will be the length of the string, *not* including the null
        crate::uvret( unsafe { uv_exepath(buf.as_mut_ptr() as _, &mut size as _) })?;
        unsafe { buf.set_len(size + 1) };
    }
    Ok(unsafe { CStr::from_bytes_with_nul_unchecked(&buf) }.to_string_lossy().into_owned())
}

/// Used to detect what type of stream should be used with a given file descriptor. Usually this
/// will be used during initialization to guess the type of the stdio streams.
///
/// For isatty(3) equivalent functionality use this function and test for TTY.
pub fn guess_handle(file: crate::File) -> crate::HandleType {
    unsafe { uv_guess_handle(file as _) }.into_inner()
}
