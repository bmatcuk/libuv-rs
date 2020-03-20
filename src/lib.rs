#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate libuv_sys2 as uv;

mod inner;
use inner::*;

pub mod error;
pub use error::*;
pub use error::Error::*;

pub mod version;
pub use version::*;

pub mod r#loop;
pub use r#loop::*;

pub mod buf;
pub use buf::*;

pub mod fs;
pub use fs::*;

pub mod net;
pub use net::*;

pub mod handles;
pub use handles::*;

pub mod requests;
pub use requests::*;

pub mod shared_libs;
pub use shared_libs::*;

pub mod misc;
pub use misc::*;

pub type Result<T> = std::result::Result<T, Error>;

#[inline]
fn uvret(code: ::std::os::raw::c_int) -> Result<()> {
    if code < 0 {
        Err(Error::from_inner(code as uv::uv_errno_t))
    } else {
        Ok(())
    }
}
