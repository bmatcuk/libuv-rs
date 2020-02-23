#[macro_use]
extern crate libuv_sys2 as uv;

pub mod error;
pub use error::*;
pub use error::Error::*;

pub mod version;
pub use version::*;

pub mod r#loop;
pub use r#loop::*;

mod addl_handle_data;
use addl_handle_data::*;
use addl_handle_data::AddlHandleData::*;

pub mod handle;
pub use handle::*;

pub mod req;
pub use req::*;

pub mod timer;
pub use timer::*;

pub type Result<T> = std::result::Result<T, error::Error>;

#[inline]
fn uvret(code: ::std::os::raw::c_int) -> Result<()> {
    if code < 0 {
        Err(Error::from(code as uv::uv_errno_t))
    } else {
        Ok(())
    }
}
