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

mod addl_handle_data;
use addl_handle_data::*;
use addl_handle_data::AddlHandleData::*;

pub mod handle;
pub use handle::*;

pub mod r#async;
pub use r#async::*;

pub mod buf;
pub use buf::*;

pub mod check;
pub use check::*;

pub mod fs_event;
pub use fs_event::*;

pub mod idle;
pub use idle::*;

pub mod prepare;
pub use prepare::*;

pub mod process;
pub use process::*;

pub mod signal;
pub use signal::*;

pub mod timer;
pub use timer::*;

mod req_data;
use req_data::*;
use req_data::ReqData::*;

pub mod req;
pub use req::*;

pub mod connect;
pub use connect::*;

pub mod shutdown;
pub use shutdown::*;

pub mod udp_send;
pub use udp_send::*;

pub mod write;
pub use write::*;

pub mod addl_stream_data;
use addl_stream_data::*;
use addl_stream_data::AddlStreamData::*;

pub mod stream;
pub use stream::*;

pub mod tcp;
pub use tcp::*;

pub mod tty;
pub use tty::*;

pub mod udp;
pub use udp::*;

pub type Result<T> = std::result::Result<T, Error>;

#[inline]
fn uvret(code: ::std::os::raw::c_int) -> Result<()> {
    if code < 0 {
        Err(Error::from_inner(code as uv::uv_errno_t))
    } else {
        Ok(())
    }
}
