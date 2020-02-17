#[macro_use]
extern crate libuv_sys2 as uv;

pub mod error;
pub use error::*;
pub use error::Error::*;

pub mod version;
pub use version::*;

pub mod r#loop;
pub use r#loop::*;

pub mod handle;
pub use handle::*;

pub type Result<T> = std::result::Result<T, error::Error>;

#[inline]
fn uvret(code: ::std::os::raw::c_int) -> Result<()> {
    if code < 0 {
        Err(Error::from(code as uv::uv_errno_t))
    } else {
        Ok(())
    }
}
